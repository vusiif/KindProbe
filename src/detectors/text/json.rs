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

    let mut lines = trimmed.lines().map(str::trim).filter(|l| !l.is_empty());
    let is_jsonl = match (lines.next(), lines.next()) {
        (Some(l1), Some(l2)) => {
            l1.starts_with('{') && l1.ends_with('}') && l2.starts_with('{') && l2.ends_with('}')
        }
        _ => false,
    };

    let (fmt, cat, cont, rec, exts, ev): (
        &'static str,
        &'static str,
        Option<&'static str>,
        &'static str,
        &'static [&'static str],
        &'static str,
    ) = if is_jsonl {
        ("jsonl", "text", None, "jsonl", &["jsonl", "ndjson"], "sig.text.jsonl")
    } else if geo_window.contains("\"FeatureCollection\"")
        || geo_window.contains("\"GeometryCollection\"")
        || (geo_window.contains("\"Feature\"") && geo_window.contains("\"geometry\""))
    {
        ("geojson", "geospatial", Some("json"), "geojson", &["geojson", "json"], "sig.text.geojson")
    } else if geo_window.contains("\"nbformat\"") && geo_window.contains("\"cells\"") {
        ("ipynb", "document", Some("json"), "ipynb", &["ipynb"], "sig.text.ipynb")
    } else if geo_window.contains("\"asset\"")
        && geo_window.contains("\"version\"")
        && (geo_window.contains("\"scenes\"")
            || geo_window.contains("\"nodes\"")
            || geo_window.contains("\"meshes\""))
    {
        ("gltf", "3d", Some("json"), "gltf", &["gltf"], "sig.text.gltf")
    } else if geo_window.contains("\"log\"")
        && geo_window.contains("\"entries\"")
        && (geo_window.contains("\"pages\"")
            || geo_window.contains("\"creator\"")
            || geo_window.contains("\"version\""))
    {
        ("har", "network", Some("json"), "har", &["har"], "sig.text.har")
    } else if geo_window.contains("\"layers\"")
        && geo_window.contains("\"fr\"")
        && geo_window.contains("\"ip\"")
        && geo_window.contains("\"op\"")
        && geo_window.contains("\"v\"")
    {
        ("lottie", "animation", Some("json"), "json", &["json", "lottie"], "sig.text.lottie")
    } else if geo_window.contains("\"terraform_version\"")
        && geo_window.contains("\"resources\"")
        && geo_window.contains("\"version\"")
    {
        ("tfstate", "configuration", Some("json"), "tfstate", &["tfstate", "json"], "sig.text.tfstate")
    } else if (geo_window.contains("\"openapi\"") || geo_window.contains("\"swagger\""))
        && geo_window.contains("\"paths\"")
        && geo_window.contains("\"info\"")
    {
        ("openapi", "configuration", Some("json"), "json", &["json"], "sig.text.openapi")
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
