use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

/// Probes whether the text represents an XML or HTML document and specialized dialects.
pub fn probe_xml_or_html(text: &str) -> Option<DetectionReport> {
    let trimmed = text.trim_start();
    if !trimmed.starts_with('<') {
        return None;
    }

    let head: String = trimmed.chars().take(256).collect::<String>().to_ascii_lowercase();

    // 1. HTML detection
    if head.starts_with("<!doctype html") || head.starts_with("<html") || head.starts_with("<head") || head.starts_with("<body") {
        return Some(make_rep("html", "text", None, "html", &["html", "htm"], "sig.text.html"));
    }

    // 2. Specialized XML dialects (SVG, KML, GPX, COLLADA DAE, Apple Plist)
    if head.starts_with("<svg") || (head.starts_with("<?xml") && head.contains("<svg")) {
        return Some(make_rep("svg", "image", Some("xml"), "svg", &["svg"], "sig.text.svg"));
    }
    if head.starts_with("<kml") || (head.starts_with("<?xml") && head.contains("<kml")) {
        return Some(make_rep("kml", "geospatial", Some("xml"), "kml", &["kml"], "sig.text.kml"));
    }
    if head.starts_with("<gpx") || (head.starts_with("<?xml") && head.contains("<gpx")) {
        return Some(make_rep("gpx", "geospatial", Some("xml"), "gpx", &["gpx"], "sig.text.gpx"));
    }
    if head.starts_with("<collada") || (head.starts_with("<?xml") && head.contains("<collada")) {
        return Some(make_rep("dae", "model", Some("xml"), "dae", &["dae"], "sig.text.collada"));
    }
    if head.starts_with("<!doctype plist") || head.starts_with("<plist") || (head.starts_with("<?xml") && head.contains("<plist")) {
        return Some(make_rep("plist", "configuration", Some("xml"), "plist", &["plist"], "sig.text.plist"));
    }

    // 3. Generic XML detection
    if head.starts_with("<?xml") || head.starts_with("<feed") || head.starts_with("<rss") || (head.starts_with('<') && head.contains("xmlns=")) {
        return Some(make_rep("xml", "text", None, "xml", &["xml"], "sig.text.xml"));
    }

    None
}

fn make_rep(fmt: &str, cat: &str, cont: Option<&str>, rec: &str, exts: &[&str], ev: &str) -> DetectionReport {
    DetectionReport {
        schema_version: 1,
        format_id: Some(fmt.to_string()),
        category: cat.to_string(),
        container_id: cont.map(|s| s.to_string()),
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some(rec.to_string()),
        accepted_extensions: exts.iter().map(|s| s.to_string()).collect(),
        extension_status: ExtensionStatus::Undetermined,
        evidence: vec![EvidenceItem::new(ev)],
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }
}
