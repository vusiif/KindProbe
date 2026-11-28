use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

/// Probes whether the text represents a YAML or TOML configuration document, returning zero-allocation DetectionCore.
pub fn probe_config_core(text: &str) -> Option<DetectionCore> {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    // 1. INI detection: Semicolon comment followed by [section] or key=value
    let mut lines = trimmed.lines().filter(|l| !l.trim().is_empty());
    if let Some(first_line) = lines.next() {
        let fl = first_line.trim();
        if fl.starts_with(';') {
            if let Some(next_line) = lines.next() {
                let nl = next_line.trim();
                if (nl.starts_with('[') && nl.ends_with(']')) || nl.contains('=') {
                    return Some(make_config_core("ini", &["ini", "cfg", "conf"], "sig.text.ini"));
                }
            }
        }
        // 2. TOML detection: Section headers like [package] (excluding EditorConfig [*], systemd [Unit], XDG [Desktop Entry], .gitmodules, .gitconfig)
        if fl.starts_with('[')
            && fl.ends_with(']')
            && fl != "[*]"
            && fl != "[Unit]"
            && fl != "[Desktop Entry]"
            && fl != "[user]"
            && fl != "[core]"
            && !fl.starts_with("[submodule ")
            && !fl.starts_with("[program:")
            && fl != "[Flatpak Ref]"
            && !fl.contains("://")
            && !fl.contains('(')
            && fl.len() >= 3
        {
            return Some(make_config_core("toml", &["toml"], "sig.text.toml"));
        }
    }

    // 3. EML (RFC 5322 email) detection: >= 2 standard email headers at top
    let mut eml_headers = 0usize;
    for line in trimmed.lines().take(12) {
        if line.trim().is_empty() {
            break;
        }
        let lb = line.as_bytes();
        let is_hdr = lb.get(..5).is_some_and(|p| p.eq_ignore_ascii_case(b"from:"))
            || lb.get(..3).is_some_and(|p| p.eq_ignore_ascii_case(b"to:"))
            || lb.get(..8).is_some_and(|p| p.eq_ignore_ascii_case(b"subject:"))
            || lb.get(..5).is_some_and(|p| p.eq_ignore_ascii_case(b"date:"))
            || lb.get(..13).is_some_and(|p| p.eq_ignore_ascii_case(b"mime-version:"))
            || lb.get(..11).is_some_and(|p| p.eq_ignore_ascii_case(b"message-id:"));
        if is_hdr {
            eml_headers += 1;
            if eml_headers >= 2 {
                return Some(DetectionCore {
                    format_id: Some("eml"),
                    category: "email",
                    container_id: None,
                    identification: Identification::Format,
                    verification: Verification::Structure,
                    inspection: Inspection::Complete,
                    recommended_extension: Some("eml"),
                    accepted_extensions: &["eml", "mht"],
                    primary_evidence: Some("sig.text.eml"),
                    secondary_evidence: None,
                    warning: None,
                    candidate: None,
                });
            }
        }
    }

    // 2. YAML detection: Starts with document marker "---" (excluding Astro frontmatter) or structured key: value pairs
    if trimmed.starts_with("---")
        && !((trimmed.contains("import ") || trimmed.contains("const "))
            && trimmed[3..].contains("\n---")
            && trimmed.contains('<'))
    {
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

    if total_lines >= 2
        && valid_yaml_lines == total_lines
        && !(trimmed.contains("web: ") && (trimmed.contains("worker: ") || trimmed.contains("release: ")))
        && !(trimmed.contains("User-agent:") && (trimmed.contains("Disallow:") || trimmed.contains("Allow:")))
    {
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
