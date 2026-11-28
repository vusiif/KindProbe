pub mod archives;
pub mod cfb;
pub mod databases;
pub mod disks;
pub mod ebooks;
pub mod executables;
pub mod fast_path;
pub mod fonts;
pub mod images;
pub mod media;
pub mod rules;
pub mod signatures;
pub mod text;
pub mod zip_ooxml;

use crate::budget::{BudgetReader, DetectOptions};
use crate::model::{
    DetectError, DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection,
    Verification, WarningItem,
};
use cfb::{detect_cfb, CFB_MAGIC};
use fast_path::{probe_fast_path, probe_fast_path_core, FastPathCoreOutcome};
use signatures::{probe_signatures, probe_signatures_core, SignatureMatch, SignatureMatchCore};
use std::io::{Read, Seek, SeekFrom};
use zip_ooxml::detect_zip_ooxml;

fn truncated_candidate_str(fmt: std::borrow::Cow<'static, str>) -> &'static str {
    match fmt {
        std::borrow::Cow::Borrowed(s) => s,
        std::borrow::Cow::Owned(ref s) => match s.as_str() {
            "pe" => "pe",
            "elf" => "elf",
            "wasm" => "wasm",
            "macho" => "macho",
            "7z" => "7z",
            "rar" => "rar",
            "zip" => "zip",
            "cfb" => "cfb",
            "tar" => "tar",
            "gzip" => "gzip",
            "bzip2" => "bzip2",
            "xz" => "xz",
            "zstd" => "zstd",
            "sqlite" | "sqlite3" => "sqlite3",
            "pcap" => "pcap",
            "pcapng" => "pcapng",
            "parquet" => "parquet",
            "avro" => "avro",
            "mp3" => "mp3",
            "mp4" => "mp4",
            "webm" => "webm",
            "mkv" => "mkv",
            "wav" => "wav",
            "webp" => "webp",
            "avi" => "avi",
            "iso" => "iso",
            "qcow2" => "qcow2",
            "vmdk" => "vmdk",
            "vhd" => "vhd",
            "vhdx" => "vhdx",
            "mobi" => "mobi",
            "palmdoc" => "palmdoc",
            "ttf" => "ttf",
            "otf" => "otf",
            "woff" => "woff",
            "woff2" => "woff2",
            _ => crate::extension::find_registered_format(s).unwrap_or("unknown"),
        },
    }
}

/// Evaluates an in-memory buffer for non-container formats (including ISO at offset 32768 when full buffer is provided).
/// Returns Ok(None) only if the input starts with a container signature (ZIP / CFB) and needs stream parsing.
fn make_dmg_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("dmg"),
        category: "disk",
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("dmg"),
        accepted_extensions: &["dmg"],
        primary_evidence: Some("sig.dmg.koly"),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}

fn probe_dmg_slice_core(bytes: &[u8]) -> Option<DetectionCore> {
    if bytes.len() < 512 {
        return None;
    }
    let trailer = &bytes[bytes.len() - 512..bytes.len() - 504];
    if &trailer[0..4] == b"koly"
        && u32::from_be_bytes([trailer[4], trailer[5], trailer[6], trailer[7]]) == 4
    {
        return Some(make_dmg_core());
    }
    None
}

fn probe_dmg_stream_core<R: Read + Seek>(reader: &mut BudgetReader<R>) -> Option<DetectionCore> {
    if reader.seek(std::io::SeekFrom::End(-512)).is_err() {
        return None;
    }
    let mut trailer = [0u8; 8];
    if reader.read_exact(&mut trailer).is_ok()
        && &trailer[0..4] == b"koly"
        && u32::from_be_bytes([trailer[4], trailer[5], trailer[6], trailer[7]]) == 4
    {
        return Some(make_dmg_core());
    }
    None
}

fn can_probe_beyond_header(bytes_read: usize, max_source_bytes_read: usize, needed: usize) -> bool {
    max_source_bytes_read.saturating_sub(bytes_read) >= needed
}

