use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
pub const PDF_SIGNATURE: &[u8] = b"%PDF-";

pub enum SignatureMatch {
    PngStructure,
    PngSignatureOnly,
    Pdf,
    Truncated(String),
    None,
}

pub fn probe_signatures(header: &[u8]) -> SignatureMatch {
    if header.len() >= 8 && header.starts_with(PNG_SIGNATURE) {
        // Validate IHDR chunk (13 bytes data + 4 bytes 'IHDR')
        if header.len() >= 16 {
            let chunk_len = u32::from_be_bytes([header[8], header[9], header[10], header[11]]);
            let chunk_type = &header[12..16];
            if chunk_len == 13 && chunk_type == b"IHDR" {
                return SignatureMatch::PngStructure;
            }
        }
        return SignatureMatch::PngSignatureOnly;
    } else if (header.len() >= 4 && header.starts_with(b"\x89PNG"))
        || (header.len() >= 1 && header[0] == 0x89 && header.len() < 8)
    {
        return SignatureMatch::Truncated("png".to_string());
    }

    // PDF signature check within first 1024 bytes
    let scan_limit = header.len().min(1024);
    if scan_limit >= 5 {
        if let Some(pos) = header[..scan_limit].windows(5).position(|w| w == PDF_SIGNATURE) {
            // Check if followed by version digits e.g. "1." or "2."
            if pos + 7 <= header.len() {
                let ver = &header[pos + 5..pos + 7];
                if ver[0].is_ascii_digit() && ver[1] == b'.' {
                    return SignatureMatch::Pdf;
                }
            }
            return SignatureMatch::Pdf;
        }
    }

    if (header.len() >= 3 && header.starts_with(b"%PD"))
        || (header.len() >= 1 && header[0] == b'%' && header.len() < 5)
    {
        return SignatureMatch::Truncated("pdf".to_string());
    }

    SignatureMatch::None
}

pub fn make_png_report(has_ihdr: bool) -> DetectionReport {
    DetectionReport {
        schema_version: 1,
        format_id: Some("png".to_string()),
        category: "image".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification: if has_ihdr {
            Verification::Structure
        } else {
            Verification::Signature
        },
        inspection: Inspection::Complete,
        recommended_extension: Some("png".to_string()),
        accepted_extensions: vec!["png".to_string()],
        extension_status: ExtensionStatus::Undetermined,
        evidence: if has_ihdr {
            vec![
                EvidenceItem::new("sig.png.header"),
                EvidenceItem::new("sig.png.ihdr"),
            ]
        } else {
            vec![EvidenceItem::new("sig.png.header")]
        },
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }
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
