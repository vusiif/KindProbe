use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use cfb::CompoundFile;

use crate::model::{
    DetectError, DetectionReport, EvidenceItem, Identification, Inspection, Verification,
    WarningItem,
};

/// Verifies Microsoft PowerPoint 97-2003 structure in PowerPoint Document stream.
pub fn verify_cfb_ppt<R: Read + std::io::Seek>(
    comp: &mut CompoundFile<R>,
    stream_path: &Path,
    stream_len: u64,
    has_current_user: bool,
    budget_handle: &AtomicBool,
    report: &mut DetectionReport,
) -> Result<bool, DetectError> {
    if stream_len < 32 {
        report.warnings.push(WarningItem::new(
            "cfb.invalid_powerpoint_stream",
            format!(
                "PowerPoint Document stream size ({}) is smaller than minimum header (32 bytes)",
                stream_len
            ),
        ));
        return Ok(false);
    }

    // Verify stream can be opened and read
    let mut probe_buf = [0u8; 8];
    match comp.open_stream(stream_path) {
        Ok(mut s) => {
            if let Err(e) = s.read(&mut probe_buf) {
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
            report.warnings.push(WarningItem::new("cfb.stream_open_error", e.to_string()));
            return Ok(false);
        }
    }

    // Fully confirmed PPT!
    report.identification = Identification::Format;
    report.format_id = Some("ppt".to_string());
    report.category = "document".to_string();
    report.container_id = Some("cfb".to_string());
    report.verification = Verification::Structure;
    report.inspection = Inspection::Complete;
    report.recommended_extension = Some("ppt".to_string());
    report.accepted_extensions = vec!["ppt".to_string(), "pot".to_string(), "pps".to_string()];
    report.evidence.push(EvidenceItem::new("cfb.powerpoint_stream"));
    if has_current_user {
        report.evidence.push(EvidenceItem::new("cfb.current_user_stream"));
    }

    Ok(true)
}
