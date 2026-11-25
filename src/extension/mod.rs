pub mod registry;

use std::ffi::OsStr;
use crate::model::{DetectionCore, DetectionReport, ExtensionStatus, Identification};
pub use registry::find_registered_format;

fn compare_extension_fields(
    identification: Identification,
    format_id: Option<&str>,
    accepted_extensions: impl IntoIterator<Item = impl AsRef<str>>,
    has_accepted_extensions: bool,
    extension: Option<&OsStr>,
) -> ExtensionStatus {
    // Priority 1: Missing extension
    let ext_os = match extension {
        Some(e) if !e.is_empty() => e,
        _ => return ExtensionStatus::Missing,
    };

    // Priority 2: Not a confirmed specific format or format cannot determine synonymous extension relationships
    if identification != Identification::Format || !has_accepted_extensions {
        return ExtensionStatus::Undetermined;
    }

    let format_id = match format_id {
        Some(fid) => fid,
        None => return ExtensionStatus::Undetermined,
    };

    // Safely check UTF-8 representation without lossy conversion or panicking
    let raw_str = match ext_os.to_str() {
        Some(s) => s.trim(),
        None => return ExtensionStatus::Unrecognized,
    };
    if raw_str.is_empty() {
        return ExtensionStatus::Missing;
    }

    // Priority 3: Fast-path local check against accepted_extensions (case-insensitive)
    if accepted_extensions.into_iter().any(|acc| acc.as_ref().eq_ignore_ascii_case(raw_str)) {
        return ExtensionStatus::Match;
    }

    // Zero-heap-allocation borrow if already ASCII lowercase
    let ext_cow = if raw_str.bytes().any(|b| b.is_ascii_uppercase()) {
        std::borrow::Cow::Owned(raw_str.to_ascii_lowercase())
    } else {
        std::borrow::Cow::Borrowed(raw_str)
    };

    // Priority 4: Bucket-indexed binary search in global sorted extension registry to detect format mismatch
    if let Some(target_fid) = find_registered_format(&ext_cow) {
        if target_fid != format_id {
            return ExtensionStatus::Mismatch;
        }
    }

    // Priority 5: Extension is unrecognized in registry
    ExtensionStatus::Unrecognized
}

/// Pure function for independent extension diagnosis on zero-allocation DetectionCore.
pub fn compare_extension_core(
    core: &DetectionCore,
    extension: Option<&OsStr>,
) -> ExtensionStatus {
    compare_extension_fields(
        core.identification,
        core.format_id,
        core.accepted_extensions.iter().copied(),
        !core.accepted_extensions.is_empty(),
        extension,
    )
}

/// Pure function for independent extension diagnosis according to specification §6.3.
pub fn compare_extension(
    report: &DetectionReport,
    extension: Option<&OsStr>,
) -> ExtensionStatus {
    compare_extension_fields(
        report.identification,
        report.format_id.as_deref(),
        report.accepted_extensions.iter().map(String::as_str),
        !report.accepted_extensions.is_empty(),
        extension,
    )
}
