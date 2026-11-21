use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use cfb::CompoundFile;

use crate::model::{
    DetectError, DetectionReport, EvidenceItem, Identification, Inspection, Verification,
    WarningItem,
};

pub const FIB_WIDENT_WORD97: u16 = 0xA5EC;
pub const FIB_NFIB_WORD97_MIN: u16 = 0x00C1;

/// Verifies MS-DOC WordDocument stream and corresponding 0Table/1Table stream.
pub fn verify_cfb_doc<R: Read + std::io::Seek>(
    comp: &mut CompoundFile<R>,
    stream_path: &Path,
    word_stream_len: u64,
    budget_handle: &AtomicBool,
    report: &mut DetectionReport,
) -> Result<bool, DetectError> {
    if word_stream_len < 32 {
        report.warnings.push(WarningItem::new(
            "cfb.invalid_word_fib",
            format!(
                "WordDocument stream size ({}) is smaller than FibBase header (32 bytes)",
                word_stream_len
            ),
        ));
        return Ok(false);
    }

    report.evidence.push(EvidenceItem::new("cfb.word_document_stream"));

    let mut stream = match comp.open_stream(stream_path) {
        Ok(s) => s,
        Err(e) => {
            if budget_handle.load(Ordering::Relaxed) {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.source_read_exceeded",
                    "Total source read budget exceeded during detection",
                ));
                return Ok(false);
            }
            report.warnings.push(WarningItem::new("cfb.stream_open_error", e.to_string()));
            return Ok(false);
        }
    };

    let mut fib_header = [0u8; 64];
    let bytes_read = match stream.read(&mut fib_header) {
        Ok(n) => n,
        Err(e) => {
            if budget_handle.load(Ordering::Relaxed) {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.source_read_exceeded",
                    "Total source read budget exceeded during detection",
                ));
                return Ok(false);
            }
            report.warnings.push(WarningItem::new("cfb.stream_read_error", e.to_string()));
            return Ok(false);
        }
    };

    if bytes_read < 32 {
        report.warnings.push(WarningItem::new(
            "cfb.invalid_word_fib",
            "Could not read complete 32-byte FibBase header",
        ));
        return Ok(false);
    }

    let w_ident = u16::from_le_bytes([fib_header[0], fib_header[1]]);
    let n_fib = u16::from_le_bytes([fib_header[2], fib_header[3]]);

    if w_ident != FIB_WIDENT_WORD97 {
        report.warnings.push(WarningItem::new(
            "cfb.invalid_word_fib",
            format!("Invalid FibBase wIdent: 0x{:04X} (expected 0xA5EC)", w_ident),
        ));
        return Ok(false);
    }

    if !(FIB_NFIB_WORD97_MIN..=0x0112).contains(&n_fib) {
        report.warnings.push(WarningItem::new(
            "cfb.unsupported_word_version",
            format!(
                "Unsupported nFib version: 0x{:04X} (expected Word 97..2007: 0x00C1..=0x0112)",
                n_fib
            ),
        ));
        return Ok(false);
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
                format!(
                    "Required table stream '{}' indicated by fWhichTblStm is missing",
                    required_table_name
                ),
            ));
            return Ok(false);
        }
    };

    // Verify table stream can be opened and read using its actual path
    match comp.open_stream(&actual_table_path) {
        Ok(mut t_stream) => {
            let mut probe_buf = [0u8; 1];
            if let Err(e) = t_stream.read(&mut probe_buf) {
                if budget_handle.load(Ordering::Relaxed) {
                    report.inspection = Inspection::Limited;
                    report.warnings.push(WarningItem::new(
                        "budget.source_read_exceeded",
                        "Total source read budget exceeded during detection",
                    ));
                    return Ok(false);
                }
                report.warnings.push(WarningItem::new(
                    "cfb.table_stream_read_error",
                    format!(
                        "Failed to read required table stream '{}': {}",
                        required_table_name, e
                    ),
                ));
                return Ok(false);
            }
        }
        Err(e) => {
            if budget_handle.load(Ordering::Relaxed) {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.source_read_exceeded",
                    "Total source read budget exceeded during detection",
                ));
                return Ok(false);
            }
            report.warnings.push(WarningItem::new(
                "cfb.table_stream_open_error",
                format!(
                    "Failed to open required table stream '{}': {}",
                    required_table_name, e
                ),
            ));
            return Ok(false);
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

    Ok(true)
}
