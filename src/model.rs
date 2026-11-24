use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Identification {
    Format,
    ContainerOnly,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verification {
    Signature,
    Structure,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Inspection {
    Complete,
    Limited,
    Ambiguous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionStatus {
    Match,
    Mismatch,
    Missing,
    Unrecognized,
    Undetermined,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceItem {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl EvidenceItem {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            description: None,
        }
    }

    pub fn with_desc(code: impl Into<String>, desc: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            description: Some(desc.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WarningItem {
    pub code: String,
    pub message: String,
}

impl WarningItem {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DetectionStats {
    pub source_bytes_read: usize,
    pub seek_count: usize,
    pub entries_inspected: usize,
    pub decompressed_bytes: usize,
    pub duration_micros: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DetectionReport {
    pub schema_version: u32,
    pub format_id: Option<String>,
    pub category: String,
    pub container_id: Option<String>,
    pub identification: Identification,
    pub verification: Verification,
    pub inspection: Inspection,
    pub recommended_extension: Option<String>,
    pub accepted_extensions: Vec<String>,
    pub extension_status: ExtensionStatus,
    pub evidence: Vec<EvidenceItem>,
    pub warnings: Vec<WarningItem>,
    pub stats: DetectionStats,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates: Option<Vec<String>>,
}

impl DetectionReport {
    /// Validates the legal state invariants according to specification §6.2.
    pub fn validate_invariants(&self) -> Result<(), &'static str> {
        if self.schema_version != 1 {
            return Err("schema_version must be 1");
        }

        // Rule: identification == format <=> format_id is Some
        if self.identification == Identification::Format {
            if self.format_id.is_none() {
                return Err("identification=format requires format_id to be non-null");
            }
        } else if self.format_id.is_some() {
            return Err("identification != format requires format_id to be null");
        }

        // Rule: identification != format => recommended_extension must be None
        if self.identification != Identification::Format && self.recommended_extension.is_some() {
            return Err("identification != format requires recommended_extension to be null");
        }

        // Rule: FORBIDDEN combination: format + ambiguous
        if self.identification == Identification::Format && self.inspection == Inspection::Ambiguous {
            return Err("FORBIDDEN: identification=format cannot be ambiguous");
        }

        // Rule: verification=none cannot coexist with confirmed format or container
        if self.verification == Verification::None
            && (self.identification == Identification::Format || self.container_id.is_some())
        {
            return Err("verification=none cannot coexist with confirmed format or container");
        }

        Ok(())
    }

    /// Computes the CLI exit code according to specification §8.
    pub fn cli_exit_code(&self) -> i32 {
        if self.identification == Identification::Format && self.inspection == Inspection::Complete {
            0
        } else {
            3
        }
    }
}

/// Lightweight, zero-allocation detection result representation.
/// Contains borrowed metadata for high-throughput filtering, scanner pipelines, and low-latency probing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DetectionCore {
    pub format_id: Option<&'static str>,
    pub category: &'static str,
    pub container_id: Option<&'static str>,
    pub identification: Identification,
    pub verification: Verification,
    pub inspection: Inspection,
    pub recommended_extension: Option<&'static str>,
    pub accepted_extensions: &'static [&'static str],
    pub primary_evidence: Option<&'static str>,
    pub secondary_evidence: Option<&'static str>,
    pub warning: Option<(&'static str, &'static str)>,
    pub candidate: Option<&'static str>,
}

impl DetectionCore {
    pub const fn empty() -> Self {
        Self {
            format_id: None,
            category: "unknown",
            container_id: None,
            identification: Identification::Unknown,
            verification: Verification::None,
            inspection: Inspection::Complete,
            recommended_extension: None,
            accepted_extensions: &[],
            primary_evidence: None,
            secondary_evidence: None,
            warning: Some(("input.empty", "Input file has 0 bytes")),
            candidate: None,
        }
    }

    pub const fn unknown() -> Self {
        Self {
            format_id: None,
            category: "unknown",
            container_id: None,
            identification: Identification::Unknown,
            verification: Verification::None,
            inspection: Inspection::Complete,
            recommended_extension: None,
            accepted_extensions: &[],
            primary_evidence: None,
            secondary_evidence: None,
            warning: Some(("format.unknown", "No matching signature or structural evidence found")),
            candidate: None,
        }
    }

    pub const fn budget_exceeded() -> Self {
        Self {
            format_id: None,
            category: "unknown",
            container_id: None,
            identification: Identification::Unknown,
            verification: Verification::None,
            inspection: Inspection::Limited,
            recommended_extension: None,
            accepted_extensions: &[],
            primary_evidence: None,
            secondary_evidence: None,
            warning: Some((
                "budget.source_read_exceeded",
                "Total source read budget exceeded during detection",
            )),
            candidate: None,
        }
    }

    pub const fn truncated(candidate: &'static str) -> Self {
        Self {
            format_id: None,
            category: "unknown",
            container_id: None,
            identification: Identification::Unknown,
            verification: Verification::None,
            inspection: Inspection::Complete,
            recommended_extension: None,
            accepted_extensions: &[],
            primary_evidence: None,
            secondary_evidence: None,
            warning: Some(("input.truncated", "Input header matches truncated signature")),
            candidate: Some(candidate),
        }
    }

    /// Converts the zero-allocation DetectionCore into an owned DetectionReport.
    pub fn to_report(&self, stats: DetectionStats, extension_status: ExtensionStatus) -> DetectionReport {
        let mut evidence = Vec::new();
        if let Some(code) = self.primary_evidence {
            evidence.push(EvidenceItem::new(code));
        }
        if let Some(code) = self.secondary_evidence {
            evidence.push(EvidenceItem::new(code));
        }

        let mut accepted_extensions = Vec::with_capacity(self.accepted_extensions.len());
        for &ext in self.accepted_extensions {
            accepted_extensions.push(ext.to_string());
        }

        let warnings = match self.warning {
            Some((code, msg)) => vec![WarningItem::new(code, msg)],
            None => Vec::new(),
        };

        let candidates = self.candidate.map(|c| vec![c.to_string()]);

        DetectionReport {
            schema_version: 1,
            format_id: self.format_id.map(|s| s.to_string()),
            category: self.category.to_string(),
            container_id: self.container_id.map(|s| s.to_string()),
            identification: self.identification,
            verification: self.verification,
            inspection: self.inspection,
            recommended_extension: self.recommended_extension.map(|s| s.to_string()),
            accepted_extensions,
            extension_status,
            evidence,
            warnings,
            stats,
            candidates,
        }
    }

    /// Validates invariants according to specification §6.2.
    pub fn validate_invariants(&self) -> Result<(), &'static str> {
        if self.identification == Identification::Format {
            if self.format_id.is_none() {
                return Err("identification=format requires format_id to be non-null");
            }
        } else if self.format_id.is_some() {
            return Err("identification != format requires format_id to be null");
        }

        if self.identification != Identification::Format && self.recommended_extension.is_some() {
            return Err("identification != format requires recommended_extension to be null");
        }

        if self.identification == Identification::Format && self.inspection == Inspection::Ambiguous {
            return Err("FORBIDDEN: identification=format cannot be ambiguous");
        }

        if self.verification == Verification::None
            && (self.identification == Identification::Format || self.container_id.is_some())
        {
            return Err("verification=none cannot coexist with confirmed format or container");
        }

        Ok(())
    }

    /// Constructs a DetectionCore from an existing DetectionReport.
    pub fn from_report(rep: &DetectionReport) -> Self {
        let format_id: Option<&'static str> = rep.format_id.as_deref().and_then(crate::extension::find_registered_format);
        let category: &'static str = match rep.category.as_str() {
            "image" => "image",
            "video" => "video",
            "audio" => "audio",
            "document" => "document",
            "archive" => "archive",
            "executable" => "executable",
            "font" => "font",
            "database" => "database",
            "disk" => "disk",
            "ebook" => "ebook",
            "text" => "text",
            "geospatial" => "geospatial",
            "model" => "model",
            "configuration" => "configuration",
            "container" => "container",
            _ => "unknown",
        };
        let container_id: Option<&'static str> = rep.container_id.as_deref().and_then(|cid| {
            crate::extension::find_registered_format(cid).or(match cid {
                "riff" => Some("riff"),
                "cfb" => Some("cfb"),
                "xml" => Some("xml"),
                "json" => Some("json"),
                _ => None,
            })
        });

        let primary_evidence = rep.evidence.first().and_then(|e| find_static_evidence(&e.code));
        let secondary_evidence = rep.evidence.get(1).and_then(|e| find_static_evidence(&e.code));

        Self {
            format_id,
            category,
            container_id,
            identification: rep.identification,
            verification: rep.verification,
            inspection: rep.inspection,
            recommended_extension: format_id,
            accepted_extensions: &[],
            primary_evidence,
            secondary_evidence,
            warning: None,
            candidate: None,
        }
    }
}

/// Resolves a known evidence string code to a static string slice.
pub fn find_static_evidence(code: &str) -> Option<&'static str> {
    match code {
        "sig.7z.header" => Some("sig.7z.header"),
        "sig.7z.start_header" => Some("sig.7z.start_header"),
        "sig.rar.header" => Some("sig.rar.header"),
        "sig.rar.main_header" => Some("sig.rar.main_header"),
        "sig.rar5.signature" => Some("sig.rar5.signature"),
        "sig.gzip.magic" => Some("sig.gzip.magic"),
        "sig.gzip.header" => Some("sig.gzip.header"),
        "sig.bzip2.magic" => Some("sig.bzip2.magic"),
        "sig.bzip2.header" => Some("sig.bzip2.header"),
        "sig.tar.ustar" => Some("sig.tar.ustar"),
        "sig.tar.checksum" => Some("sig.tar.checksum"),
        "sig.xz.magic" => Some("sig.xz.magic"),
        "sig.xz.crc" => Some("sig.xz.crc"),
        "sig.zstd.magic" => Some("sig.zstd.magic"),
        "sig.png.header" => Some("sig.png.header"),
        "sig.png.ihdr" => Some("sig.png.ihdr"),
        "sig.jpeg.soi" => Some("sig.jpeg.soi"),
        "sig.jpeg.segment" => Some("sig.jpeg.segment"),
        "sig.gif.header" => Some("sig.gif.header"),
        "sig.gif.logical_screen" => Some("sig.gif.logical_screen"),
        "sig.pdf.header" => Some("sig.pdf.header"),
        "sig.riff.header" => Some("sig.riff.header"),
        "sig.wav.wave_header" => Some("sig.wav.wave_header"),
        "sig.wav.fmt_chunk" => Some("sig.wav.fmt_chunk"),
        "sig.webp.header" => Some("sig.webp.header"),
        "sig.webp.vp8" => Some("sig.webp.vp8"),
        "sig.avi.header" => Some("sig.avi.header"),
        "sig.ebml.header" => Some("sig.ebml.header"),
        "sig.ebml.doctype_webm" => Some("sig.ebml.doctype_webm"),
        "sig.ebml.doctype_mkv" => Some("sig.ebml.doctype_mkv"),
        "sig.id3.tag" => Some("sig.id3.tag"),
        "sig.mp3.frame_sync" => Some("sig.mp3.frame_sync"),
        "sig.mp4.ftyp" => Some("sig.mp4.ftyp"),
        "sig.mov.brand" => Some("sig.mov.brand"),
        "sig.mp4.brand" => Some("sig.mp4.brand"),
        "sig.mov.header" => Some("sig.mov.header"),
        "sig.elf.magic" => Some("sig.elf.magic"),
        "sig.elf.header" => Some("sig.elf.header"),
        "sig.pe.mz_header" => Some("sig.pe.mz_header"),
        "sig.pe.signature" => Some("sig.pe.signature"),
        "sig.wasm.magic" => Some("sig.wasm.magic"),
        "sig.wasm.version" => Some("sig.wasm.version"),
        "sig.macho.magic" => Some("sig.macho.magic"),
        "sig.java.class" => Some("sig.java.class"),
        "sig.macho.fat_magic" => Some("sig.macho.fat_magic"),
        "sig.macho.universal" => Some("sig.macho.universal"),
        "sig.font.ttf_magic" => Some("sig.font.ttf_magic"),
        "sig.font.otf_magic" => Some("sig.font.otf_magic"),
        "sig.font.num_tables" => Some("sig.font.num_tables"),
        "sig.font.woff_magic" => Some("sig.font.woff_magic"),
        "sig.font.woff2_magic" => Some("sig.font.woff2_magic"),
        "sig.sqlite.header" => Some("sig.sqlite.header"),
        "sig.sqlite.page_size" => Some("sig.sqlite.page_size"),
        "sig.parquet.magic" => Some("sig.parquet.magic"),
        "sig.avro.magic" => Some("sig.avro.magic"),
        "sig.pcap.magic" => Some("sig.pcap.magic"),
        "sig.pcap.version" => Some("sig.pcap.version"),
        "sig.pcapng.shb" => Some("sig.pcapng.shb"),
        "sig.pcapng.bom" => Some("sig.pcapng.bom"),
        "sig.iso.volume_descriptor" => Some("sig.iso.volume_descriptor"),
        "sig.qcow.magic" => Some("sig.qcow.magic"),
        "sig.qcow.header" => Some("sig.qcow.header"),
        "sig.vhdx.magic" => Some("sig.vhdx.magic"),
        "sig.vhd.magic" => Some("sig.vhd.magic"),
        "sig.vmdk.sparse_magic" => Some("sig.vmdk.sparse_magic"),
        "sig.vmdk.version" => Some("sig.vmdk.version"),
        "sig.vmdk.desc_magic" => Some("sig.vmdk.desc_magic"),
        "sig.mobi.header" => Some("sig.mobi.header"),
        "sig.palmdoc.header" => Some("sig.palmdoc.header"),
        "cfb.header" => Some("cfb.header"),
        "cfb.directory" => Some("cfb.directory"),
        "cfb.word_document_stream" => Some("cfb.word_document_stream"),
        "cfb.fib_word97" => Some("cfb.fib_word97"),
        "cfb.workbook_stream" => Some("cfb.workbook_stream"),
        "cfb.biff_record" => Some("cfb.biff_record"),
        "cfb.powerpoint_stream" => Some("cfb.powerpoint_stream"),
        "cfb.current_user_stream" => Some("cfb.current_user_stream"),
        "epub.mimetype" => Some("epub.mimetype"),
        "epub.container" => Some("epub.container"),
        "odf.mimetype" => Some("odf.mimetype"),
        "zip.central_directory" => Some("zip.central_directory"),
        "ooxml.package_relationships" => Some("ooxml.package_relationships"),
        "ooxml.content_types" => Some("ooxml.content_types"),
        "zip.apk.manifest" => Some("zip.apk.manifest"),
        "zip.apk.dex" => Some("zip.apk.dex"),
        "zip.ipa.payload" => Some("zip.ipa.payload"),
        "zip.jar.manifest" => Some("zip.jar.manifest"),
        "zip.kmz.kml" => Some("zip.kmz.kml"),
        "ooxml.word_main_part" => Some("ooxml.word_main_part"),
        "ooxml.word_macro_main_part" => Some("ooxml.word_macro_main_part"),
        "ooxml.spreadsheet_main_part" => Some("ooxml.spreadsheet_main_part"),
        "ooxml.spreadsheet_macro_main_part" => Some("ooxml.spreadsheet_macro_main_part"),
        "ooxml.presentation_main_part" => Some("ooxml.presentation_main_part"),
        "ooxml.presentation_macro_main_part" => Some("ooxml.presentation_macro_main_part"),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    pub schema_version: u32,
    pub error: ErrorDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
}

#[derive(Debug)]
pub enum DetectError {
    Io(std::io::Error),
    NotRegularFile(String),
    InvalidParameter(String),
}

impl fmt::Display for DetectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DetectError::Io(e) => write!(f, "I/O error: {}", e),
            DetectError::NotRegularFile(path) => write!(f, "Not a regular file: {}", path),
            DetectError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
        }
    }
}

impl std::error::Error for DetectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DetectError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for DetectError {
    fn from(e: std::io::Error) -> Self {
        DetectError::Io(e)
    }
}