fn truncated_report(fmt: String, source_bytes_read: usize) -> DetectionReport {
    DetectionReport {
        schema_version: 1,
        format_id: None,
        category: "unknown".to_string(),
        container_id: None,
        identification: Identification::Unknown,
        verification: Verification::None,
        inspection: Inspection::Complete,
        recommended_extension: None,
        accepted_extensions: Vec::new(),
        extension_status: ExtensionStatus::Undetermined,
        evidence: Vec::new(),
        warnings: vec![WarningItem::new(
            "input.truncated",
            format!("Input header matches truncated signature for {}", fmt),
        )],
        stats: crate::model::DetectionStats {
            source_bytes_read,
            ..Default::default()
        },
        candidates: Some(vec![fmt]),
    }
}

fn probe_non_container_header_core(
    header: &[u8],
    options: &DetectOptions,
) -> Result<Option<DetectionCore>, DetectError> {
    if header.is_empty() {
        return Ok(Some(DetectionCore::empty()));
    }

    if let Some(core) = fast_path::probe_fast_path_slice_core(header, options)? {
        return Ok(Some(core));
    }

    match probe_signatures_core(header) {
        SignatureMatchCore::Confirmed(core) => return Ok(Some(core)),
        SignatureMatchCore::Truncated(fmt) => {
            return Ok(Some(DetectionCore::truncated(truncated_candidate_str(fmt))));
        }
        SignatureMatchCore::None => (),
    }

    if (header.len() >= 2 && header.starts_with(b"PK")) && header.len() < 4 {
        return Ok(Some(DetectionCore::truncated("zip")));
    }

    if header.len() >= 2 && header.starts_with(&[0xD0, 0xCF]) && header.len() < 8 {
        return Ok(Some(DetectionCore::truncated("cfb")));
    }

    if (header.len() >= 4
        && (header.starts_with(b"PK\x03\x04") || header.starts_with(b"PK\x05\x06")))
        || (header.len() >= 8 && header.starts_with(CFB_MAGIC))
    {
        return Ok(None);
    }

    // Reuse bytes already in the initial header window before text or seeks.
    if let Some(iso_core) = disks::probe_iso_buffer_core(header) {
        return Ok(Some(iso_core));
    }

    // Classify text/structured text from header before attempting out-of-window ISO/DMG seeks.
    if let Some(core) = text::probe_text_core(header) {
        return Ok(Some(core));
    }

    Ok(None)
}

pub fn detect_bytes_non_container_core(
    bytes: &[u8],
    options: &DetectOptions,
) -> Result<Option<DetectionCore>, DetectError> {
    let header_len = bytes
        .len()
        .min(options.initial_window_size)
        .min(options.max_source_bytes_read);
    let header = &bytes[..header_len];
    let header_is_eof = header_len == bytes.len();

    if let Some(core) = probe_non_container_header_core(header, options)? {
        return Ok(Some(core));
    }

    if (header.len() >= 4
        && (header.starts_with(b"PK\x03\x04") || header.starts_with(b"PK\x05\x06")))
        || (header.len() >= 8 && header.starts_with(CFB_MAGIC))
    {
        return Ok(None);
    }

    if header_is_eof {
        if let Some(dmg_core) = probe_dmg_slice_core(header) {
            return Ok(Some(dmg_core));
        }
        return Ok(Some(DetectionCore::unknown()));
    }

    let mut bytes_read = header.len();
    if header.len() < (disks::iso::ISO_OFFSET as usize) + 6
        && can_probe_beyond_header(bytes_read, options.max_source_bytes_read, 6)
    {
        if bytes.len() >= (disks::iso::ISO_OFFSET as usize) + 6 {
            bytes_read = bytes_read.saturating_add(6);
        }
        if let Some(iso_core) = disks::probe_iso_buffer_core(bytes) {
            return Ok(Some(iso_core));
        }
    }

    if can_probe_beyond_header(bytes_read, options.max_source_bytes_read, 8) {
        if let Some(dmg_core) = probe_dmg_slice_core(bytes) {
            return Ok(Some(dmg_core));
        }
    }

    Ok(Some(DetectionCore::unknown()))
}

