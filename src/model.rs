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
        let format_id: Option<&'static str> = rep
            .format_id
            .as_deref()
            .and_then(|fid| static_format_id(fid).or_else(|| crate::extension::find_registered_format(fid)));
        let category: &'static str = match rep.category.as_str() {
            "archive" => "archive",
            "audio" => "audio",
            "automotive" => "automotive",
            "bioinformatics" => "bioinformatics",
            "blockchain" => "blockchain",
            "bytecode" => "bytecode",
            "cad" => "cad",
            "calendar" => "calendar",
            "capture" => "capture",
            "code" => "code",
            "configuration" => "configuration",
            "contact" => "contact",
            "container" => "container",
            "crypto" => "crypto",
            "data" => "data",
            "database" => "database",
            "disk" => "disk",
            "disk_image" => "disk_image",
            "document" => "document",
            "ebook" => "ebook",
            "emulation" => "emulation",
            "engineering" => "engineering",
            "exchange" => "exchange",
            "executable" => "executable",
            "finance" => "finance",
            "firmware" => "firmware",
            "font" => "font",
            "forensics" => "forensics",
            "game" => "game",
            "game_asset" => "game_asset",
            "geospatial" => "geospatial",
            "gis" => "gis",
            "hardware" => "hardware",
            "image" => "image",
            "location" => "location",
            "material" => "material",
            "media" => "media",
            "model" => "model",
            "multimedia" => "multimedia",
            "navigation" => "navigation",
            "network" => "network",
            "package" => "package",
            "robotics" => "robotics",
            "rom" => "rom",
            "schema" => "schema",
            "scientific" => "scientific",
            "security" => "security",
            "source" => "source",
            "system" => "system",
            "telecom" => "telecom",
            "text" => "text",
            "texture" => "texture",
            "translation" => "translation",
            "video" => "video",
            "virtualization" => "virtualization",
            "web" => "web",
            _ => "unknown",
        };
        let container_id: Option<&'static str> = rep.container_id.as_deref().and_then(|cid| {
            crate::extension::find_registered_format(cid).or(match cid {
                "riff" => Some("riff"),
                "cfb" => Some("cfb"),
                "xml" => Some("xml"),
                "json" => Some("json"),
                "zip" => Some("zip"),
                "tar" => Some("tar"),
                "gzip" => Some("gzip"),
                "bzip2" => Some("bzip2"),
                "xz" => Some("xz"),
                "zstd" => Some("zstd"),
                "7z" => Some("7z"),
                "rar" => Some("rar"),
                "isobmff" => Some("isobmff"),
                "matroska" => Some("matroska"),
                "palmdoc" => Some("palmdoc"),
                "sqlite3" => Some("sqlite3"),
                "ar" => Some("ar"),
                "ogg" => Some("ogg"),
                _ => None,
            })
        });

        let primary_evidence = rep.evidence.first().and_then(|e| find_static_evidence(&e.code));
        let secondary_evidence = rep.evidence.get(1).and_then(|e| find_static_evidence(&e.code));
        let accepted_extensions = format_id.map(static_accepted_extensions).unwrap_or(&[]);
        let recommended_extension = format_id.and_then(|fid| {
            rep.recommended_extension
                .as_deref()
                .and_then(|rec| if rec == fid { Some(fid) } else { static_recommended_extension(fid) })
                .or(Some(fid))
        });
        let warning = rep.warnings.first().and_then(|w| find_static_warning(&w.code));
        let candidate = rep
            .candidates
            .as_ref()
            .and_then(|c| c.first())
            .and_then(|s| static_format_id(s).or_else(|| crate::extension::find_registered_format(s)));

        Self {
            format_id,
            category,
            container_id,
            identification: rep.identification,
            verification: rep.verification,
            inspection: rep.inspection,
            recommended_extension,
            accepted_extensions,
            primary_evidence,
            secondary_evidence,
            warning,
            candidate,
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
        "zip.aab.bundle" => Some("zip.aab.bundle"),
        "zip.ipa.payload" => Some("zip.ipa.payload"),
        "zip.jar.manifest" => Some("zip.jar.manifest"),
        "zip.kmz.kml" => Some("zip.kmz.kml"),
        "zip.npz.array" => Some("zip.npz.array"),
        "zip.whl.wheel" => Some("zip.whl.wheel"),
        "zip.vsix.manifest" => Some("zip.vsix.manifest"),
        "zip.nupkg.nuspec" => Some("zip.nupkg.nuspec"),
        "zip.xpi.manifest" => Some("zip.xpi.manifest"),
        "zip.msix.manifest" => Some("zip.msix.manifest"),
        "ooxml.word_main_part" => Some("ooxml.word_main_part"),
        "ooxml.word_macro_main_part" => Some("ooxml.word_macro_main_part"),
        "ooxml.spreadsheet_main_part" => Some("ooxml.spreadsheet_main_part"),
        "ooxml.spreadsheet_macro_main_part" => Some("ooxml.spreadsheet_macro_main_part"),
        "ooxml.presentation_main_part" => Some("ooxml.presentation_main_part"),
        "ooxml.presentation_macro_main_part" => Some("ooxml.presentation_macro_main_part"),
        _ => None,
    }
}

