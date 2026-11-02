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
    let mut word_stream_path = None;
    let mut word_stream_len = 0u64;

    for entry in comp.walk() {
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
        if word_stream_path.is_none()
            && entry.name().eq_ignore_ascii_case("WordDocument")
            && entry.is_stream()
        {
            word_stream_path = Some(entry.path().to_path_buf());
            word_stream_len = entry.len();
        }
    }
    report.stats.entries_inspected = entries_count;

    let stream_path = match word_stream_path {
        Some(p) => p,
        None => {
            // Valid CFB container, but not a Word document
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

    if n_fib < FIB_NFIB_WORD97_MIN {
        report.warnings.push(WarningItem::new(
            "cfb.unsupported_word_version",
            format!("Unsupported pre-Word97 nFib version: 0x{:04X} (< 0x00C1)", n_fib),
        ));
        return Ok(report);
    }

    // Check encryption flag (bit 8 of flags at offset 10)
    let flags = u16::from_le_bytes([fib_header[10], fib_header[11]]);
    let is_encrypted = (flags & (1 << 8)) != 0;
    if is_encrypted {
        report.warnings.push(WarningItem::new(
            "cfb.document_encrypted",
            "Word document stream indicates password protection / encryption",
        ));
    }

    // Check fcMin and fcMac offsets if available (offsets 0x20 and 0x24)
    if bytes_read >= 40 {
        let fc_min = u32::from_le_bytes([fib_header[32], fib_header[33], fib_header[34], fib_header[35]]);
        let fc_mac = u32::from_le_bytes([fib_header[36], fib_header[37], fib_header[38], fib_header[39]]);
        if fc_min > fc_mac {
            report.warnings.push(WarningItem::new(
                "cfb.invalid_fib_offsets",
                format!("FibBase fcMin ({}) > fcMac ({})", fc_min, fc_mac),
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