/// Evaluates an in-memory buffer for non-container formats returning an owned DetectionReport.
/// Returns Ok(None) only if the input starts with a container signature (ZIP / CFB) and needs stream parsing.
pub fn detect_bytes_non_container(
    bytes: &[u8],
    options: &DetectOptions,
) -> Result<Option<DetectionReport>, DetectError> {
    let header_len = bytes
        .len()
        .min(options.initial_window_size)
        .min(options.max_source_bytes_read);
    let header = &bytes[..header_len];
    let header_is_eof = header_len == bytes.len();

    if header.is_empty() {
        return Ok(Some(
            DetectionCore::empty().to_report(Default::default(), ExtensionStatus::Undetermined),
        ));
    }

    if let Some(mut rep) = fast_path::probe_fast_path_slice(header, options)? {
        rep.stats.source_bytes_read = header.len();
        return Ok(Some(rep));
    }

    match probe_signatures(header) {
        SignatureMatch::Confirmed(mut rep) => {
            rep.stats.source_bytes_read = header.len();
            return Ok(Some(*rep));
        }
        SignatureMatch::Truncated(fmt) => {
            return Ok(Some(truncated_report(fmt, header.len())));
        }
        SignatureMatch::None => (),
    }

    if (header.len() >= 2 && header.starts_with(b"PK")) && header.len() < 4 {
        return Ok(Some(DetectionCore::truncated("zip").to_report(
            crate::model::DetectionStats {
                source_bytes_read: header.len(),
                ..Default::default()
            },
            ExtensionStatus::Undetermined,
        )));
    }

    if header.len() >= 2 && header.starts_with(&[0xD0, 0xCF]) && header.len() < 8 {
        return Ok(Some(DetectionCore::truncated("cfb").to_report(
            crate::model::DetectionStats {
                source_bytes_read: header.len(),
                ..Default::default()
            },
            ExtensionStatus::Undetermined,
        )));
    }

    if (header.len() >= 4
        && (header.starts_with(b"PK\x03\x04") || header.starts_with(b"PK\x05\x06")))
        || (header.len() >= 8 && header.starts_with(CFB_MAGIC))
    {
        return Ok(None);
    }

    if let disks::IsoMatch::Confirmed(mut rep) = disks::probe_iso_buffer(header) {
        rep.stats.source_bytes_read = header.len();
        return Ok(Some(*rep));
    }

    if let Some(mut rep) = text::probe_text(header) {
        rep.stats.source_bytes_read = header.len();
        return Ok(Some(rep));
    }

    if header_is_eof {
        if let Some(dmg_core) = probe_dmg_slice_core(header) {
            return Ok(Some(dmg_core.to_report(
                crate::model::DetectionStats {
                    source_bytes_read: header.len(),
                    ..Default::default()
                },
                ExtensionStatus::Undetermined,
            )));
        }
        return Ok(Some(DetectionCore::unknown().to_report(
            crate::model::DetectionStats {
                source_bytes_read: header.len(),
                ..Default::default()
            },
            ExtensionStatus::Undetermined,
        )));
    }

    let mut bytes_read = header.len();
    let mut seek_count = 0usize;
    if header.len() < (disks::iso::ISO_OFFSET as usize) + 6
        && can_probe_beyond_header(bytes_read, options.max_source_bytes_read, 6)
    {
        if bytes.len() >= (disks::iso::ISO_OFFSET as usize) + 6 {
            bytes_read = bytes_read.saturating_add(6);
        }
        seek_count = seek_count.saturating_add(1);
        if let disks::IsoMatch::Confirmed(mut rep) = disks::probe_iso_buffer(bytes) {
            rep.stats.source_bytes_read = bytes_read;
            rep.stats.seek_count = seek_count;
            return Ok(Some(*rep));
        }
    }

    if can_probe_beyond_header(bytes_read, options.max_source_bytes_read, 8) {
        if bytes.len() >= 512 {
            bytes_read = bytes_read.saturating_add(8);
        }
        seek_count = seek_count.saturating_add(1);
        if let Some(dmg_core) = probe_dmg_slice_core(bytes) {
            return Ok(Some(dmg_core.to_report(
                crate::model::DetectionStats {
                    source_bytes_read: bytes_read,
                    seek_count,
                    ..Default::default()
                },
                ExtensionStatus::Undetermined,
            )));
        }
    }

    Ok(Some(DetectionCore::unknown().to_report(
        crate::model::DetectionStats {
            source_bytes_read: bytes_read,
            seek_count,
            ..Default::default()
        },
        ExtensionStatus::Undetermined,
    )))
}

