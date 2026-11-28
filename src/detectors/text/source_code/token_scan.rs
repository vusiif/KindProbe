use super::make_source_core;
use crate::model::{DetectionCore, Verification};

pub(super) fn probe_token_scan(window: &str) -> Option<DetectionCore> {
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
        let split_pos = trimmed_bytes
            .iter()
            .position(|&b| b.is_ascii_whitespace() || b == b'(');
        let (first_word, rest) = match split_pos {
            Some(pos) => (&trimmed[..pos], trimmed[pos..].trim_start()),
            None => (trimmed, ""),
        };

        match first_word {
            "fn" => {
                has_rs_fn = true;
                if rest.starts_with("main(") || rest.starts_with("main (") {
                    return Some(make_source_core(
                        "rs",
                        &["rs"],
                        "sig.text.rust",
                        Verification::Structure,
                    ));
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
                        return Some(make_source_core(
                            "rs",
                            &["rs"],
                            "sig.text.rust",
                            Verification::Structure,
                        ));
                    }
                    if fn_rest.contains("->") || fn_rest.contains("mut ") {
                        has_rs_kw = true;
                    }
                } else if rest.starts_with("struct ") || rest.starts_with("enum ") {
                    has_rs_type = true;
                } else if rest.starts_with("mod ")
                    || rest.starts_with("use ")
                    || rest.starts_with("const ")
                {
                    has_rs_impl = true;
                }
            }
            "use" => {
                if rest.starts_with("std::")
                    || rest.starts_with("core::")
                    || rest.starts_with("crate::")
                {
                    return Some(make_source_core(
                        "rs",
                        &["rs"],
                        "sig.text.rust",
                        Verification::Structure,
                    ));
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
                    return Some(make_source_core(
                        "v",
                        &["v", "vh", "sv"],
                        "sig.text.verilog",
                        Verification::Structure,
                    ));
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
                    return Some(make_source_core(
                        "java",
                        &["java"],
                        "sig.text.java",
                        Verification::Structure,
                    ));
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
                    return Some(make_source_core(
                        "py",
                        &["py", "pyw"],
                        "sig.text.python",
                        Verification::Structure,
                    ));
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
        return Some(make_source_core(
            "v",
            &["v", "vh", "sv"],
            "sig.text.verilog",
            Verification::Structure,
        ));
    }

    if ((has_rs_fn && has_rs_kw) || (has_rs_type && has_rs_impl)) && !window.contains("@import(") {
        return Some(make_source_core(
            "rs",
            &["rs"],
            "sig.text.rust",
            Verification::Structure,
        ));
    }

    if has_package && has_import && has_class_or_pub {
        return Some(make_source_core(
            "java",
            &["java"],
            "sig.text.java",
            Verification::Structure,
        ));
    }

    if has_package && has_go_func {
        return Some(make_source_core(
            "go",
            &["go"],
            "sig.text.go",
            Verification::Structure,
        ));
    }

    // Kotlin & Scala (check before Python `def` fallback since Scala uses `def` + `package`/`object`/`case class`)
    if window.contains("fun ")
        && (window.contains("val ") || window.contains("var ") || window.contains("data class "))
        && !window.contains("fn ")
    {
        return Some(make_source_core(
            "kt",
            &["kt", "kts"],
            "sig.text.kotlin",
            Verification::Structure,
        ));
    }

    if window.contains("package ")
        && (window.contains("object ")
            || window.contains("case class ")
            || window.contains("trait "))
        && window.contains("def ")
    {
        return Some(make_source_core(
            "scala",
            &["scala", "sc"],
            "sig.text.scala",
            Verification::Structure,
        ));
    }

    // React TSX / JSX (check before Python `import` fallback since JS/TS uses `import ... from '...'`)
    if (window.contains("from 'react'")
        || window.contains("from \"react\"")
        || window.contains("import React")
        || window.contains("export default function "))
        && window.contains("return (")
        && window.contains("</")
    {
        if window.contains("interface ")
            || window.contains(": React.FC")
            || window.contains(": string")
            || window.contains(": number")
        {
            return Some(make_source_core(
                "tsx",
                &["tsx"],
                "sig.text.tsx",
                Verification::Structure,
            ));
        }
        return Some(make_source_core(
            "jsx",
            &["jsx"],
            "sig.text.jsx",
            Verification::Structure,
        ));
    }

    if has_py_def || (has_import && (has_py_def || has_class_or_pub || has_py_from)) {
        return Some(make_source_core(
            "py",
            &["py", "pyw"],
            "sig.text.python",
            Verification::Structure,
        ));
    }
    None
}
