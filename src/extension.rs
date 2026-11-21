use std::ffi::OsStr;
use crate::model::{DetectionReport, ExtensionStatus, Identification};

/// Registry of known incompatible extensions for mismatch detection in P0.
const KNOWN_REGISTRY_EXTENSIONS: &[(&str, &str)] = &[
    ("png", "png"),
    ("pdf", "pdf"),
    ("docx", "docx"),
    ("docm", "docm"),
    ("xlsx", "xlsx"),
    ("xlsm", "xlsm"),
    ("pptx", "pptx"),
    ("pptm", "pptm"),
    ("doc", "doc"),
    ("dot", "doc"),
    ("xls", "xls"),
    ("xlt", "xls"),
    ("ppt", "ppt"),
    ("pot", "ppt"),
    ("pps", "ppt"),
    ("zip", "zip"),
    ("jpg", "jpeg"),
    ("jpeg", "jpeg"),
    ("gif", "gif"),
    ("bmp", "bmp"),
    ("7z", "7z"),
    ("rar", "rar"),
    ("mp4", "mp4"),
    ("m4v", "mp4"),
    ("m4a", "m4a"),
    ("mov", "mov"),
    ("qt", "mov"),
    ("webm", "webm"),
    ("mkv", "mkv"),
    ("mka", "mkv"),
    ("wav", "wav"),
    ("wave", "wav"),
    ("webp", "webp"),
    ("avi", "avi"),
    ("mp3", "mp3"),
    ("txt", "txt"),
];

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

    // Priority 3: Extension is in accepted_extensions
    if report.accepted_extensions.iter().any(|acc| acc.eq_ignore_ascii_case(&ext_str)) {
        return ExtensionStatus::Match;
    }

    // Priority 4: Extension belongs to another known incompatible format
    for (known_ext, target_fid) in KNOWN_REGISTRY_EXTENSIONS {
        if known_ext.eq_ignore_ascii_case(&ext_str) {
            if *target_fid != format_id {
                return ExtensionStatus::Mismatch;
            }
        }
    }

    // Priority 5: Extension is unrecognized in registry
    ExtensionStatus::Unrecognized
}