/// Direct in-memory slice detection returning zero-allocation DetectionCore.
/// Returns Ok(None) if the slice requires seekable container stream detection (ZIP, CFB, ISO).
pub fn detect_slice_non_container_core(
    header: &[u8],
    options: &DetectOptions,
) -> Result<Option<DetectionCore>, DetectError> {
    if header.len() >= options.initial_window_size {
        if header.is_empty() {
            return Ok(Some(DetectionCore::empty()));
        }
        if let Some(core) = fast_path::probe_fast_path_slice_core(header, options)? {
            return Ok(Some(core));
        }
        match probe_signatures_core(header) {
            SignatureMatchCore::Confirmed(core) => return Ok(Some(core)),
            SignatureMatchCore::Truncated(fmt) => {
                return Ok(Some(DetectionCore::truncated(truncated_candidate_str(fmt))));
            }
            SignatureMatchCore::None => (),
        }
        return Ok(None);
    }
    detect_bytes_non_container_core(header, options)
}

/// Direct in-memory slice detection for non-container formats.
/// Returns Ok(None) if the slice requires seekable container stream detection (ZIP, CFB, ISO).
pub fn detect_slice_non_container(
    header: &[u8],
    options: &DetectOptions,
) -> Result<Option<DetectionReport>, DetectError> {
    if header.is_empty() {
        return Ok(Some(DetectionCore::empty().to_report(
            crate::model::DetectionStats {
                source_bytes_read: 0,
                seek_count: 0,
                entries_inspected: 0,
                decompressed_bytes: 0,
                duration_micros: 0,
            },
            ExtensionStatus::Undetermined,
        )));
    }

    // Fast path for non-container
    if let Some(mut rep) = fast_path::probe_fast_path_slice(header, options)? {
        rep.stats.source_bytes_read = header.len();
        return Ok(Some(rep));
    }

    // Signatures and rules
    match probe_signatures(header) {
        SignatureMatch::Confirmed(mut rep) => {
            rep.stats.source_bytes_read = header.len();
            return Ok(Some(*rep));
        }
        SignatureMatch::Truncated(fmt) => {
            return Ok(Some(DetectionReport {
                schema_version: 1,
                format_id: None,
                category: "unknown".to_string(),
                container_id: None,
                identification: Identification::Unknown,
                verification: Verification::None,
                inspection: Inspection::Complete,
                recommended_extension: None,
                accepted_extensions: Vec::new(),
                extension_status: ExtensionStatus::Undetermined,
                evidence: Vec::new(),
                warnings: vec![WarningItem::new(
                    "input.truncated",
                    format!("Input header matches truncated signature for {}", fmt),
                )],
                stats: crate::model::DetectionStats {
                    source_bytes_read: header.len(),
                    seek_count: 0,
                    entries_inspected: 0,
                    decompressed_bytes: 0,
                    duration_micros: 0,
                },
                candidates: Some(vec![fmt]),
            }));
        }
        SignatureMatch::None => (),
    }

    // Truncated ZIP
    if (header.len() >= 2 && header.starts_with(b"PK")) && header.len() < 4 {
        return Ok(Some(DetectionCore::truncated("zip").to_report(
            crate::model::DetectionStats {
                source_bytes_read: header.len(),
                seek_count: 0,
                entries_inspected: 0,
                decompressed_bytes: 0,
                duration_micros: 0,
            },
            ExtensionStatus::Undetermined,
        )));
    }

    // Truncated CFB
    if header.len() >= 2 && header.starts_with(&[0xD0, 0xCF]) && header.len() < 8 {
        return Ok(Some(DetectionCore::truncated("cfb").to_report(
            crate::model::DetectionStats {
                source_bytes_read: header.len(),
                seek_count: 0,
                entries_inspected: 0,
                decompressed_bytes: 0,
                duration_micros: 0,
            },
            ExtensionStatus::Undetermined,
        )));
    }

    // If matches full container signature, delegate to stream detector
    if (header.len() >= 4
        && (header.starts_with(b"PK\x03\x04") || header.starts_with(b"PK\x05\x06")))
        || (header.len() >= 8 && header.starts_with(CFB_MAGIC))
        || (header.len() >= options.initial_window_size)
    {
        return Ok(None);
    }

    // Text formats
    if let Some(mut rep) = text::probe_text(header) {
        rep.stats.source_bytes_read = header.len();
        return Ok(Some(rep));
    }

    // Unknown format
    Ok(Some(DetectionCore::unknown().to_report(
        crate::model::DetectionStats {
            source_bytes_read: header.len(),
            seek_count: 0,
            entries_inspected: 0,
            decompressed_bytes: 0,
            duration_micros: 0,
        },
        ExtensionStatus::Undetermined,
    )))
}

