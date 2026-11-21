use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

/// Probes whether the text represents an XML or HTML document.
pub fn probe_xml_or_html(text: &str) -> Option<DetectionReport> {
    let trimmed = text.trim_start();
    if !trimmed.starts_with('<') {
        return None;
    }

    let lower_head: String = trimmed.chars().take(256).collect::<String>().to_ascii_lowercase();

    // 1. HTML detection
    if lower_head.starts_with("<!doctype html")
        || lower_head.starts_with("<html")
        || lower_head.starts_with("<head")
        || lower_head.starts_with("<body")
    {
        return Some(DetectionReport {
            schema_version: 1,
            format_id: Some("html".to_string()),
            category: "text".to_string(),
            container_id: None,
            identification: Identification::Format,
            verification: Verification::Structure,
            inspection: Inspection::Complete,
            recommended_extension: Some("html".to_string()),
            accepted_extensions: vec!["html".to_string(), "htm".to_string()],
            extension_status: ExtensionStatus::Undetermined,
            evidence: vec![EvidenceItem::new("sig.text.html")],
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        });
    }

    // 2. XML detection
    if lower_head.starts_with("<?xml")
        || lower_head.starts_with("<svg")
        || lower_head.starts_with("<feed")
        || lower_head.starts_with("<rss")
        || (lower_head.starts_with('<') && lower_head.contains("xmlns="))
    {
        return Some(DetectionReport {
            schema_version: 1,
            format_id: Some("xml".to_string()),
            category: "text".to_string(),
            container_id: None,
            identification: Identification::Format,
            verification: Verification::Structure,
            inspection: Inspection::Complete,
            recommended_extension: Some("xml".to_string()),
            accepted_extensions: vec!["xml".to_string()],
            extension_status: ExtensionStatus::Undetermined,
            evidence: vec![EvidenceItem::new("sig.text.xml")],
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        });
    }

    None
}
