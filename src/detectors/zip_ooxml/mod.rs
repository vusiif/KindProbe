pub mod cd_check;
pub mod epub;
pub mod opc;
pub mod schemas;
pub mod xml;

use std::io::{Read, Seek, SeekFrom};
use zip::ZipArchive;

use crate::budget::{BudgetReader, DecompressLimitReader, DetectOptions};
use crate::model::{
    DetectError, DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
    Verification, WarningItem,
};

use cd_check::precheck_zip_central_directory;
use epub::check_epub_or_odf;
use opc::{parse_content_types, parse_package_relationships, resolve_target_path};
use schemas::classify_ooxml_content_type;
use xml::{verify_main_part_xml, MainPartXmlOutcome};

pub fn detect_zip_ooxml<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    options: &DetectOptions,
) -> Result<DetectionReport, DetectError> {
    let mut report = DetectionReport {
        schema_version: 1,
        format_id: None,
        category: "container".to_string(),
        container_id: Some("zip".to_string()),
        identification: Identification::ContainerOnly,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: None,
        accepted_extensions: Vec::new(),
        extension_status: ExtensionStatus::Undetermined,
        evidence: Vec::new(),
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    };

    // Pre-check ZIP End of Central Directory and Central Directory Headers
    if precheck_zip_central_directory(reader, options, &mut report)?.is_none() {
        return Ok(report);
    }

    // Rewind back to start before calling ZipArchive
    if let Err(e) = reader.seek(SeekFrom::Start(0)) {
        if reader.is_budget_exceeded() {
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "budget.source_read_exceeded",
                "Total source read budget exceeded during detection",
            ));
            return Ok(report);
        }
        return Err(DetectError::Io(e));
    }

    let budget_handle = reader.budget_exceeded_handle();
    let (window_read, window_exceeded, window_active) =
        reader.start_window(options.max_zip_directory_window);

    let mut archive = match ZipArchive::new(&mut *reader) {
        Ok(a) => {
            window_active.store(false, std::sync::atomic::Ordering::Relaxed);
            let bytes_read = window_read.load(std::sync::atomic::Ordering::Relaxed);
            if window_exceeded.load(std::sync::atomic::Ordering::Relaxed)
                || bytes_read > options.max_zip_directory_window
            {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.zip_directory_window_exceeded",
                    format!(
                        "ZIP central directory read window exceeded budget limit ({})",
                        options.max_zip_directory_window
                    ),
                ));
                return Ok(report);
            }
            a
        }
        Err(e) => {
            window_active.store(false, std::sync::atomic::Ordering::Relaxed);
            if window_exceeded.load(std::sync::atomic::Ordering::Relaxed) {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.zip_directory_window_exceeded",
                    format!(
                        "ZIP central directory read window exceeded budget limit ({})",
                        options.max_zip_directory_window
                    ),
                ));
                return Ok(report);
            }
            if budget_handle.load(std::sync::atomic::Ordering::Relaxed) {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.source_read_exceeded",
                    "Total source read budget exceeded during detection",
                ));
                return Ok(report);
            }
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "zip.truncated_or_corrupted",
                format!("Failed to parse ZIP archive: {}", e),
            ));
            return Ok(report);
        }
    };

    report.stats.entries_inspected = archive.len();
    report.evidence.push(EvidenceItem::new("zip.central_directory"));

    // Check for OOXML indicators
    let mut has_content_types = false;
    let mut has_rels = false;
    let mut has_ooxml_subdirs = false;

    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index_raw(i) {
            let name = entry.name();
            if name == "[Content_Types].xml" {
                has_content_types = true;
            } else if name == "_rels/.rels" {
                has_rels = true;
            } else if name.starts_with("word/")
                || name.starts_with("xl/")
                || name.starts_with("ppt/")
                || name.starts_with("customXml/")
                || name.starts_with("docProps/")
            {
                has_ooxml_subdirs = true;
            }
        }
    }

    // Pure ZIP confirmation: no OOXML characteristics
    if !has_content_types && !has_rels && !has_ooxml_subdirs {
        if check_epub_or_odf(&mut archive, &mut report) {
            return Ok(report);
        }

        report.identification = Identification::Format;
        report.format_id = Some("zip".to_string());
        report.category = "archive".to_string();
        report.container_id = Some("zip".to_string());
        report.recommended_extension = Some("zip".to_string());
        report.accepted_extensions = vec!["zip".to_string()];
        return Ok(report);
    }

    // If OOXML indicators exist but [Content_Types].xml is absent -> incomplete OOXML
    if !has_content_types {
        report.warnings.push(WarningItem::new(
            "ooxml.content_types_missing",
            "OOXML package structure incomplete: [Content_Types].xml missing",
        ));
        return Ok(report);
    }

    // Parse _rels/.rels to locate main officeDocument part
    let main_rel_target = {
        let rels_entry = match archive.by_name("_rels/.rels") {
            Ok(e) => e,
            Err(_) => {
                report.warnings.push(WarningItem::new(
                    "ooxml.required_relationship_missing",
                    "Cannot open _rels/.rels entry in package",
                ));
                return Ok(report);
            }
        };

        let remaining_total = options
            .max_total_decompressed_bytes
            .saturating_sub(report.stats.decompressed_bytes);
        let allowed = options.max_single_decompressed_bytes.min(remaining_total);
        if allowed == 0 {
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "budget.total_decompressed_exceeded",
                "Cumulative decompressed byte budget exceeded",
            ));
            return Ok(report);
        }

        let mut limit_reader = DecompressLimitReader::new(rels_entry, allowed);
        let mut xml_data = Vec::new();
        if let Err(e) = limit_reader.read_to_end(&mut xml_data) {
            report.stats.decompressed_bytes += limit_reader.bytes_read();
            if limit_reader.is_exceeded() {
                report.inspection = Inspection::Limited;
                if report.stats.decompressed_bytes >= options.max_total_decompressed_bytes {
                    report.warnings.push(WarningItem::new(
                        "budget.total_decompressed_exceeded",
                        "Cumulative decompressed byte budget exceeded",
                    ));
                } else {
                    report.warnings.push(WarningItem::new(
                        "budget.xml_output_exceeded",
                        "Decompressed size of _rels/.rels exceeded single XML limit",
                    ));
                }
            } else {
                report.warnings.push(WarningItem::new("ooxml.rels_read_error", e.to_string()));
            }
            return Ok(report);
        }
        report.stats.decompressed_bytes += limit_reader.bytes_read();

        match parse_package_relationships(&xml_data) {
            Ok(Some(target)) => target,
            Ok(None) => {
                report.warnings.push(WarningItem::new(
                    "ooxml.required_relationship_missing",
                    "No officeDocument core relationship found in _rels/.rels",
                ));
                return Ok(report);
            }
            Err(warn) => {
                report.warnings.push(warn);
                return Ok(report);
            }
        }
    };

    report.evidence.push(EvidenceItem::new("ooxml.package_relationships"));

    let normalized_target = match resolve_target_path("", &main_rel_target) {
        Some(t) => t,
        None => {
            report.warnings.push(WarningItem::new(
                "ooxml.invalid_target_path",
                format!("Relationship target path '{}' resolves outside package root", main_rel_target),
            ));
            return Ok(report);
        }
    };

    // Parse [Content_Types].xml to verify target part content type
    let content_type = {
        let ct_entry = match archive.by_name("[Content_Types].xml") {
            Ok(e) => e,
            Err(_) => {
                report.warnings.push(WarningItem::new(
                    "ooxml.content_types_missing",
                    "Cannot open [Content_Types].xml entry",
                ));
                return Ok(report);
            }
        };

        let remaining_total = options
            .max_total_decompressed_bytes
            .saturating_sub(report.stats.decompressed_bytes);
        let allowed = options.max_single_decompressed_bytes.min(remaining_total);
        if allowed == 0 {
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "budget.total_decompressed_exceeded",
                "Cumulative decompressed byte budget exceeded",
            ));
            return Ok(report);
        }

        let mut limit_reader = DecompressLimitReader::new(ct_entry, allowed);
        let mut xml_data = Vec::new();
        if let Err(e) = limit_reader.read_to_end(&mut xml_data) {
            report.stats.decompressed_bytes += limit_reader.bytes_read();
            if limit_reader.is_exceeded() {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.xml_output_exceeded",
                    "Decompressed size of [Content_Types].xml exceeded single XML limit",
                ));
            } else {
                report.warnings.push(WarningItem::new("ooxml.content_types_read_error", e.to_string()));
            }
            return Ok(report);
        }
        report.stats.decompressed_bytes += limit_reader.bytes_read();

        match parse_content_types(&xml_data, &normalized_target) {
            Ok(Some(ct)) => ct,
            Ok(None) => {
                report.warnings.push(WarningItem::new(
                    "ooxml.content_type_not_found",
                    format!("No ContentType entry found for part '/{}'", normalized_target),
                ));
                return Ok(report);
            }
            Err(warn) => {
                report.warnings.push(warn);
                return Ok(report);
            }
        }
    };

    report.evidence.push(EvidenceItem::new("ooxml.content_types"));

    // Classify supported OOXML family
    let format_kind = match classify_ooxml_content_type(&content_type) {
        Some(k) => k,
        None => {
            report.warnings.push(WarningItem::new(
                "ooxml.unsupported_content_type",
                format!("Package target has unsupported OOXML content type: {}", content_type),
            ));
            return Ok(report);
        }
    };

    // Decompress and parse main part XML, verifying root tag and depth budget
    let main_entry = match archive.by_name(&normalized_target) {
        Ok(e) => e,
        Err(_) => {
            report.warnings.push(WarningItem::new(
                "ooxml.main_part_missing",
                "Failed to open main document part in package",
            ));
            return Ok(report);
        }
    };

    let remaining_total = options
        .max_total_decompressed_bytes
        .saturating_sub(report.stats.decompressed_bytes);
    let allowed = options.max_single_decompressed_bytes.min(remaining_total);
    if allowed == 0 {
        report.inspection = Inspection::Limited;
        report.warnings.push(WarningItem::new(
            "budget.total_decompressed_exceeded",
            "Cumulative decompressed byte budget exceeded",
        ));
        return Ok(report);
    }

    let mut limit_reader = DecompressLimitReader::new(main_entry, allowed);
    let mut xml_data = Vec::new();
    if let Err(e) = limit_reader.read_to_end(&mut xml_data) {
        report.stats.decompressed_bytes += limit_reader.bytes_read();
        if limit_reader.is_exceeded() {
            report.inspection = Inspection::Limited;
            if report.stats.decompressed_bytes >= options.max_total_decompressed_bytes {
                report.warnings.push(WarningItem::new(
                    "budget.total_decompressed_exceeded",
                    "Cumulative decompressed byte budget exceeded",
                ));
            } else {
                report.warnings.push(WarningItem::new(
                    "budget.xml_output_exceeded",
                    "Decompressed size of main part exceeded single XML limit",
                ));
            }
        } else {
            report.warnings.push(WarningItem::new("ooxml.main_part_read_error", e.to_string()));
        }
        return Ok(report);
    }
    report.stats.decompressed_bytes += limit_reader.bytes_read();

    match verify_main_part_xml(&xml_data, format_kind, options.max_xml_depth) {
        Ok(MainPartXmlOutcome::Valid(kind)) => {
            report.identification = Identification::Format;
            report.format_id = Some(kind.format_id().to_string());
            report.category = "document".to_string();
            report.container_id = Some("zip".to_string());
            report.verification = Verification::Structure;
            report.inspection = Inspection::Complete;
            report.recommended_extension = Some(kind.recommended_extension().to_string());
            report.accepted_extensions = vec![kind.recommended_extension().to_string()];
            report.evidence.push(EvidenceItem::new(kind.evidence_code()));
        }
        Ok(MainPartXmlOutcome::DepthExceededValidRoot(kind)) => {
            report.identification = Identification::Format;
            report.format_id = Some(kind.format_id().to_string());
            report.category = "document".to_string();
            report.container_id = Some("zip".to_string());
            report.verification = Verification::Structure;
            report.inspection = Inspection::Limited;
            report.recommended_extension = Some(kind.recommended_extension().to_string());
            report.accepted_extensions = vec![kind.recommended_extension().to_string()];
            report.evidence.push(EvidenceItem::new(kind.evidence_code()));
            report.warnings.push(WarningItem::new(
                "budget.xml_depth_exceeded",
                format!(
                    "XML element depth ({}) exceeded allowed maximum ({})",
                    options.max_xml_depth + 1,
                    options.max_xml_depth
                ),
            ));
        }
        Ok(MainPartXmlOutcome::DepthExceededInvalidRoot) => {
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "ooxml.invalid_root_element",
                "Main part XML does not have valid root element for the detected content type",
            ));
            report.warnings.push(WarningItem::new(
                "budget.xml_depth_exceeded",
                format!(
                    "XML element depth ({}) exceeded allowed maximum ({})",
                    options.max_xml_depth + 1,
                    options.max_xml_depth
                ),
            ));
        }
        Ok(MainPartXmlOutcome::InvalidRoot(msg)) => {
            report.warnings.push(WarningItem::new("ooxml.invalid_root_element", msg));
        }
        Err(warn) => {
            report.warnings.push(warn);
        }
    }

    Ok(report)
}
