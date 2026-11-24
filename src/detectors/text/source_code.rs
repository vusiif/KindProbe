use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

/// High-performance source code and script probe returning zero-allocation DetectionCore.
pub fn probe_source_code_core(text: &str) -> Option<DetectionCore> {
    // 1. Shebang check (deterministic, zero false-positives)
    if let Some(first_line) = text.lines().next() {
        let trimmed = first_line.trim_start();
        if trimmed.starts_with("#!") {
            if trimmed.contains("python") {
                return Some(make_source_core("py", &["py", "pyw"], "sig.text.shebang", Verification::Signature));
            }
            if trimmed.contains("bash") || trimmed.contains("sh") || trimmed.contains("zsh") {
                return Some(make_source_core("sh", &["sh", "bash", "zsh"], "sig.text.shebang", Verification::Signature));
            }
            if trimmed.contains("node") {
                return Some(make_source_core("js", &["js", "mjs", "cjs"], "sig.text.shebang", Verification::Signature));
            }
            if trimmed.contains("perl") {
                return Some(make_source_core("pl", &["pl", "pm"], "sig.text.shebang", Verification::Signature));
            }
            if trimmed.contains("ruby") {
                return Some(make_source_core("rb", &["rb"], "sig.text.shebang", Verification::Signature));
            }
        }
    }

    // 2. High-confidence syntax signatures within first 4KB window (sub-microsecond memory scan)
    // Safely clamp to char boundary to prevent panic on multi-byte UTF-8 character splits
    let window = if text.len() > 4096 {
        let mut end = 4096;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        &text[..end]
    } else {
        text
    };
    let w_bytes = window.as_bytes();

    if window.starts_with("diff --git ") || window.starts_with("--- a/") {
        return Some(make_source_core("diff", &["diff", "patch"], "sig.text.diff", Verification::Structure));
    }
    if window.lines().next().map(|l| l.trim().eq_ignore_ascii_case("@echo off")).unwrap_or(false) {
        return Some(make_source_core("bat", &["bat", "cmd"], "sig.text.batch", Verification::Structure));
    }

    // C / C++ and Markdown fast check (only executed if `#` is present)
    let has_hash = w_bytes.contains(&b'#');
    if has_hash {
        if window.contains("#include <") || window.contains("#include \"") {
            if window.contains("std::")
                || window.contains("namespace ")
                || window.contains("template<")
                || window.contains("template <")
                || window.contains("class ")
                || window.contains("using namespace ")
            {
                return Some(make_source_core("cpp", &["cpp", "cxx", "cc", "hpp", "hxx"], "sig.text.cpp", Verification::Structure));
            } else {
                return Some(make_source_core("c", &["c", "h"], "sig.text.c", Verification::Structure));
            }
        }
        if window.starts_with("# ") && (window.contains("](") || window.contains("\n- ") || window.contains("\n```")) {
            return Some(make_source_core("md", &["md", "markdown"], "sig.text.markdown", Verification::Structure));
        }
    }

    let mut has_v_module = false;
    let mut has_v_end = false;
    let mut has_rs_fn = false;
    let mut has_rs_kw = false;
    let mut has_rs_type = false;
    let mut has_rs_impl = false;
    let mut has_package = false;
    let mut has_import = false;
    let mut has_class_or_pub = false;
    let mut has_go_func = false;
    let mut has_py_def = false;
    let mut has_py_from = false;

    // Single-pass line-based token feature extraction
    for line in window.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }

        // Fast comment and docstring bypass
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }
        if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
            continue;
        }

        // High-performance byte-level token boundary scan
        let trimmed_bytes = trimmed.as_bytes();
        let split_pos = trimmed_bytes.iter().position(|&b| b.is_ascii_whitespace() || b == b'(');
        let (first_word, rest) = match split_pos {
            Some(pos) => (&trimmed[..pos], trimmed[pos..].trim_start()),
            None => (trimmed, ""),
        };

        match first_word {
            "fn" => {
                has_rs_fn = true;
                if rest.starts_with("main(") || rest.starts_with("main (") {
                    return Some(make_source_core("rs", &["rs"], "sig.text.rust", Verification::Structure));
                }
                if rest.contains("->") || rest.contains("mut ") {
                    has_rs_kw = true;
                }
            }
            "pub" => {
                has_rs_impl = true;
                has_class_or_pub = true;
                if let Some(stripped) = rest.strip_prefix("fn ") {
                    has_rs_fn = true;
                    let fn_rest = stripped.trim_start();
                    if fn_rest.starts_with("main(") || fn_rest.starts_with("main (") {
                        return Some(make_source_core("rs", &["rs"], "sig.text.rust", Verification::Structure));
                    }
                    if fn_rest.contains("->") || fn_rest.contains("mut ") {
                        has_rs_kw = true;
                    }
                } else if rest.starts_with("struct ") || rest.starts_with("enum ") {
                    has_rs_type = true;
                } else if rest.starts_with("mod ") || rest.starts_with("use ") || rest.starts_with("const ") {
                    has_rs_impl = true;
                }
            }
            "use" => {
                if rest.starts_with("std::") || rest.starts_with("core::") || rest.starts_with("crate::") {
                    return Some(make_source_core("rs", &["rs"], "sig.text.rust", Verification::Structure));
                }
            }
            "struct" | "enum" => {
                has_rs_type = true;
            }
            "let" => {
                has_rs_kw = true;
            }
            "impl" => {
                has_rs_impl = true;
            }
            "module" => {
                has_v_module = true;
            }
            "endmodule" => {
                has_v_end = true;
                if has_v_module {
                    return Some(make_source_core("v", &["v", "vh", "sv"], "sig.text.verilog", Verification::Structure));
                }
            }
            "package" => {
                has_package = true;
            }
            "import" => {
                has_import = true;
                if rest.starts_with('(') || rest.starts_with('"') {
                    has_go_func = true;
                }
            }
            "func" => {
                has_go_func = true;
            }
            "public" | "private" | "protected" => {
                has_class_or_pub = true;
                if rest.starts_with("class ")
                    || rest.starts_with("interface ")
                    || rest.starts_with("static class ")
                    || rest.starts_with("final class ")
                {
                    return Some(make_source_core("java", &["java"], "sig.text.java", Verification::Structure));
                }
            }
            "class" | "interface" => {
                has_class_or_pub = true;
            }
            "def" => {
                if trimmed.contains(':') {
                    has_py_def = true;
                }
            }
            "from" => {
                has_py_from = true;
                if trimmed.contains("import ") {
                    has_import = true;
                }
            }
            "if" => {
                if rest.starts_with("__name__ ==")
                    || rest.starts_with("__name__==")
                    || trimmed.contains("if __name__ == ")
                {
                    return Some(make_source_core("py", &["py", "pyw"], "sig.text.python", Verification::Structure));
                }
            }
            _ => {
                if trimmed.starts_with("#[derive(") || trimmed.starts_with("#[cfg(") {
                    has_rs_impl = true;
                }
            }
        }
    }

    if has_v_module && has_v_end {
        return Some(make_source_core("v", &["v", "vh", "sv"], "sig.text.verilog", Verification::Structure));
    }

    if (has_rs_fn && has_rs_kw) || (has_rs_type && has_rs_impl) {
        return Some(make_source_core("rs", &["rs"], "sig.text.rust", Verification::Structure));
    }

    if has_package && has_import && has_class_or_pub {
        return Some(make_source_core("java", &["java"], "sig.text.java", Verification::Structure));
    }

    if has_package && has_go_func {
        return Some(make_source_core("go", &["go"], "sig.text.go", Verification::Structure));
    }

    if has_py_def || (has_import && (has_py_def || has_class_or_pub || has_py_from)) {
        return Some(make_source_core("py", &["py", "pyw"], "sig.text.python", Verification::Structure));
    }

    None
}

/// Probes source code returning an owned DetectionReport.
pub fn probe_source_code(text: &str) -> Option<DetectionReport> {
    probe_source_code_core(text).map(|core| core.to_report(Default::default(), ExtensionStatus::Undetermined))
}

fn make_source_core(fid: &'static str, exts: &'static [&'static str], evidence: &'static str, ver: Verification) -> DetectionCore {
    DetectionCore {
        format_id: Some(fid),
        category: "text",
        container_id: None,
        identification: Identification::Format,
        verification: ver,
        inspection: Inspection::Complete,
        recommended_extension: Some(exts[0]),
        accepted_extensions: exts,
        primary_evidence: Some(evidence),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}
