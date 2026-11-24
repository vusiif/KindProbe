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
    DetectError, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
    WarningItem,
};
use signatures::{probe_signatures, SignatureMatch};
use zip_ooxml::detect_zip_ooxml;
use cfb::{detect_cfb, CFB_MAGIC};
use fast_path::probe_fast_path;

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
    let n = match reader.read(header_buf) {
        Ok(bytes) => bytes,
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
    };
    let header = &header_buf[..n];

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
