use std::io::Read;
use zip::ZipArchive;

use crate::model::{
    DetectionReport, EvidenceItem, Identification, Inspection, Verification,
};

/// Inspects ZIP archive for EPUB (OCF) or OpenDocument (ODF) mimetype specification.
pub fn check_epub_or_odf<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    report: &mut DetectionReport,
) -> bool {
    let mut mimetype_content = None;

    if let Ok(mut entry) = archive.by_name("mimetype") {
        let mut buf = [0u8; 80];
        if let Ok(n) = entry.read(&mut buf) {
            if let Ok(s) = std::str::from_utf8(&buf[..n]) {
                mimetype_content = Some(s.trim().to_ascii_lowercase());
            }
        }
    }

    let mime = match mimetype_content {
        Some(m) => m,
        None => return false,
    };

    if mime == "application/epub+zip" {
        let has_container = archive.by_name("META-INF/container.xml").is_ok();
        report.identification = Identification::Format;
        report.format_id = Some("epub".to_string());
        report.category = "document".to_string();
        report.container_id = Some("zip".to_string());
        report.verification = Verification::Structure;
        report.inspection = Inspection::Complete;
        report.recommended_extension = Some("epub".to_string());
        report.accepted_extensions = vec!["epub".to_string()];
        report.evidence.push(EvidenceItem::new("epub.mimetype"));
        if has_container {
            report.evidence.push(EvidenceItem::new("epub.container"));
        }
        return true;
    }

    if mime == "application/vnd.oasis.opendocument.text" {
        report.identification = Identification::Format;
        report.format_id = Some("odt".to_string());
        report.category = "document".to_string();
        report.container_id = Some("zip".to_string());
        report.verification = Verification::Structure;
        report.inspection = Inspection::Complete;
        report.recommended_extension = Some("odt".to_string());
        report.accepted_extensions = vec!["odt".to_string(), "ott".to_string()];
        report.evidence.push(EvidenceItem::new("odf.mimetype"));
        return true;
    }

    if mime == "application/vnd.oasis.opendocument.spreadsheet" {
        report.identification = Identification::Format;
        report.format_id = Some("ods".to_string());
        report.category = "document".to_string();
        report.container_id = Some("zip".to_string());
        report.verification = Verification::Structure;
        report.inspection = Inspection::Complete;
        report.recommended_extension = Some("ods".to_string());
        report.accepted_extensions = vec!["ods".to_string(), "ots".to_string()];
        report.evidence.push(EvidenceItem::new("odf.mimetype"));
        return true;
    }

    if mime == "application/vnd.oasis.opendocument.presentation" {
        report.identification = Identification::Format;
        report.format_id = Some("odp".to_string());
        report.category = "document".to_string();
        report.container_id = Some("zip".to_string());
        report.verification = Verification::Structure;
        report.inspection = Inspection::Complete;
        report.recommended_extension = Some("odp".to_string());
        report.accepted_extensions = vec!["odp".to_string(), "otp".to_string()];
        report.evidence.push(EvidenceItem::new("odf.mimetype"));
        return true;
    }

    false
}
