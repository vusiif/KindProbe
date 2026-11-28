use cfb::CompoundFile;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::model::{
    DetectError, DetectionReport, EvidenceItem, Identification, Inspection, Verification,
    WarningItem,
};

/// Verifies Microsoft Excel 97-2003 BIFF8 / BIFF5 structure in Workbook/Book stream.
pub fn verify_cfb_xls<R: Read + std::io::Seek>(
    comp: &mut CompoundFile<R>,
    stream_path: &Path,
    stream_len: u64,
    budget_handle: &AtomicBool,
    report: &mut DetectionReport,
) -> Result<bool, DetectError> {
    if stream_len < 8 {
        report.warnings.push(WarningItem::new(
            "cfb.invalid_workbook_stream",
            format!(
                "Workbook stream size ({}) is smaller than BOF record (8 bytes)",
                stream_len
            ),
        ));
        return Ok(false);
    }

    let mut bof = [0u8; 16];
    let n = match comp.open_stream(stream_path) {
        Ok(mut s) => match s.read(&mut bof) {
            Ok(bytes) => bytes,
            Err(e) => {
                if budget_handle.load(Ordering::Relaxed) {
                    report.inspection = Inspection::Limited;
                    report.warnings.push(WarningItem::new(
                        "budget.source_read_exceeded",
                        "Total source read budget exceeded during detection",
                    ));
                    return Ok(false);
                }
                report
                    .warnings
                    .push(WarningItem::new("cfb.stream_read_error", e.to_string()));
                return Ok(false);
            }
        },
        Err(e) => {
            if budget_handle.load(Ordering::Relaxed) {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.source_read_exceeded",
                    "Total source read budget exceeded during detection",
                ));
                return Ok(false);
            }
            report
                .warnings
                .push(WarningItem::new("cfb.stream_open_error", e.to_string()));
            return Ok(false);
        }
    };

    if n < 8 {
        return Ok(false);
    }

    let record_type = u16::from_le_bytes([bof[0], bof[1]]);
    if record_type != 0x0809 {
        report.warnings.push(WarningItem::new(
            "cfb.invalid_biff_bof",
            format!(
                "Invalid BIFF BOF record type: expected 0x0809, found 0x{:04X}",
                record_type
            ),
        ));
        return Ok(false);
    }

    let version = u16::from_le_bytes([bof[4], bof[5]]);
    // BIFF8 = 0x0600 (Excel 97-2003), BIFF5 = 0x0500 (Excel 5.0/95)
    let is_valid_biff_version = version == 0x0600 || version == 0x0500;
    if !is_valid_biff_version {
        report.warnings.push(WarningItem::new(
            "cfb.unsupported_biff_version",
            format!(
                "Unsupported BIFF version in Excel stream: 0x{:04X}",
                version
            ),
        ));
        return Ok(false);
    }

    // Fully confirmed XLS!
    report.identification = Identification::Format;
    report.format_id = Some("xls".to_string());
    report.category = "document".to_string();
    report.container_id = Some("cfb".to_string());
    report.verification = Verification::Structure;
    report.inspection = Inspection::Complete;
    report.recommended_extension = Some("xls".to_string());
    report.accepted_extensions = vec!["xls".to_string(), "xlt".to_string()];
    report
        .evidence
        .push(EvidenceItem::new("cfb.workbook_stream"));
    report.evidence.push(EvidenceItem::new("cfb.biff_record"));

    Ok(true)
}
