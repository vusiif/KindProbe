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

    let (fmt, cat, cont, rec, exts, ev) = if text.contains("\"FeatureCollection\"")
        || text.contains("\"GeometryCollection\"")
        || (text.contains("\"Feature\"") && text.contains("\"geometry\""))
    {
        ("geojson", "geospatial", Some("json".to_string()), "geojson", vec!["geojson".to_string(), "json".to_string()], "sig.text.geojson")
    } else {
        ("json", "text", None, "json", vec!["json".to_string()], "sig.text.json")
    };

    Some(DetectionReport {
        schema_version: 1,
        format_id: Some(fmt.to_string()),
        category: cat.to_string(),
        container_id: cont,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some(rec.to_string()),
        accepted_extensions: exts,
        extension_status: ExtensionStatus::Undetermined,
        evidence: vec![EvidenceItem::new(ev)],
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    })
}
