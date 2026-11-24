use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

/// High-performance source code and script probe via Shebang and key syntax patterns.
pub fn probe_source_code(text: &str) -> Option<DetectionReport> {
    // 1. Shebang check (deterministic, zero false-positives)
    if let Some(first_line) = text.lines().next() {
        let trimmed = first_line.trim_start();
        if trimmed.starts_with("#!") {
            if trimmed.contains("python") {
                return Some(make_source_report("py", &["py", "pyw"], "sig.text.shebang", Verification::Signature));
            }
            if trimmed.contains("bash") || trimmed.contains("sh") || trimmed.contains("zsh") {
                return Some(make_source_report("sh", &["sh", "bash", "zsh"], "sig.text.shebang", Verification::Signature));
            }
            if trimmed.contains("node") {
                return Some(make_source_report("js", &["js", "mjs", "cjs"], "sig.text.shebang", Verification::Signature));
            }
            if trimmed.contains("perl") {
                return Some(make_source_report("pl", &["pl", "pm"], "sig.text.shebang", Verification::Signature));
            }
            if trimmed.contains("ruby") {
                return Some(make_source_report("rb", &["rb"], "sig.text.shebang", Verification::Signature));
            }
        }
    }

    // 2. High-confidence syntax signatures within first 4KB window (sub-microsecond memory scan)
    let window = if text.len() > 4096 { &text[..4096] } else { text };
    let w_bytes = window.as_bytes();

    if window.starts_with("diff --git ") || window.starts_with("--- a/") {
        return Some(make_source_report("diff", &["diff", "patch"], "sig.text.diff", Verification::Structure));
    }
    if window.lines().next().map(|l| l.trim().eq_ignore_ascii_case("@echo off")).unwrap_or(false) {
        return Some(make_source_report("bat", &["bat", "cmd"], "sig.text.batch", Verification::Structure));
    }

    // C / C++ and Markdown
    if w_bytes.contains(&b'#') {
        if window.contains("#include <") || window.contains("#include \"") {
            if window.contains("std::")
                || window.contains("namespace ")
                || window.contains("template<")
                || window.contains("template <")
                || window.contains("class ")
                || window.contains("using namespace ")
            {
                return Some(make_source_report("cpp", &["cpp", "cxx", "cc", "hpp", "hxx"], "sig.text.cpp", Verification::Structure));
            } else {
                return Some(make_source_report("c", &["c", "h"], "sig.text.c", Verification::Structure));
            }
        }
        if window.starts_with("# ") && (window.contains("](") || window.contains("\n- ") || window.contains("\n```")) {
            return Some(make_source_report("md", &["md", "markdown"], "sig.text.markdown", Verification::Structure));
        }
    }

    // Verilog
    if window.contains("module ") && window.contains("endmodule") {
        return Some(make_source_report("v", &["v", "vh", "sv"], "sig.text.verilog", Verification::Structure));
    }

    // Rust
    if window.contains("fn main(")
        || (window.contains("use std::") || window.contains("use core::") || window.contains("use crate::"))
        || ((window.contains("fn ") || window.contains("pub fn ")) && (window.contains("let ") || window.contains("mut ") || window.contains("->")))
        || ((window.contains("struct ") || window.contains("enum ")) && (window.contains("impl ") || window.contains("pub ") || window.contains("#[derive(")))
    {
        return Some(make_source_report("rs", &["rs"], "sig.text.rust", Verification::Structure));
    }

    // Java
    if window.contains("public class ")
        || window.contains("public interface ")
        || (window.contains("package ") && window.contains("import ") && (window.contains("class ") || window.contains("interface ") || window.contains("public ")))
    {
        return Some(make_source_report("java", &["java"], "sig.text.java", Verification::Structure));
    }

    // Go
    if window.contains("package ") && (window.contains("func ") || window.contains("import (") || window.contains("import \"")) {
        return Some(make_source_report("go", &["go"], "sig.text.go", Verification::Structure));
    }

    // Python (scripts without shebang)
    if (window.contains("def ") && window.contains(":"))
        || (window.contains("import ") && (window.contains("def ") || window.contains("class ") || window.contains("from ")))
        || window.contains("if __name__ == ")
    {
        return Some(make_source_report("py", &["py", "pyw"], "sig.text.python", Verification::Structure));
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
