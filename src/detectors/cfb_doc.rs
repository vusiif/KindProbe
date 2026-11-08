use std::io::{self, Read, Seek};
use cfb::CompoundFile;

use crate::budget::{BudgetReader, DetectOptions};
use crate::model::{
    DetectError, DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
    Verification, WarningItem,
};

pub const CFB_MAGIC: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// FibBase wIdent constant according to [MS-DOC] Section 2.5.1
pub const FIB_WIDENT_WORD97: u16 = 0xA5EC;
/// Minimum nFib version for Word 97 (0x00C1 = 193)
pub const FIB_NFIB_WORD97_MIN: u16 = 0x00C1;

pub fn detect_cfb_doc<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    options: &DetectOptions,
) -> Result<DetectionReport, DetectError> {
    let mut report = DetectionReport {
        schema_version: 1,
        format_id: None,
        category: "container".to_string(),
        container_id: Some("cfb".to_string()),
        identification: Identification::ContainerOnly,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: None,
        accepted_extensions: Vec::new(),
        extension_status: ExtensionStatus::Undetermined,
        evidence: vec![EvidenceItem::new("cfb.header")],
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    };

    let budget_handle = reader.budget_exceeded_handle();
    let mut comp = match CompoundFile::open(&mut *reader) {
        Ok(c) => c,
        Err(e) => {
            if budget_handle.load(std::sync::atomic::Ordering::Relaxed) {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.source_read_exceeded",
                    "Total source read budget exceeded during detection",
                ));
                return Ok(report);
            }
            if e.kind() == io::ErrorKind::InvalidData || e.kind() == io::ErrorKind::UnexpectedEof {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "cfb.corrupted_or_invalid_chain",
                    format!("Failed to open compound document: {}", e),
                ));
                return Ok(report);
            }
            return Err(DetectError::Io(e));
        }
    };

    report.evidence.push(EvidenceItem::new("cfb.directory"));

    // Check CFB entries budget and count inspected entries
    let mut entries_count = 0;
    for _ in comp.walk() {
        entries_count += 1;
        if entries_count > options.max_entries {
            report.stats.entries_inspected = entries_count;
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "budget.entries_exceeded",
                format!(
                    "CFB entry count ({}) exceeded budget limit ({})",
                    entries_count, options.max_entries
                ),
            ));
            return Ok(report);
        }
    }
    report.stats.entries_inspected = entries_count;

    // WordDocument must be a direct top-level stream under the root storage
    let mut word_stream_path = None;
    let mut word_stream_len = 0u64;

    if let Ok(root_entries) = comp.read_storage("/") {
        for entry in root_entries {
            if entry.name().eq_ignore_ascii_case("WordDocument") && entry.is_stream() {
                word_stream_path = Some(entry.path().to_path_buf());
                word_stream_len = entry.len();
                break;
            }
        }
    }

    let stream_path = match word_stream_path {
        Some(p) => p,
        None => {
            // Valid CFB container, but not a Word document at root
            return Ok(report);
        }
    };

    if word_stream_len < 32 {
        report.warnings.push(WarningItem::new(
            "cfb.invalid_word_fib",
            format!(
                "WordDocument stream size ({}) is smaller than FibBase header (32 bytes)",
                word_stream_len
            ),
        ));
        return Ok(report);
    }

    report.evidence.push(EvidenceItem::new("cfb.word_document_stream"));

    // Open and read the FIB header
    let mut stream = match comp.open_stream(&stream_path) {
        Ok(s) => s,
        Err(e) => {
            if budget_handle.load(std::sync::atomic::Ordering::Relaxed) {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.source_read_exceeded",
                    "Total source read budget exceeded during detection",
                ));
                return Ok(report);
            }
            if e.kind() == io::ErrorKind::InvalidData || e.kind() == io::ErrorKind::UnexpectedEof {
                report.warnings.push(WarningItem::new(
                    "cfb.stream_open_error",
                    format!("Failed to open WordDocument stream: {}", e),
                ));
                return Ok(report);
            }
            return Err(DetectError::Io(e));
        }
    };

    let mut fib_header = [0u8; 64];
    let bytes_read = match stream.read(&mut fib_header) {
        Ok(n) => n,
        Err(e) => {
            if budget_handle.load(std::sync::atomic::Ordering::Relaxed) {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.source_read_exceeded",
                    "Total source read budget exceeded during detection",
                ));
                return Ok(report);
            }
            if e.kind() == io::ErrorKind::InvalidData || e.kind() == io::ErrorKind::UnexpectedEof {
                report.warnings.push(WarningItem::new(
                    "cfb.stream_read_error",
                    format!("Failed to read WordDocument FIB header: {}", e),
                ));
                return Ok(report);
            }
            return Err(DetectError::Io(e));
        }
    };

    if bytes_read < 32 {
        report.warnings.push(WarningItem::new(
            "cfb.invalid_word_fib",
            "Could not read complete 32-byte FibBase header",
        ));
        return Ok(report);
    }

    let w_ident = u16::from_le_bytes([fib_header[0], fib_header[1]]);
    let n_fib = u16::from_le_bytes([fib_header[2], fib_header[3]]);

    if w_ident != FIB_WIDENT_WORD97 {
        report.warnings.push(WarningItem::new(
            "cfb.invalid_word_fib",
            format!("Invalid FibBase wIdent: 0x{:04X} (expected 0xA5EC)", w_ident),
        ));
        return Ok(report);
    }

    if n_fib < FIB_NFIB_WORD97_MIN || n_fib > 0x0112 {
        report.warnings.push(WarningItem::new(
            "cfb.unsupported_word_version",
            format!("Unsupported nFib version: 0x{:04X} (expected Word 97..2007: 0x00C1..=0x0112)", n_fib),
        ));
        return Ok(report);
    }

    // Check flags at offset 10..12
    let flags = u16::from_le_bytes([fib_header[10], fib_header[11]]);
    let is_encrypted = (flags & (1 << 8)) != 0;
    if is_encrypted {
        report.warnings.push(WarningItem::new(
            "cfb.document_encrypted",
            "Word document stream indicates password protection / encryption",
        ));
    }

    // Verify required table stream indicated by fWhichTblStm (bit 9)
    let is_1table = (flags & 0x0200) != 0;
    let required_table_name = if is_1table { "1Table" } else { "0Table" };

    let mut table_stream_path = None;
    if let Ok(root_entries) = comp.read_storage("/") {
        for entry in root_entries {
            if entry.name().eq_ignore_ascii_case(required_table_name) && entry.is_stream() {
                table_stream_path = Some(entry.path().to_path_buf());
                break;
            }
        }
    }

    let actual_table_path = match table_stream_path {
        Some(p) => p,
        None => {
            report.warnings.push(WarningItem::new(
                "cfb.table_stream_missing",
                format!("Required table stream '{}' indicated by fWhichTblStm is missing", required_table_name),
            ));
            return Ok(report);
        }
    };

    // Verify table stream can be opened and read using its actual path
    match comp.open_stream(&actual_table_path) {
        Ok(mut t_stream) => {
            let mut probe_buf = [0u8; 1];
            if let Err(e) = t_stream.read(&mut probe_buf) {
                if budget_handle.load(std::sync::atomic::Ordering::Relaxed) {
                    report.inspection = Inspection::Limited;
                    report.warnings.push(WarningItem::new(
                        "budget.source_read_exceeded",
                        "Total source read budget exceeded during detection",
                    ));
                    return Ok(report);
                }
                report.warnings.push(WarningItem::new(
                    "cfb.table_stream_read_error",
                    format!("Failed to read required table stream '{}': {}", required_table_name, e),
                ));
                return Ok(report);
            }
        }
        Err(e) => {
            if budget_handle.load(std::sync::atomic::Ordering::Relaxed) {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.source_read_exceeded",
                    "Total source read budget exceeded during detection",
                ));
                return Ok(report);
            }
            report.warnings.push(WarningItem::new(
                "cfb.table_stream_open_error",
                format!("Failed to open required table stream '{}': {}", required_table_name, e),
            ));
            return Ok(report);
        }
    }

    // Confirmed Word 97+ DOC!
    report.identification = Identification::Format;
    report.format_id = Some("doc".to_string());
    report.category = "document".to_string();
    report.container_id = Some("cfb".to_string());
    report.verification = Verification::Structure;
    report.inspection = Inspection::Complete;
    report.recommended_extension = Some("doc".to_string());
    report.accepted_extensions = vec!["doc".to_string(), "dot".to_string()];
    report.evidence.push(EvidenceItem::new("cfb.fib_word97"));

    Ok(report)
}
