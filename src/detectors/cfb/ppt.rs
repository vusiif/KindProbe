use cfb::CompoundFile;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

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

    // Verify stream can be opened and starts with a valid MS-PPT RecordHeader
    let mut probe_buf = [0u8; 8];
    match comp.open_stream(stream_path) {
        Ok(mut s) => {
            if let Err(e) = s.read_exact(&mut probe_buf) {
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
            report
                .warnings
                .push(WarningItem::new("cfb.stream_open_error", e.to_string()));
            return Ok(false);
        }
    }

    let rec_ver_inst = u16::from_le_bytes([probe_buf[0], probe_buf[1]]);
    let rec_ver = rec_ver_inst & 0x0F;
    let rec_type = u16::from_le_bytes([probe_buf[2], probe_buf[3]]);
    let rec_len = u32::from_le_bytes([probe_buf[4], probe_buf[5], probe_buf[6], probe_buf[7]]);
    if !(0x03E8..=0x3000).contains(&rec_type) || (rec_len == 0 && rec_ver != 0x0F) {
        report.warnings.push(WarningItem::new(
            "cfb.invalid_powerpoint_stream",
            format!(
                "Invalid MS-PPT RecordHeader in PowerPoint Document stream (recType=0x{:04X}, recVer=0x{:X})",
                rec_type, rec_ver
            ),
        ));
        return Ok(false);
    }

    if has_current_user {
        if let Ok(mut cu_stream) = comp.open_stream(Path::new("/Current User")) {
            let mut cu_buf = [0u8; 16];
            if cu_stream.read_exact(&mut cu_buf).is_ok() {
                let cu_type = u16::from_le_bytes([cu_buf[2], cu_buf[3]]);
                let header_token =
                    u32::from_le_bytes([cu_buf[12], cu_buf[13], cu_buf[14], cu_buf[15]]);
                if cu_type != 0x0FF6 || (header_token != 0xE391_C05F && header_token != 0xF3D1_C4DF)
                {
                    report.warnings.push(WarningItem::new(
                        "cfb.invalid_powerpoint_stream",
                        "Invalid CurrentUserAtom header in Current User stream",
                    ));
                    return Ok(false);
                }
            }
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
    report
        .evidence
        .push(EvidenceItem::new("cfb.powerpoint_stream"));
    if has_current_user {
        report
            .evidence
            .push(EvidenceItem::new("cfb.current_user_stream"));
    }

    Ok(true)
}
