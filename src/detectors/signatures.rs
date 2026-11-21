use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};
use super::archives::{probe_archive_signatures, ArchiveMatch};
use super::images::{probe_image_signatures, ImageMatch};

pub const PDF_SIGNATURE: &[u8] = b"%PDF-";

pub enum SignatureMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn probe_signatures(header: &[u8]) -> SignatureMatch {
    // 1. Probe image formats (PNG, JPEG, GIF)
    match probe_image_signatures(header) {
        ImageMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
        ImageMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
        ImageMatch::None => (),
    }

    // 2. Probe archive formats (7z, RAR)
    match probe_archive_signatures(header) {
        ArchiveMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
        ArchiveMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
        ArchiveMatch::None => (),
    }

    // 3. Probe PDF signature within first 1024 bytes
    let scan_limit = header.len().min(1024);
    if scan_limit >= 5 && header[..scan_limit].windows(5).any(|w| w == PDF_SIGNATURE) {
        return SignatureMatch::Confirmed(Box::new(make_pdf_report()));
    }

    if (header.len() >= 3 && header.starts_with(b"%PD"))
        || (!header.is_empty() && header[0] == b'%' && header.len() < 5)
    {
        return SignatureMatch::Truncated("pdf".to_string());
    }

    SignatureMatch::None
}

pub fn make_pdf_report() -> DetectionReport {
    DetectionReport {
        schema_version: 1,
        format_id: Some("pdf".to_string()),
        category: "document".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Signature,
        inspection: Inspection::Complete,
        recommended_extension: Some("pdf".to_string()),
        accepted_extensions: vec!["pdf".to_string()],
        extension_status: ExtensionStatus::Undetermined,
        evidence: vec![EvidenceItem::new("sig.pdf.header")],
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }
}
