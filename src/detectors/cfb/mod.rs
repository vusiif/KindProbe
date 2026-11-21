pub mod doc;
pub mod ppt;
pub mod xls;

use std::io::{self, Read, Seek};
use std::path::PathBuf;
use cfb::CompoundFile;

use crate::budget::{BudgetReader, DetectOptions};
use crate::model::{
    DetectError, DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
    Verification, WarningItem,
};
pub use doc::verify_cfb_doc;
pub use ppt::verify_cfb_ppt;
pub use xls::verify_cfb_xls;

pub const CFB_MAGIC: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// Detects CFB compound file structure and specific formats (DOC, XLS, PPT).
pub fn detect_cfb<R: Read + Seek>(
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

    // Scan top-level entries under root storage
    let mut word_doc = None::<(PathBuf, u64)>;
    let mut xls_doc = None::<(PathBuf, u64)>;
    let mut ppt_doc = None::<(PathBuf, u64)>;
    let mut has_current_user = false;

    if let Ok(root_entries) = comp.read_storage("/") {
        for entry in root_entries {
            if !entry.is_stream() {
                continue;
            }
            let name = entry.name();
            if name.eq_ignore_ascii_case("WordDocument") {
                word_doc = Some((entry.path().to_path_buf(), entry.len()));
            } else if name.eq_ignore_ascii_case("Workbook") || name.eq_ignore_ascii_case("Book") {
                xls_doc = Some((entry.path().to_path_buf(), entry.len()));
            } else if name.eq_ignore_ascii_case("PowerPoint Document") {
                ppt_doc = Some((entry.path().to_path_buf(), entry.len()));
            } else if name.eq_ignore_ascii_case("Current User") {
                has_current_user = true;
            }
        }
    }

    // 1. Try Word Document verification if WordDocument stream exists
    if let Some((path, len)) = word_doc {
        if verify_cfb_doc(&mut comp, &path, len, &budget_handle, &mut report)? {
            return Ok(report);
        }
    }

    // 2. Try Excel Workbook verification if Workbook/Book stream exists
    if let Some((path, len)) = xls_doc {
        if verify_cfb_xls(&mut comp, &path, len, &budget_handle, &mut report)? {
            return Ok(report);
        }
    }

    // 3. Try PowerPoint verification if PowerPoint Document stream exists
    if let Some((path, len)) = ppt_doc {
        if verify_cfb_ppt(&mut comp, &path, len, has_current_user, &budget_handle, &mut report)? {
            return Ok(report);
        }
    }

    // If no specific office format confirmed, report remains ContainerOnly (CFB)
    Ok(report)
}

/// Backwards compatible alias for detect_cfb
pub use detect_cfb as detect_cfb_doc;