fn static_format_id(fid: &str) -> Option<&'static str> {
    match fid {
        "docx" => Some("docx"),
        "xlsx" => Some("xlsx"),
        "pptx" => Some("pptx"),
        "docm" => Some("docm"),
        "xlsm" => Some("xlsm"),
        "pptm" => Some("pptm"),
        "doc" => Some("doc"),
        "xls" => Some("xls"),
        "ppt" => Some("ppt"),
        "zip" => Some("zip"),
        "apk" => Some("apk"),
        "aab" => Some("aab"),
        "ipa" => Some("ipa"),
        "jar" => Some("jar"),
        "kmz" => Some("kmz"),
        "npz" => Some("npz"),
        "whl" => Some("whl"),
        "vsix" => Some("vsix"),
        "nupkg" => Some("nupkg"),
        "xpi" => Some("xpi"),
        "msix" => Some("msix"),
        "epub" => Some("epub"),
        "odt" => Some("odt"),
        "ods" => Some("ods"),
        "odp" => Some("odp"),
        "odg" => Some("odg"),
        "iso" => Some("iso"),
        "macho" => Some("macho"),
        "sqlite3" => Some("sqlite3"),
        "gpkg" => Some("gpkg"),
        "tar.gz" => Some("tar.gz"),
        "svgz" => Some("svgz"),
        _ => None,
    }
}

fn static_recommended_extension(fid: &str) -> Option<&'static str> {
    match fid {
        "jpeg" => Some("jpg"),
        "tiff" => Some("tif"),
        "gzip" => Some("gz"),
        "bzip2" => Some("bz2"),
        "zstd" => Some("zst"),
        "macho" => Some("dylib"),
        "palmdoc" => Some("prc"),
        _ => None,
    }
}

fn static_accepted_extensions(fid: &str) -> &'static [&'static str] {
    match fid {
        "docx" => &["docx"],
        "xlsx" => &["xlsx"],
        "pptx" => &["pptx"],
        "docm" => &["docm"],
        "xlsm" => &["xlsm"],
        "pptm" => &["pptm"],
        "doc" => &["doc", "dot"],
        "xls" => &["xls", "xlt"],
        "ppt" => &["ppt", "pps"],
        "zip" => &["zip"],
        "apk" => &["apk", "xapk"],
        "aab" => &["aab"],
        "ipa" => &["ipa"],
        "jar" => &["jar", "war", "ear"],
        "kmz" => &["kmz", "kml"],
        "npz" => &["npz"],
        "whl" => &["whl"],
        "vsix" => &["vsix"],
        "nupkg" => &["nupkg"],
        "xpi" => &["xpi"],
        "msix" => &["msix", "appx"],
        "epub" => &["epub"],
        "odt" => &["odt"],
        "ods" => &["ods"],
        "odp" => &["odp"],
        "odg" => &["odg"],
        "iso" => &["iso", "img"],
        "png" => &["png"],
        "jpeg" => &["jpg", "jpeg"],
        "gif" => &["gif"],
        "pdf" => &["pdf"],
        "7z" => &["7z"],
        "rar" => &["rar"],
        "gzip" => &["gz", "gzip", "tgz"],
        "bzip2" => &["bz2", "bzip2", "tbz2", "tbz"],
        "xz" => &["xz", "txz"],
        "zstd" => &["zst", "zstd", "tzst"],
        "tar" => &["tar"],
        "tar.gz" => &["tar.gz", "tgz", "gz"],
        "svgz" => &["svgz", "svg", "gz"],
        "elf" => &["elf", "so", "o", "bin"],
        "pe" => &["exe", "dll", "sys", "efi"],
        "wasm" => &["wasm"],
        "macho" => &["dylib", "bundle", "so", "bin"],
        "class" => &["class"],
        "sqlite3" => &["sqlite3", "sqlite", "db", "db3"],
        "gpkg" => &["gpkg", "sqlite3", "db"],
        "pcap" => &["pcap", "cap", "dmp"],
        "pcapng" => &["pcapng", "ntar"],
        "parquet" => &["parquet"],
        "avro" => &["avro"],
        "qcow2" => &["qcow2", "qcow", "img"],
        "vmdk" => &["vmdk"],
        "vhd" => &["vhd"],
        "vhdx" => &["vhdx"],
        "mobi" => &["mobi", "azw", "prc"],
        "mp4" => &["mp4", "m4v", "m4a"],
        "mov" => &["mov", "qt"],
        "m4a" => &["m4a", "mp4"],
        "webm" => &["webm"],
        "mkv" => &["mkv", "mka"],
        "wav" => &["wav", "wave"],
        "webp" => &["webp"],
        "avi" => &["avi"],
        "mp3" => &["mp3"],
        "ttf" => &["ttf"],
        "otf" => &["otf"],
        "woff" => &["woff"],
        "woff2" => &["woff2"],
        "json" => &["json"],
        "geojson" => &["geojson", "json"],
        "xml" => &["xml"],
        "html" => &["html", "htm"],
        "svg" => &["svg"],
        "yaml" => &["yaml", "yml"],
        "toml" => &["toml"],
        "csv" => &["csv"],
        "tsv" => &["tsv", "tab"],
        "txt" => &["txt", "text", "log"],
        _ => &[],
    }
}

