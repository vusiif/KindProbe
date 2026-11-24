use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

/// High-performance source code and script probe via Shebang and key syntax patterns.
pub fn probe_source_code(text: &str) -> Option<DetectionReport> {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return None;
    }
    let first = trimmed.as_bytes()[0];

    // 1. Shebang check - only possible when trimmed starts with '#'
    if first == b'#' {
        if trimmed.starts_with("#!") {
            if let Some(first_line) = trimmed.lines().next() {
                if first_line.contains("python") {
                    return Some(make_source_report("py", &["py", "pyw"], "sig.text.shebang", Verification::Signature));
                }
                if first_line.contains("bash") || first_line.contains("sh") || first_line.contains("zsh") {
                    return Some(make_source_report("sh", &["sh", "bash", "zsh"], "sig.text.shebang", Verification::Signature));
                }
                if first_line.contains("node") {
                    return Some(make_source_report("js", &["js", "mjs", "cjs"], "sig.text.shebang", Verification::Signature));
                }
                if first_line.contains("perl") {
                    return Some(make_source_report("pl", &["pl", "pm"], "sig.text.shebang", Verification::Signature));
                }
                if first_line.contains("ruby") {
                    return Some(make_source_report("rb", &["rb"], "sig.text.shebang", Verification::Signature));
                }
            }
        }

        let window = if text.len() > 4096 { &text[..4096] } else { text };
        if window.contains("#include <") || window.contains("#include \"") {
            if window.contains("std::") || window.contains("namespace ") || window.contains("template<") || window.contains("class ") {
                return Some(make_source_report("cpp", &["cpp", "cxx", "cc", "hpp", "hxx"], "sig.text.cpp", Verification::Structure));
            } else {
                return Some(make_source_report("c", &["c", "h"], "sig.text.c", Verification::Structure));
            }
        }
        if window.starts_with("# ") && (window.contains("](") || window.contains("\n- ") || window.contains("\n```")) {
            return Some(make_source_report("md", &["md", "markdown"], "sig.text.markdown", Verification::Structure));
        }
        if (window.contains("def ") && window.contains("):")) || window.contains("import sys") || window.contains("import os\n") {
            return Some(make_source_report("py", &["py", "pyw"], "sig.text.python", Verification::Structure));
        }
        return None;
    }

    // 2. High-confidence syntax signatures within first 4KB window (sub-microsecond memory scan)
    let window = if text.len() > 4096 { &text[..4096] } else { text };

    if (first == b'd' || first == b'-') && (window.starts_with("diff --git ") || window.starts_with("--- a/")) {
        return Some(make_source_report("diff", &["diff", "patch"], "sig.text.diff", Verification::Structure));
    }
    if first == b'@' && window.lines().next().map(|l| l.trim().eq_ignore_ascii_case("@echo off")).unwrap_or(false) {
        return Some(make_source_report("bat", &["bat", "cmd"], "sig.text.batch", Verification::Structure));
    }
    if (first == b'm' || first == b'/') && window.contains("module ") && window.contains("endmodule") {
        return Some(make_source_report("v", &["v", "vh", "sv"], "sig.text.verilog", Verification::Structure));
    }
    if first == b'p' || first == b'/' {
        if window.contains("public class ") || (window.contains("package ") && window.contains("import java.")) {
            return Some(make_source_report("java", &["java"], "sig.text.java", Verification::Structure));
        }
        if window.contains("package main\n") || (window.contains("package ") && window.contains("func main()")) {
            return Some(make_source_report("go", &["go"], "sig.text.go", Verification::Structure));
        }
    }
    if first == b'f' || first == b'u' || first == b'/' {
        if window.contains("fn main(") || (window.contains("use std::") && window.contains("fn ")) || (window.contains("pub struct ") && window.contains("impl ")) {
            return Some(make_source_report("rs", &["rs"], "sig.text.rust", Verification::Structure));
        }
        if window.contains("package main\n") || (window.contains("package ") && window.contains("func main()")) {
            return Some(make_source_report("go", &["go"], "sig.text.go", Verification::Structure));
        }
    }
    if (first == b'd' || first == b'i' || first == b'/')
        && ((window.contains("def ") && window.contains("):")) || window.contains("import sys") || window.contains("import os\n"))
    {
        return Some(make_source_report("py", &["py", "pyw"], "sig.text.python", Verification::Structure));
    }
    if first == b'/' && (window.contains("#include <") || window.contains("#include \"")) {
        if window.contains("std::") || window.contains("namespace ") || window.contains("template<") || window.contains("class ") {
            return Some(make_source_report("cpp", &["cpp", "cxx", "cc", "hpp", "hxx"], "sig.text.cpp", Verification::Structure));
        } else {
            return Some(make_source_report("c", &["c", "h"], "sig.text.c", Verification::Structure));
        }
    }

    None
}

fn make_source_report(fid: &'static str, exts: &[&'static str], evidence: &'static str, ver: Verification) -> DetectionReport {
    DetectionReport {
        schema_version: 1,
        format_id: Some(fid.to_string()),
        category: "text".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification: ver,
        inspection: Inspection::Complete,
        recommended_extension: Some(exts[0].to_string()),
        accepted_extensions: exts.iter().map(|s| s.to_string()).collect(),
        extension_status: ExtensionStatus::Undetermined,
        evidence: vec![EvidenceItem::new(evidence)],
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }
}
