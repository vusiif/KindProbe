pub mod registry;

use std::ffi::OsStr;
use crate::model::{DetectionReport, ExtensionStatus, Identification};
pub use registry::find_registered_format;

/// Pure function for independent extension diagnosis according to specification §6.3.
pub fn compare_extension(
    report: &DetectionReport,
    extension: Option<&OsStr>,
) -> ExtensionStatus {
    // Priority 1: Missing extension
    let ext_os = match extension {
        Some(e) if !e.is_empty() => e,
        _ => return ExtensionStatus::Missing,
    };

    // Priority 2: Not a confirmed specific format or format cannot determine synonymous extension relationships
    if report.identification != Identification::Format || report.accepted_extensions.is_empty() {
        return ExtensionStatus::Undetermined;
    }

    let format_id = match &report.format_id {
        Some(fid) => fid.as_str(),
        None => return ExtensionStatus::Undetermined,
    };

    // Safely check UTF-8 representation without lossy loss or panicking
    let ext_str = match ext_os.to_str() {
        Some(s) => s.trim().to_ascii_lowercase(),
        None => return ExtensionStatus::Unrecognized,
    };

    // Priority 3: Fast-path local check against accepted_extensions
    if report.accepted_extensions.iter().any(|acc| acc.eq_ignore_ascii_case(&ext_str)) {
        return ExtensionStatus::Match;
    }

    // Priority 4: Binary search in global sorted extension registry to detect format mismatch
    if let Some(target_fid) = find_registered_format(&ext_str) {
        if target_fid != format_id {
            return ExtensionStatus::Mismatch;
        }
    }

    // Priority 5: Extension is unrecognized in registry
    ExtensionStatus::Unrecognized
}
