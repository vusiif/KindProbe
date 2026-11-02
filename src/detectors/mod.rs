pub mod cfb_doc;
pub mod signatures;
pub mod zip_ooxml;

use std::io::{Read, Seek, SeekFrom};
use crate::budget::{BudgetReader, DetectOptions};
use crate::model::{
    DetectError, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
    WarningItem,
};
use signatures::{make_pdf_report, make_png_report, probe_signatures, SignatureMatch};
use zip_ooxml::detect_zip_ooxml;
use cfb_doc::{detect_cfb_doc, CFB_MAGIC};

pub fn detect_stream<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    options: &DetectOptions,
) -> Result<DetectionReport, DetectError> {
    // Read initial header window
    let mut header = vec![0u8; options.initial_window_size];
    let n = match reader.read(&mut header) {
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
    header.truncate(n);

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

    // Step 1: Check PNG and PDF signatures
    match probe_signatures(&header) {
        SignatureMatch::Png => return Ok(make_png_report()),
        SignatureMatch::Pdf => return Ok(make_pdf_report()),
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
        return detect_cfb_doc(reader, options);
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

    // Step 4: Unknown format
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
