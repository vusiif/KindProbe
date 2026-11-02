use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek, SeekFrom};
use quick_xml::events::Event;
use quick_xml::reader::Reader as XmlReader;
use zip::ZipArchive;

use crate::budget::{BudgetReader, DecompressLimitReader, DetectOptions};
use crate::model::{
    DetectError, DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
    Verification, WarningItem,
};

const TRANSITIONAL_REL_OFFICE_DOC: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
const STRICT_REL_OFFICE_DOC: &str =
    "http://purl.oclc.org/ooxml/officeDocument/relationships/officeDocument";

const WORD_MAIN_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml";
const WORD_TEMPLATE_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.template.main+xml";
const WORD_MACRO_CONTENT_TYPE: &str =
    "application/vnd.ms-word.document.macroEnabled.main+xml";

const WORD_XML_NS_TRANSITIONAL: &str =
    "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const WORD_XML_NS_STRICT: &str =
    "http://purl.oclc.org/ooxml/wordprocessingml/main";

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
    // to enforce max_zip_directory_window, max_entries, and detect duplicate members
    // before ZipArchive deduplicates them into its internal IndexMap.
    let file_len = match reader.seek(SeekFrom::End(0)) {
        Ok(l) => l,
        Err(e) => {
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
    };

    let tail_search_len = (65536 + 22).min(file_len as usize);
    let mut eocd_found = false;
    let mut cd_total_entries = 0usize;
    let mut cd_size = 0usize;
    let mut cd_offset = 0u64;

    if file_len >= 22 {
        let seek_pos = file_len - tail_search_len as u64;
        if let Ok(_) = reader.seek(SeekFrom::Start(seek_pos)) {
            let mut tail_buf = vec![0u8; tail_search_len];
            if let Ok(n) = reader.read(&mut tail_buf) {
                tail_buf.truncate(n);
                // Scan backwards for PK\x05\x06
                for i in (0..=tail_buf.len().saturating_sub(22)).rev() {
                    if &tail_buf[i..i + 4] == b"PK\x05\x06" {
                        let total_ent = u16::from_le_bytes([tail_buf[i + 10], tail_buf[i + 11]]) as usize;
                        let size = u32::from_le_bytes([
                            tail_buf[i + 12],
                            tail_buf[i + 13],
                            tail_buf[i + 14],
                            tail_buf[i + 15],
                        ]) as usize;
                        let offset = u32::from_le_bytes([
                            tail_buf[i + 16],
                            tail_buf[i + 17],
                            tail_buf[i + 18],
                            tail_buf[i + 19],
                        ]) as u64;
                        cd_total_entries = total_ent;
                        cd_size = size;
                        cd_offset = offset;
                        eocd_found = true;
                        break;
                    }
                }
            }
        }
    }

    if eocd_found {
        if cd_size > options.max_zip_directory_window {
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "budget.zip_directory_window_exceeded",
                format!(
                    "ZIP central directory size ({}) exceeded budget limit ({})",
                    cd_size, options.max_zip_directory_window
                ),
            ));
            return Ok(report);
        }

        if cd_total_entries > options.max_entries {
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "budget.entries_exceeded",
                format!(
                    "ZIP entry count ({}) exceeded budget limit ({})",
                    cd_total_entries, options.max_entries
                ),
            ));
            return Ok(report);
        }

        // Walk central directory headers to check for duplicate entry names or conflicting paths
        if cd_offset + (cd_size as u64) <= file_len {
            if let Ok(_) = reader.seek(SeekFrom::Start(cd_offset)) {
                let mut cd_bytes = vec![0u8; cd_size];
                if let Ok(n) = reader.read(&mut cd_bytes) {
                    cd_bytes.truncate(n);
                    let mut pos = 0;
                    let mut seen_raw_names = HashSet::new();
                    let mut seen_normalized_names = HashSet::new();
                    let mut duplicate_entry: Option<String> = None;
                    let mut conflicting_norm: Option<String> = None;

                    while pos + 46 <= cd_bytes.len() {
                        if &cd_bytes[pos..pos + 4] != b"PK\x01\x02" {
                            break;
                        }
                        let fn_len =
                            u16::from_le_bytes([cd_bytes[pos + 28], cd_bytes[pos + 29]]) as usize;
                        let extra_len =
                            u16::from_le_bytes([cd_bytes[pos + 30], cd_bytes[pos + 31]]) as usize;
                        let comment_len =
                            u16::from_le_bytes([cd_bytes[pos + 32], cd_bytes[pos + 33]]) as usize;
                        let fn_start = pos + 46;
                        let fn_end = fn_start + fn_len;
                        if fn_end > cd_bytes.len() {
                            break;
                        }
                        let raw_name =
                            String::from_utf8_lossy(&cd_bytes[fn_start..fn_end]).to_string();
                        if !seen_raw_names.insert(raw_name.clone()) && duplicate_entry.is_none() {
                            duplicate_entry = Some(raw_name.clone());
                        }
                        let norm = normalize_opc_path(&raw_name);
                        if !norm.is_empty()
                            && !seen_normalized_names.insert(norm.clone())
                            && conflicting_norm.is_none()
                            && duplicate_entry.is_none()
                        {
                            conflicting_norm = Some(norm);
                        }
                        pos = fn_end + extra_len + comment_len;
                    }

                    if let Some(dup) = duplicate_entry {
                        report.identification = Identification::ContainerOnly;
                        report.format_id = None;
                        report.recommended_extension = None;
                        report.inspection = Inspection::Ambiguous;
                        report.warnings.push(WarningItem::new(
                            "zip.conflicting_entries",
                            format!("Duplicate ZIP entry detected: '{}'", dup),
                        ));
                        return Ok(report);
                    }

                    if let Some(conf) = conflicting_norm {
                        report.identification = Identification::ContainerOnly;
                        report.format_id = None;
                        report.recommended_extension = None;
                        report.inspection = Inspection::Ambiguous;
                        report.warnings.push(WarningItem::new(
                            "zip.conflicting_entries",
                            format!("Conflicting normalized ZIP path detected: '{}'", conf),
                        ));
                        return Ok(report);
                    }
                }
            }
        }
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
            match e {
                zip::result::ZipError::Io(io_err) => {
                    if io_err.kind() == std::io::ErrorKind::UnexpectedEof {
                        report.inspection = Inspection::Limited;
                        report.warnings.push(WarningItem::new(
                            "zip.truncated_or_corrupted",
                            format!("Failed to parse ZIP central directory: {}", io_err),
                        ));
                        return Ok(report);
                    } else {
                        return Err(DetectError::Io(io_err));
                    }
                }
                _ => {
                    report.inspection = Inspection::Limited;
                    report.warnings.push(WarningItem::new(
                        "zip.truncated_or_corrupted",
                        format!("Failed to parse ZIP central directory: {}", e),
                    ));
                    return Ok(report);
                }
            }
        }
    };

    let entry_count = archive.len();
    report.stats.entries_inspected = entry_count;

    if entry_count > options.max_entries {
        report.inspection = Inspection::Limited;
        report.warnings.push(WarningItem::new(
            "budget.entries_exceeded",
            format!(
                "ZIP entry count ({}) exceeded budget limit ({})",
                entry_count, options.max_entries
            ),
        ));
        return Ok(report);
    }

    report.evidence.push(EvidenceItem::new("zip.central_directory"));

    // Collect all entry names and check for OOXML indicators
    let mut names: Vec<String> = Vec::with_capacity(entry_count);
    let mut has_content_types = false;
    let mut has_rels = false;
    let mut has_ooxml_subdirs = false;

    for i in 0..entry_count {
        if let Ok(entry) = archive.by_index(i) {
            let name = entry.name().to_string();
            if name == "[Content_Types].xml" {
                has_content_types = true;
            } else if name == "_rels/.rels" {
                has_rels = true;
            } else if name.starts_with("word/")
                || name.starts_with("xl/")
                || name.starts_with("ppt/")
                || name.starts_with("docProps/")
            {
                has_ooxml_subdirs = true;
            }
            names.push(name);
        }
    }

    // Check if this is a plain ZIP (no OOXML indicators at all)
    if !has_content_types && !has_rels && !has_ooxml_subdirs {
        report.identification = Identification::Format;
        report.format_id = Some("zip".to_string());
        report.category = "archive".to_string();
        report.recommended_extension = Some("zip".to_string());
        report.accepted_extensions = vec!["zip".to_string()];
        return Ok(report);
    }

    // If OOXML indicators exist (like word/), but [Content_Types].xml is absent,
    // this is a fake or broken OOXML. Must NOT be confirmed as DOCX, and must NOT
    // silently fall back to confirmed plain ZIP!
    if !has_content_types {
        report.warnings.push(WarningItem::new(
            "ooxml.content_types_missing",
            "OOXML package structure incomplete: [Content_Types].xml missing",
        ));
        return Ok(report);
    }

    if !has_rels {
        report.warnings.push(WarningItem::new(
            "ooxml.required_relationship_missing",
            "Required OOXML package relationship is missing: _rels/.rels absent",
        ));
        return Ok(report);
    }

    // Step 2: Parse _rels/.rels to locate main office document relationship
    let target_path = {
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
            Ok(PackageRelOutcome::Target(target)) => target,
            Ok(PackageRelOutcome::Conflict) => {
                report.identification = Identification::ContainerOnly;
                report.format_id = None;
                report.recommended_extension = None;
                report.inspection = Inspection::Ambiguous;
                report.warnings.push(WarningItem::new(
                    "zip.conflicting_entries",
                    "Conflicting officeDocument relationships found in _rels/.rels",
                ));
                return Ok(report);
            }
            Ok(PackageRelOutcome::None) => {
                report.warnings.push(WarningItem::new(
                    "ooxml.required_relationship_missing",
                    "No officeDocument relationship found in _rels/.rels",
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

    // Normalize target path relative to package root
    let normalized_target = normalize_opc_path(&target_path);
    if normalized_target.is_empty() {
        report.warnings.push(WarningItem::new(
            "ooxml.invalid_target_path",
            format!("Relationship target path '{}' resolves outside root", target_path),
        ));
        return Ok(report);
    }

    // Verify target part exists in the package
    if !names.iter().any(|n| n == &normalized_target) {
        report.warnings.push(WarningItem::new(
            "ooxml.main_part_missing",
            format!("Relationship target part '{}' not found in package", normalized_target),
        ));
        return Ok(report);
    }

    // Step 3: Parse [Content_Types].xml to verify target part content type
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
                if report.stats.decompressed_bytes >= options.max_total_decompressed_bytes {
                    report.warnings.push(WarningItem::new(
                        "budget.total_decompressed_exceeded",
                        "Cumulative decompressed byte budget exceeded",
                    ));
                } else {
                    report.warnings.push(WarningItem::new(
                        "budget.xml_output_exceeded",
                        "Decompressed size of [Content_Types].xml exceeded single XML limit",
                    ));
                }
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

    // Check if ContentType is a Word processing document
    let is_word_ct = content_type == WORD_MAIN_CONTENT_TYPE
        || content_type == WORD_TEMPLATE_CONTENT_TYPE
        || content_type == WORD_MACRO_CONTENT_TYPE;

    if !is_word_ct {
        report.warnings.push(WarningItem::new(
            "ooxml.non_word_package",
            format!("Package target has non-Word content type: {}", content_type),
        ));
        return Ok(report);
    }

    // Step 4: Decompress and parse main part XML, verifying root tag and checking depth budget
    {
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

        match verify_main_document_xml(&xml_data, options.max_xml_depth) {
            Ok(true) => {
                // Fully confirmed DOCX!
                report.identification = Identification::Format;
                report.format_id = Some("docx".to_string());
                report.category = "document".to_string();
                report.container_id = Some("zip".to_string());
                report.verification = Verification::Structure;
                report.inspection = Inspection::Complete;
                report.recommended_extension = Some("docx".to_string());
                report.accepted_extensions = vec!["docx".to_string()];
                report.evidence.push(EvidenceItem::new("ooxml.word_main_part"));
            }
            Ok(false) => {
                report.warnings.push(WarningItem::new(
                    "ooxml.invalid_root_element",
                    "Main document part XML does not have valid <w:document> root element",
                ));
            }
            Err(warn) => {
                if warn.code == "budget.xml_depth_exceeded" {
                    // Identification is confirmed DOCX, but depth budget was triggered
                    report.identification = Identification::Format;
                    report.format_id = Some("docx".to_string());
                    report.category = "document".to_string();
                    report.container_id = Some("zip".to_string());
                    report.verification = Verification::Structure;
                    report.inspection = Inspection::Limited;
                    report.recommended_extension = Some("docx".to_string());
                    report.accepted_extensions = vec!["docx".to_string()];
                    report.evidence.push(EvidenceItem::new("ooxml.word_main_part"));
                    report.warnings.push(warn);
                } else {
                    report.warnings.push(warn);
                }
            }
        }
    }

    Ok(report)
}

/// Normalizes an OPC relative or absolute path relative to package root.
/// Returns normalized path without leading slash, e.g. "word/document.xml".
pub fn normalize_opc_path(raw_path: &str) -> String {
    let clean = raw_path.trim_start_matches('/');
    let mut parts: Vec<&str> = Vec::new();
    for seg in clean.split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            if parts.is_empty() {
                return String::new(); // Escaped root!
            }
            parts.pop();
        } else {
            parts.push(seg);
        }
    }
    parts.join("/")
}

#[derive(Debug, PartialEq, Eq)]
enum PackageRelOutcome {
    None,
    Target(String),
    Conflict,
}

fn parse_package_relationships(xml_bytes: &[u8]) -> Result<PackageRelOutcome, WarningItem> {
    let mut reader = XmlReader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut found_target: Option<String> = None;
    let mut conflict = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                if e.name().as_ref() == b"Relationship" {
                    let mut rel_type = String::new();
                    let mut rel_target = String::new();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"Type" {
                            rel_type = String::from_utf8_lossy(&attr.value).to_string();
                        } else if attr.key.as_ref() == b"Target" {
                            rel_target = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                    if rel_type == TRANSITIONAL_REL_OFFICE_DOC || rel_type == STRICT_REL_OFFICE_DOC {
                        let norm = normalize_opc_path(&rel_target);
                        if let Some(existing) = &found_target {
                            if existing != &norm {
                                conflict = true;
                            }
                        } else {
                            found_target = Some(norm);
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(WarningItem::new(
                    "ooxml.rels_xml_error",
                    format!("XML error parsing _rels/.rels: {}", e),
                ));
            }
            _ => (),
        }
        buf.clear();
    }

    if conflict {
        Ok(PackageRelOutcome::Conflict)
    } else if let Some(target) = found_target {
        Ok(PackageRelOutcome::Target(target))
    } else {
        Ok(PackageRelOutcome::None)
    }
}

fn parse_content_types(
    xml_bytes: &[u8],
    normalized_target: &str,
) -> Result<Option<String>, WarningItem> {
    let mut reader = XmlReader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);

    let target_part_name = format!("/{}", normalized_target);
    let target_ext = normalized_target.rsplit('.').next().unwrap_or("");

    let mut overrides = HashMap::new();
    let mut defaults = HashMap::new();

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"Override" {
                    let mut part_name = String::new();
                    let mut content_type = String::new();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"PartName" {
                            part_name = String::from_utf8_lossy(&attr.value).to_string();
                        } else if attr.key.as_ref() == b"ContentType" {
                            content_type = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                    if !part_name.is_empty() && !content_type.is_empty() {
                        overrides.insert(part_name, content_type);
                    }
                } else if name.as_ref() == b"Default" {
                    let mut ext = String::new();
                    let mut content_type = String::new();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"Extension" {
                            ext = String::from_utf8_lossy(&attr.value).to_string();
                        } else if attr.key.as_ref() == b"ContentType" {
                            content_type = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                    if !ext.is_empty() && !content_type.is_empty() {
                        defaults.insert(ext, content_type);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(WarningItem::new(
                    "ooxml.content_types_xml_error",
                    format!("XML error parsing [Content_Types].xml: {}", e),
                ));
            }
            _ => (),
        }
        buf.clear();
    }

    if let Some(ct) = overrides.get(&target_part_name) {
        return Ok(Some(ct.clone()));
    }
    if let Some(ct) = defaults.get(target_ext) {
        return Ok(Some(ct.clone()));
    }

    Ok(None)
}

fn verify_main_document_xml(
    xml_bytes: &[u8],
    max_depth: usize,
) -> Result<bool, WarningItem> {
    let mut reader = XmlReader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);

    let mut current_depth: usize = 0;
    let mut found_root = false;
    let mut valid_root = false;

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                current_depth += 1;
                if current_depth > max_depth {
                    return Err(WarningItem::new(
                        "budget.xml_depth_exceeded",
                        format!(
                            "XML element depth ({}) exceeded allowed maximum ({})",
                            current_depth, max_depth
                        ),
                    ));
                }

                if !found_root {
                    found_root = true;
                    let local_name = e.local_name();
                    if local_name.as_ref() == b"document" {
                        // Check attributes for Word XML namespace
                        let mut has_ns = false;
                        for attr in e.attributes().flatten() {
                            let val = String::from_utf8_lossy(&attr.value);
                            if val == WORD_XML_NS_TRANSITIONAL || val == WORD_XML_NS_STRICT {
                                has_ns = true;
                                break;
                            }
                        }
                        // Also accept if prefix is w:
                        if has_ns || e.name().as_ref().starts_with(b"w:") {
                            valid_root = true;
                        }
                    }
                }
            }
            Ok(Event::End(_)) => {
                if current_depth > 0 {
                    current_depth -= 1;
                }
            }
            Ok(Event::Empty(ref e)) => {
                if !found_root {
                    found_root = true;
                    if e.local_name().as_ref() == b"document" {
                        valid_root = true;
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(WarningItem::new(
                    "ooxml.main_part_xml_error",
                    format!("XML error parsing main document part: {}", e),
                ));
            }
            _ => (),
        }
        buf.clear();
    }

    Ok(valid_root)
}