fn find_static_warning(code: &str) -> Option<(&'static str, &'static str)> {
    match code {
        "input.empty" => Some(("input.empty", "Input file has 0 bytes")),
        "format.unknown" => Some(("format.unknown", "No matching signature or structural evidence found")),
        "input.truncated" => Some(("input.truncated", "Input header matches truncated signature")),
        "budget.source_read_exceeded" => Some((
            "budget.source_read_exceeded",
            "Total source read budget exceeded during detection",
        )),
        "budget.zip_directory_window_exceeded" => Some((
            "budget.zip_directory_window_exceeded",
            "ZIP central directory read window exceeded budget limit",
        )),
        "budget.entries_exceeded" => Some((
            "budget.entries_exceeded",
            "Container entry count exceeded budget limit",
        )),
        "budget.xml_output_exceeded" => Some((
            "budget.xml_output_exceeded",
            "Decompressed XML output exceeded single XML limit",
        )),
        "budget.total_decompressed_exceeded" => Some((
            "budget.total_decompressed_exceeded",
            "Cumulative decompressed byte budget exceeded",
        )),
        "budget.xml_depth_exceeded" => Some((
            "budget.xml_depth_exceeded",
            "XML element depth exceeded allowed maximum",
        )),
        "zip.conflicting_entries" => Some((
            "zip.conflicting_entries",
            "Conflicting or duplicate ZIP entries detected",
        )),
        "zip.truncated_or_corrupted" => Some((
            "zip.truncated_or_corrupted",
            "Failed to parse ZIP central directory or archive",
        )),
        "ooxml.content_types_missing" => Some((
            "ooxml.content_types_missing",
            "OOXML package structure incomplete: [Content_Types].xml missing",
        )),
        "ooxml.content_types_xml_error" => Some((
            "ooxml.content_types_xml_error",
            "XML error parsing [Content_Types].xml",
        )),
        "ooxml.required_relationship_missing" => Some((
            "ooxml.required_relationship_missing",
            "No officeDocument core relationship found in _rels/.rels",
        )),
        "ooxml.rels_xml_error" => Some((
            "ooxml.rels_xml_error",
            "XML error parsing _rels/.rels",
        )),
        "ooxml.conflicting_relationships" => Some((
            "ooxml.conflicting_relationships",
            "Multiple officeDocument core relationships found in _rels/.rels",
        )),
        "ooxml.invalid_target_path" => Some((
            "ooxml.invalid_target_path",
            "Relationship target path resolves outside package root",
        )),
        "ooxml.content_type_not_found" => Some((
            "ooxml.content_type_not_found",
            "No content type found for target part",
        )),
        "ooxml.unsupported_content_type" => Some((
            "ooxml.unsupported_content_type",
            "Content type is not a supported OOXML document type",
        )),
        "ooxml.main_part_missing" => Some((
            "ooxml.main_part_missing",
            "Failed to open main document part in package",
        )),
        "ooxml.main_part_xml_error" => Some((
            "ooxml.main_part_xml_error",
            "XML error parsing main part",
        )),
        "ooxml.invalid_root_element" => Some((
            "ooxml.invalid_root_element",
            "Main part XML does not have valid root element",
        )),
        "cfb.corrupted_or_invalid_chain" => Some((
            "cfb.corrupted_or_invalid_chain",
            "Failed to open compound document",
        )),
        "cfb.invalid_word_fib" => Some((
            "cfb.invalid_word_fib",
            "Invalid or truncated WordDocument FIB structure",
        )),
        "cfb.unsupported_word_version" => Some((
            "cfb.unsupported_word_version",
            "Unsupported legacy Word FIB version",
        )),
        "cfb.document_encrypted" => Some((
            "cfb.document_encrypted",
            "Word document is encrypted",
        )),
        "cfb.table_stream_missing" => Some((
            "cfb.table_stream_missing",
            "Required Word Table stream (0Table or 1Table) missing",
        )),
        "cfb.invalid_biff_bof" => Some((
            "cfb.invalid_biff_bof",
            "Invalid Excel Workbook BIFF BOF header",
        )),
        "cfb.unsupported_biff_version" => Some((
            "cfb.unsupported_biff_version",
            "Unsupported Excel BIFF version",
        )),
        "cfb.invalid_powerpoint_stream" => Some((
            "cfb.invalid_powerpoint_stream",
            "Invalid PowerPoint Document or Current User stream",
        )),
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
