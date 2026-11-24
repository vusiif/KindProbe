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

use std::io::{Read, Seek, SeekFrom};
use crate::budget::{BudgetReader, DetectOptions};
use crate::model::{
    DetectError, DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
    WarningItem,
};
use signatures::{probe_signatures, probe_signatures_core, SignatureMatch, SignatureMatchCore};
use zip_ooxml::detect_zip_ooxml;
use cfb::{detect_cfb, CFB_MAGIC};
use fast_path::probe_fast_path;

/// Direct in-memory slice detection returning zero-allocation DetectionCore.
/// Returns Ok(None) if the slice requires seekable container stream detection (ZIP, CFB, ISO).
pub fn detect_slice_non_container_core(
    header: &[u8],
    options: &DetectOptions,
) -> Result<Option<DetectionCore>, DetectError> {
    if header.is_empty() {
        return Ok(Some(DetectionCore::empty()));
    }

    // Fast path for non-container
    if let Some(core) = fast_path::probe_fast_path_slice_core(header, options)? {
        return Ok(Some(core));
    }

    // Signatures and rules
    match probe_signatures_core(header) {
        SignatureMatchCore::Confirmed(core) => {
            return Ok(Some(core));
        }
        SignatureMatchCore::Truncated(fmt) => {
            let cand = match fmt {
                std::borrow::Cow::Borrowed(s) => s,
                std::borrow::Cow::Owned(ref s) => {
                    match s.as_str() {
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
                        "sqlite" => "sqlite",
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
                    }
                }
            };
            return Ok(Some(DetectionCore::truncated(cand)));
        }
        SignatureMatchCore::None => (),
    }

    // Truncated ZIP
    if (header.len() >= 2 && header.starts_with(b"PK")) && header.len() < 4 {
        return Ok(Some(DetectionCore::truncated("zip")));
    }

    // Truncated CFB
    if header.len() >= 2 && header.starts_with(&[0xD0, 0xCF]) && header.len() < 8 {
        return Ok(Some(DetectionCore::truncated("cfb")));
    }

    // If matches full container signature, delegate to stream detector
    if (header.len() >= 4 && (header.starts_with(b"PK\x03\x04") || header.starts_with(b"PK\x05\x06")))
        || (header.len() >= 8 && header.starts_with(CFB_MAGIC))
        || (header.len() >= options.initial_window_size)
    {
        return Ok(None);
    }

    // Text formats
    if let Some(core) = text::probe_text_core(header) {
        return Ok(Some(core));
    }

    // Unknown format
    Ok(Some(DetectionCore::unknown()))
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
    if (header.len() >= 4 && (header.starts_with(b"PK\x03\x04") || header.starts_with(b"PK\x05\x06")))
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

pub fn detect_stream<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    options: &DetectOptions,
) -> Result<DetectionReport, DetectError> {
    // Read initial header window (stack-allocated buffer for default 4KB, avoiding heap allocations)
    let mut stack_buf = [0u8; 4096];
    let mut heap_buf;
    let header_buf: &mut [u8] = if options.initial_window_size <= 4096 {
        &mut stack_buf[..options.initial_window_size]
    } else {
        heap_buf = vec![0u8; options.initial_window_size];
        &mut heap_buf[..]
    };
    let mut total_read = 0;
    while total_read < header_buf.len() {
        match reader.read(&mut header_buf[total_read..]) {
            Ok(0) => break, // True EOF reached
            Ok(bytes) => total_read += bytes,
            Err(e) => {
                if reader.is_budget_exceeded() {
                    return Ok(DetectionReport {
                        schema_version: 1,
                        format_id: None,
                        category: "unknown".to_string(),
                        container_id: None,
                        identification: Identification::Unknown,
                        verification: Verification::None,
                        inspection: Inspection::Limited,
                        recommended_extension: None,
                        accepted_extensions: Vec::new(),
                        extension_status: ExtensionStatus::Undetermined,
                        evidence: Vec::new(),
                        warnings: vec![WarningItem::new(
                            "budget.source_read_exceeded",
                            "Total source read budget exceeded during detection",
                        )],
                        stats: Default::default(),
                        candidates: None,
                    });
                } else {
                    return Err(DetectError::Io(e));
                }
            }
        }
    }
    let header = &header_buf[..total_read];

    // Rewind back to start for full structural parsing
    if let Err(e) = reader.seek(SeekFrom::Start(0)) {
        if reader.is_budget_exceeded() {
            return Ok(DetectionReport {
                schema_version: 1,
                format_id: None,
                category: "unknown".to_string(),
                container_id: None,
                identification: Identification::Unknown,
                verification: Verification::None,
                inspection: Inspection::Limited,
                recommended_extension: None,
                accepted_extensions: Vec::new(),
                extension_status: ExtensionStatus::Undetermined,
                evidence: Vec::new(),
                warnings: vec![WarningItem::new(
                    "budget.source_read_exceeded",
                    "Total source read budget exceeded during detection",
                )],
                stats: Default::default(),
                candidates: None,
            });
        } else {
            return Err(DetectError::Io(e));
        }
    }

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
        SignatureMatch::Truncated(fmt) => {
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
                    format!("Input header matches truncated signature for {}", fmt),
                )],
                stats: Default::default(),
                candidates: Some(vec![fmt]),
            });
        }
        SignatureMatch::None => (),
    }

    // Step 2: Check ZIP signature (PK\x03\x04 or PK\x05\x06)
    if header.len() >= 4 && (header.starts_with(b"PK\x03\x04") || header.starts_with(b"PK\x05\x06")) {
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

    // Step 4: Check ISO 9660 volume descriptor at offset 32768 (only if stream size >= window)
    if header.len() >= options.initial_window_size {
        if let Some(rep) = disks::probe_iso_stream(reader) {
            return Ok(rep);
        }
    }

    // Step 5: Check structured and plain text formats (JSON, XML, HTML, YAML, TOML, CSV, TXT)
    if let Some(rep) = text::probe_text(header) {
        return Ok(rep);
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
