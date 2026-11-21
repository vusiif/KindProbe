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
    ("elf", "elf"),
    ("so", "elf"),
    ("o", "elf"),
    ("exe", "pe"),
    ("dll", "pe"),
    ("sys", "pe"),
    ("efi", "pe"),
    ("wasm", "wasm"),
    ("dylib", "macho"),
    ("class", "class"),
    ("epub", "epub"),
    ("mobi", "mobi"),
    ("azw", "mobi"),
    ("prc", "mobi"),
    ("ttf", "ttf"),
    ("otf", "otf"),
    ("woff", "woff"),
    ("woff2", "woff2"),
    ("odt", "odt"),
    ("ods", "ods"),
    ("odp", "odp"),
    ("sqlite3", "sqlite3"),
    ("sqlite", "sqlite3"),
    ("db", "sqlite3"),
    ("db3", "sqlite3"),
    ("pcap", "pcap"),
    ("cap", "pcap"),
    ("pcapng", "pcapng"),
    ("parquet", "parquet"),
    ("avro", "avro"),
    ("qcow2", "qcow2"),
    ("qcow", "qcow2"),
    ("vmdk", "vmdk"),
    ("vhdx", "vhdx"),
    ("vhd", "vhd"),
    ("iso", "iso"),
    ("img", "iso"),
    ("tar", "tar"),
    ("gz", "gzip"),
    ("gzip", "gzip"),
    ("tgz", "gzip"),
    ("bz2", "bzip2"),
    ("bzip2", "bzip2"),
    ("tbz2", "bzip2"),
    ("tbz", "bzip2"),
    ("xz", "xz"),
    ("txz", "xz"),
    ("zst", "zstd"),
    ("zstd", "zstd"),
    ("tzst", "zstd"),
    ("json", "json"),
    ("xml", "xml"),
    ("html", "html"),
    ("htm", "html"),
    ("yaml", "yaml"),
    ("yml", "yaml"),
    ("toml", "toml"),
    ("csv", "csv"),
    ("tsv", "tsv"),
    ("tab", "tsv"),
    ("deb", "deb"),
    ("udeb", "deb"),
    ("rpm", "rpm"),
    ("crx", "crx"),
    ("flac", "flac"),
    ("ogg", "ogg"),
    ("oga", "ogg"),
    ("ogv", "ogg"),
    ("ogx", "ogg"),
    ("opus", "ogg"),
    ("psd", "psd"),
    ("tiff", "tiff"),
    ("tif", "tiff"),
    ("ico", "ico"),
    ("cur", "cur"),
    ("apk", "apk"),
    ("xapk", "apk"),
    ("ipa", "ipa"),
    ("jar", "jar"),
    ("war", "jar"),
    ("ear", "jar"),
    ("aac", "aac"),
    ("aif", "aiff"),
    ("aiff", "aiff"),
    ("aifc", "aiff"),
    ("mid", "mid"),
    ("midi", "mid"),
    ("chm", "chm"),
    ("txt", "txt"),
    ("text", "txt"),
    ("log", "txt"),
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
