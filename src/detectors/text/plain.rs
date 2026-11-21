use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

/// Makes a detection report for plain text files.
pub fn make_plain_text_report() -> DetectionReport {
    DetectionReport {
        schema_version: 1,
        format_id: Some("txt".to_string()),
        category: "text".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("txt".to_string()),
        accepted_extensions: vec!["txt".to_string(), "text".to_string(), "log".to_string()],
        extension_status: ExtensionStatus::Undetermined,
        evidence: vec![EvidenceItem::new("sig.text.plain")],
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }
}