enum HeaderReadOutcome {
    Ready {
        total_read: usize,
        reached_eof: bool,
    },
    BudgetExceeded,
}

fn read_stream_header_and_rewind<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    header_buf: &mut [u8],
    options: &DetectOptions,
) -> Result<HeaderReadOutcome, DetectError> {
    let mut total_read = 0;
    let mut reached_eof = false;
    while total_read < header_buf.len() {
        match reader.read(&mut header_buf[total_read..]) {
            Ok(0) => {
                reached_eof = true;
                break;
            }
            Ok(bytes) => total_read += bytes,
            Err(e) => {
                if reader.is_budget_exceeded() {
                    return Ok(HeaderReadOutcome::BudgetExceeded);
                }
                return Err(DetectError::Io(e));
            }
        }
    }

    if !reached_eof
        && options.max_source_bytes_read < options.initial_window_size
        && total_read == options.max_source_bytes_read
    {
        let mut probe = [0u8; 1];
        match reader.read(&mut probe) {
            Ok(0) => reached_eof = true,
            Ok(_) => {}
            Err(e) => {
                if reader.is_budget_exceeded() {
                    return Ok(HeaderReadOutcome::BudgetExceeded);
                }
                return Err(DetectError::Io(e));
            }
        }
    }

    if let Err(e) = reader.seek(SeekFrom::Start(0)) {
        if reader.is_budget_exceeded() {
            return Ok(HeaderReadOutcome::BudgetExceeded);
        }
        return Err(DetectError::Io(e));
    }

    Ok(HeaderReadOutcome::Ready {
        total_read,
        reached_eof,
    })
}

