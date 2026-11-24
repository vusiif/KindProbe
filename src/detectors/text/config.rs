use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

/// Probes whether the text represents a YAML or TOML configuration document, returning zero-allocation DetectionCore.
pub fn probe_config_core(text: &str) -> Option<DetectionCore> {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    // 1. TOML detection: Section headers like [package] or multiple key = value lines
    let mut lines = trimmed.lines().filter(|l| !l.trim().is_empty());
    if let Some(first_line) = lines.next() {
        let fl = first_line.trim();
        if fl.starts_with('[') && fl.ends_with(']') && !fl.contains("://") && fl.len() >= 3 {
            return Some(make_config_core("toml", &["toml"], "sig.text.toml"));
        }
    }

    // 2. YAML detection: Starts with document marker "---" or structured key: value pairs
    if trimmed.starts_with("---") {
        return Some(make_config_core("yaml", &["yaml", "yml"], "sig.text.yaml"));
    }

    // Heuristic for key: value structured documents
    let mut valid_yaml_lines = 0usize;
    let mut total_lines = 0usize;

    for line in trimmed.lines().take(10) {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        total_lines += 1;
        if l.starts_with("- ") {
            valid_yaml_lines += 1;
        } else if let Some((k, _v)) = l.split_once(':') {
            let k_trimmed = k.trim();
            if !k_trimmed.is_empty()
                && !k_trimmed.contains(' ')
                && k_trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                valid_yaml_lines += 1;
            }
        }
    }

    if total_lines >= 2 && valid_yaml_lines == total_lines {
        return Some(make_config_core("yaml", &["yaml", "yml"], "sig.text.yaml"));
    }

    None
}

/// Probes whether the text represents a YAML or TOML configuration document.
pub fn probe_config(text: &str) -> Option<DetectionReport> {
    probe_config_core(text).map(|core| core.to_report(Default::default(), ExtensionStatus::Undetermined))
}

fn make_config_core(fid: &'static str, exts: &'static [&'static str], ev: &'static str) -> DetectionCore {
    DetectionCore {
        format_id: Some(fid),
        category: "text",
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some(fid),
        accepted_extensions: exts,
        primary_evidence: Some(ev),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}
