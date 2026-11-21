use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

/// Probes whether the text represents a JSON document.
pub fn probe_json(text: &str) -> Option<DetectionReport> {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    let first = trimmed.chars().next()?;
    let is_json = match first {
        '{' => {
            let rest = trimmed[1..].trim_start();
            rest.is_empty()
                || rest.starts_with('}')
                || rest.starts_with('"')
        }
        '[' => {
            let rest = trimmed[1..].trim_start();
            if rest.is_empty() || rest.starts_with(']') {
                true
            } else if let Some(next_ch) = rest.chars().next() {
                next_ch == '{'
                    || next_ch == '['
                    || next_ch == '"'
                    || next_ch.is_ascii_digit()
                    || next_ch == '-'
                    || rest.starts_with("true")
                    || rest.starts_with("false")
                    || rest.starts_with("null")
            } else {
                false
            }
        }
        _ => false,
    };

    if !is_json {
        return None;
    }

    Some(DetectionReport {
        schema_version: 1,
        format_id: Some("json".to_string()),
        category: "text".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("json".to_string()),
        accepted_extensions: vec!["json".to_string()],
        extension_status: ExtensionStatus::Undetermined,
        evidence: vec![EvidenceItem::new("sig.text.json")],
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    })
}