/// Zero-allocation stream detector returning DetectionCore directly for all non-ZIP/non-CFB formats.
pub fn detect_stream_core<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    options: &DetectOptions,
) -> Result<DetectionCore, DetectError> {
    let header_cap = options
        .initial_window_size
        .min(options.max_source_bytes_read);
    let mut stack_buf = [0u8; 4096];
    let mut heap_buf;
    let header_buf: &mut [u8] = if header_cap <= 4096 {
        &mut stack_buf[..header_cap]
    } else {
        heap_buf = vec![0u8; header_cap];
        &mut heap_buf[..]
    };

    let (total_read, reached_eof) =
        match read_stream_header_and_rewind(reader, header_buf, options)? {
            HeaderReadOutcome::Ready {
                total_read,
                reached_eof,
            } => (total_read, reached_eof),
            HeaderReadOutcome::BudgetExceeded => return Ok(DetectionCore::budget_exceeded()),
        };
    let header = &header_buf[..total_read];

    if header.is_empty() {
        return Ok(DetectionCore::empty());
    }

    match probe_fast_path_core(reader, header, options)? {
        FastPathCoreOutcome::Core(core) => return Ok(core),
        FastPathCoreOutcome::Report(rep) => return Ok(DetectionCore::from_report(&rep)),
        FastPathCoreOutcome::None => (),
    }

    match probe_signatures_core(header) {
        SignatureMatchCore::Confirmed(core) => return Ok(core),
        SignatureMatchCore::Truncated(fmt) => {
            return Ok(DetectionCore::truncated(truncated_candidate_str(fmt)));
        }
        SignatureMatchCore::None => (),
    }

    if header.len() >= 4 && (header.starts_with(b"PK\x03\x04") || header.starts_with(b"PK\x05\x06"))
    {
        let rep = detect_zip_ooxml(reader, options)?;
        return Ok(DetectionCore::from_report(&rep));
    } else if (header.len() >= 2 && header.starts_with(b"PK")) && header.len() < 4 {
        return Ok(DetectionCore::truncated("zip"));
    }

    if header.len() >= 8 && header.starts_with(CFB_MAGIC) {
        let rep = detect_cfb(reader, options)?;
        return Ok(DetectionCore::from_report(&rep));
    } else if header.len() >= 2 && header.starts_with(&[0xD0, 0xCF]) && header.len() < 8 {
        return Ok(DetectionCore::truncated("cfb"));
    }

    if let Some(iso_core) = disks::probe_iso_buffer_core(header) {
        return Ok(iso_core);
    }

    if let Some(core) = text::probe_text_core(header) {
        return Ok(core);
    }

    if reached_eof {
        if let Some(dmg_core) = probe_dmg_slice_core(header) {
            return Ok(dmg_core);
        }
        return Ok(DetectionCore::unknown());
    }

    if header.len() < (disks::iso::ISO_OFFSET as usize) + 6
        && can_probe_beyond_header(reader.total_read_bytes(), options.max_source_bytes_read, 6)
    {
        if let Some(core) = disks::probe_iso_stream_core(reader) {
            return Ok(core);
        }
    }

    if can_probe_beyond_header(reader.total_read_bytes(), options.max_source_bytes_read, 8) {
        if let Some(dmg_core) = probe_dmg_stream_core(reader) {
            return Ok(dmg_core);
        }
    }

    Ok(DetectionCore::unknown())
}

