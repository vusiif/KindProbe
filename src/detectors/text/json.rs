use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

/// Probes whether the text represents a JSON document, returning zero-allocation DetectionCore.
pub fn probe_json_core(text: &str) -> Option<DetectionCore> {
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

    let geo_window = if text.len() > 4096 {
        let mut end = 4096;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        &text[..end]
    } else {
        text
    };

    let (fmt, cat, cont, rec, exts, ev): (
        &'static str,
        &'static str,
        Option<&'static str>,
        &'static str,
        &'static [&'static str],
        &'static str,
    ) = if geo_window.contains("\"FeatureCollection\"")
        || geo_window.contains("\"GeometryCollection\"")
        || (geo_window.contains("\"Feature\"") && geo_window.contains("\"geometry\""))
    {
        ("geojson", "geospatial", Some("json"), "geojson", &["geojson", "json"], "sig.text.geojson")
    } else {
        ("json", "text", None, "json", &["json"], "sig.text.json")
    };

    Some(DetectionCore {
        format_id: Some(fmt),
        category: cat,
        container_id: cont,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some(rec),
        accepted_extensions: exts,
        primary_evidence: Some(ev),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    })
}

/// Probes whether the text represents a JSON document.
pub fn probe_json(text: &str) -> Option<DetectionReport> {
    probe_json_core(text).map(|core| core.to_report(Default::default(), ExtensionStatus::Undetermined))
}