pub fn detect_stream<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    options: &DetectOptions,
) -> Result<DetectionReport, DetectError> {
    // Clamp header allocation to the readable budget ceiling (stack buffer up to 4 KiB)
    let header_cap = options
        .initial_window_size
        .min(options.max_source_bytes_read);
    let mut stack_buf = [0u8; 4096];
    let mut heap_buf;
    let header_buf: &mut [u8] = if header_cap <= 4096 {
        &mut stack_buf[..header_cap]
    } else {
        heap_buf = vec![0u8; header_cap];
        &mut heap_buf[..]
    };
    let (total_read, reached_eof) =
        match read_stream_header_and_rewind(reader, header_buf, options)? {
            HeaderReadOutcome::Ready {
                total_read,
                reached_eof,
            } => (total_read, reached_eof),
            HeaderReadOutcome::BudgetExceeded => {
                return Ok(DetectionCore::budget_exceeded()
                    .to_report(Default::default(), ExtensionStatus::Undetermined));
            }
        };
    let header = &header_buf[..total_read];

    // If stream is completely empty
    if header.is_empty() {
        return Ok(DetectionReport {
            schema_version: 1,
            format_id: None,
            category: "unknown".to_string(),
            container_id: None,
            identification: Identification::Unknown,
            verification: Verification::None,
            inspection: Inspection::Complete,
            recommended_extension: None,
            accepted_extensions: Vec::new(),
            extension_status: ExtensionStatus::Undetermined,
            evidence: Vec::new(),
            warnings: vec![WarningItem::new("input.empty", "Input file has 0 bytes")],
            stats: Default::default(),
            candidates: None,
        });
    }

    // Step 0: Extension-guided fast path probing (optimistic match)
    if let Some(rep) = probe_fast_path(reader, header, options)? {
        return Ok(rep);
    }

    // Step 1: Check signatures (PNG, JPEG, GIF, PDF)
    match probe_signatures(header) {
        SignatureMatch::Confirmed(rep) => return Ok(*rep),
        SignatureMatch::Truncated(fmt) => return Ok(truncated_report(fmt, 0)),
        SignatureMatch::None => (),
    }

    // Step 2: Check ZIP signature (PK\x03\x04 or PK\x05\x06)
    if header.len() >= 4 && (header.starts_with(b"PK\x03\x04") || header.starts_with(b"PK\x05\x06"))
    {
        return detect_zip_ooxml(reader, options);
    } else if (header.len() >= 2 && header.starts_with(b"PK")) && header.len() < 4 {
        return Ok(DetectionReport {
            schema_version: 1,
            format_id: None,
            category: "unknown".to_string(),
            container_id: None,
            identification: Identification::Unknown,
            verification: Verification::None,
            inspection: Inspection::Complete,
            recommended_extension: None,
            accepted_extensions: Vec::new(),
            extension_status: ExtensionStatus::Undetermined,
            evidence: Vec::new(),
            warnings: vec![WarningItem::new(
                "input.truncated",
                "Input header matches truncated ZIP signature",
            )],
            stats: Default::default(),
            candidates: Some(vec!["zip".to_string()]),
        });
    }

    // Step 3: Check CFB signature (\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1)
    if header.len() >= 8 && header.starts_with(CFB_MAGIC) {
        return detect_cfb(reader, options);
    } else if header.len() >= 2 && header.starts_with(&[0xD0, 0xCF]) && header.len() < 8 {
        return Ok(DetectionReport {
            schema_version: 1,
            format_id: None,
            category: "unknown".to_string(),
            container_id: None,
            identification: Identification::Unknown,
            verification: Verification::None,
            inspection: Inspection::Complete,
            recommended_extension: None,
            accepted_extensions: Vec::new(),
            extension_status: ExtensionStatus::Undetermined,
            evidence: Vec::new(),
            warnings: vec![WarningItem::new(
                "input.truncated",
                "Input header matches truncated CFB signature",
            )],
            stats: Default::default(),
            candidates: Some(vec!["cfb".to_string()]),
        });
    }

    // Step 4: Check ISO 9660 volume descriptor from already-read header if large enough
    if let Some(rep) = disks::probe_iso_buffer(header).into_report() {
        return Ok(rep);
    }

    // Step 5: Check structured and plain text formats (JSON, XML, HTML, YAML, TOML, CSV, TXT)
    if let Some(rep) = text::probe_text(header) {
        return Ok(rep);
    }

    if reached_eof {
        if let Some(dmg_core) = probe_dmg_slice_core(header) {
            return Ok(dmg_core.to_report(Default::default(), ExtensionStatus::Undetermined));
        }
    } else {
        if header.len() < (disks::iso::ISO_OFFSET as usize) + 6
            && can_probe_beyond_header(reader.total_read_bytes(), options.max_source_bytes_read, 6)
        {
            if let Some(rep) = disks::probe_iso_stream(reader) {
                return Ok(rep);
            }
        }

        if can_probe_beyond_header(reader.total_read_bytes(), options.max_source_bytes_read, 8) {
            if let Some(dmg_core) = probe_dmg_stream_core(reader) {
                return Ok(dmg_core.to_report(Default::default(), ExtensionStatus::Undetermined));
            }
        }
    }

    // Step 6: Unknown format
    Ok(DetectionReport {
        schema_version: 1,
        format_id: None,
        category: "unknown".to_string(),
        container_id: None,
        identification: Identification::Unknown,
        verification: Verification::None,
        inspection: Inspection::Complete,
        recommended_extension: None,
        accepted_extensions: Vec::new(),
        extension_status: ExtensionStatus::Undetermined,
        evidence: Vec::new(),
        warnings: vec![WarningItem::new(
            "format.unknown",
            "No matching signature or structural evidence found",
        )],
        stats: Default::default(),
        candidates: None,
    })
}
