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
    if window.starts_with("Windows Registry Editor Version 5.00") || window.starts_with("REGEDIT4") {
        return Some(make_source_core("reg", &["reg"], "sig.text.reg", Verification::Structure));
    }
    if window.contains("@import(\"std\")") || (window.contains("pub fn ") && window.contains("@import(")) {
        return Some(make_source_core("zig", &["zig"], "sig.text.zig", Verification::Structure));
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

    if ((has_rs_fn && has_rs_kw) || (has_rs_type && has_rs_impl)) && !window.contains("@import(") {
        return Some(make_source_core("rs", &["rs"], "sig.text.rust", Verification::Structure));
    }

    if has_package && has_import && has_class_or_pub {
        return Some(make_source_core("java", &["java"], "sig.text.java", Verification::Structure));
    }

    if has_package && has_go_func {
        return Some(make_source_core("go", &["go"], "sig.text.go", Verification::Structure));
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
        && (window.contains("object ") || window.contains("case class ") || window.contains("trait "))
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
        return Some(make_source_core("py", &["py", "pyw"], "sig.text.python", Verification::Structure));
    }

    // Dockerfile detection: first non-comment line starts with FROM <image> + instruction
    if let Some(first_inst) = window
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))
    {
        if first_inst.starts_with("FROM ")
            && first_inst.len() > 5
            && (window.contains("\nRUN ")
                || window.contains("\nCOPY ")
                || window.contains("\nWORKDIR ")
                || window.contains("\nCMD ")
                || window.contains("\nENTRYPOINT ")
                || window.contains("\nENV ")
                || window.contains("\nEXPOSE "))
        {
            return Some(make_source_core(
                "dockerfile",
                &["dockerfile"],
                "sig.text.dockerfile",
                Verification::Structure,
            ));
        }
    }

    // CMakeLists.txt / *.cmake
    if (window.contains("cmake_minimum_required(") || window.contains("CMAKE_MINIMUM_REQUIRED("))
        && (window.contains("project(")
            || window.contains("add_executable(")
            || window.contains("add_library("))
    {
        return Some(make_source_core(
            "cmake",
            &["cmake"],
            "sig.text.cmake",
            Verification::Structure,
        ));
    }

    // Makefile (.PHONY / target: + tab-indented recipe)
    if (window.contains(".PHONY:") || window.contains("all:") || window.contains("clean:"))
        && window.contains("\n\t")
    {
        return Some(make_source_core(
            "makefile",
            &["mk", "mak", "makefile"],
            "sig.text.makefile",
            Verification::Structure,
        ));
    }

    // Terraform / HCL (.tf / .hcl)
    if (window.contains("resource \"")
        || window.contains("provider \"")
        || window.contains("terraform {"))
        && (window.contains("variable \"")
            || window.contains("output \"")
            || window.contains("module \"")
            || window.contains("required_version")
            || window.contains("required_providers")
            || window.contains(" = "))
    {
        return Some(make_source_core(
            "hcl",
            &["tf", "hcl", "tfvars"],
            "sig.text.hcl",
            Verification::Structure,
        ));
    }

    // GraphQL schema / query
    if (window.contains("type ") || window.contains("query ") || window.contains("input "))
        && w_bytes.contains(&b'{')
        && (window.contains(": String")
            || window.contains(": Int")
            || window.contains(": ID")
            || window.contains(": Boolean"))
    {
        return Some(make_source_core(
            "graphql",
            &["graphql", "gql"],
            "sig.text.graphql",
            Verification::Structure,
        ));
    }

    // Dart
    if window.contains("import 'package:")
        || (window.contains("void main(")
            && (window.contains("Widget ") || window.contains("print("))
            && window.contains(';'))
    {
        return Some(make_source_core(
            "dart",
            &["dart"],
            "sig.text.dart",
            Verification::Structure,
        ));
    }

    // Protocol Buffers (.proto)
    if (window.contains("syntax = \"proto3\"") || window.contains("syntax = \"proto2\""))
        && window.contains("message ")
    {
        return Some(make_source_core(
            "proto",
            &["proto"],
            "sig.text.proto",
            Verification::Structure,
        ));
    }

    // C# (.cs)
    if window.contains("using System")
        && (window.contains("namespace ")
            || window.contains("class ")
            || window.contains("record "))
    {
        return Some(make_source_core(
            "cs",
            &["cs"],
            "sig.text.csharp",
            Verification::Structure,
        ));
    }

    // Swift (.swift)
    if (window.contains("import Foundation")
        || window.contains("import SwiftUI")
        || window.contains("import UIKit"))
        && (window.contains("func ")
            || window.contains("struct ")
            || window.contains("class ")
            || window.contains("let "))
    {
        return Some(make_source_core(
            "swift",
            &["swift"],
            "sig.text.swift",
            Verification::Structure,
        ));
    }

    // R (.r)
    if window.contains("<-")
        && (window.contains("library(")
            || window.contains("data.frame(")
            || window.contains("ggplot("))
    {
        return Some(make_source_core(
            "r",
            &["r", "R"],
            "sig.text.r",
            Verification::Structure,
        ));
    }

    // .env (dotenv)
    let env_hits = window
        .lines()
        .take(20)
        .filter(|l| {
            let t = l.trim();
            if let Some((k, _v)) = t.split_once('=') {
                !k.is_empty()
                    && !k.contains(' ')
                    && k.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
                    && (k.contains('_')
                        || matches!(k, "PORT" | "HOST" | "DEBUG" | "ENV" | "SECRET"))
            } else {
                false
            }
        })
        .count();
    if env_hits >= 2 {
        return Some(make_source_core(
            "env",
            &["env"],
            "sig.text.env",
            Verification::Structure,
        ));
    }

    // .gitignore / .dockerignore
    let ignore_patterns = [
        "target/",
        "node_modules/",
        "__pycache__/",
        ".DS_Store",
        "*.log",
        "*.pyc",
        "dist/",
        "build/",
        ".env",
    ];
    let ignore_hits = ignore_patterns
        .iter()
        .filter(|pat| window.lines().any(|l| l.trim() == **pat))
        .count();
    if ignore_hits >= 2 {
        return Some(make_source_core(
            "gitignore",
            &["gitignore", "dockerignore"],
            "sig.text.gitignore",
            Verification::Structure,
        ));
    }

    // PowerShell (.ps1)
    if (window.contains("[CmdletBinding(")
        || window.contains("param(")
        || window.contains("Param(")
        || window.contains("$PSScriptRoot"))
        && (window.contains("Write-Host")
            || window.contains("Write-Output")
            || window.contains("Get-ChildItem")
            || window.contains("Set-ExecutionPolicy")
            || w_bytes.contains(&b'$'))
    {
        return Some(make_source_core(
            "ps1",
            &["ps1", "psm1", "psd1"],
            "sig.text.powershell",
            Verification::Structure,
        ));
    }

    // Nix expression (.nix)
    if (window.contains("{ pkgs ?")
        || window.contains("stdenv.mkDerivation")
        || window.contains("pkgs.mkShell"))
        && window.contains(';')
    {
        return Some(make_source_core(
            "nix",
            &["nix"],
            "sig.text.nix",
            Verification::Structure,
        ));
    }

    // SQL (.sql)
    let has_sql = ((window.contains("CREATE TABLE ") || window.contains("create table "))
        && window.contains('('))
        || ((window.contains("SELECT ") || window.contains("select "))
            && (window.contains(" FROM ") || window.contains(" from "))
            && window.contains(';'))
        || ((window.contains("INSERT INTO ") || window.contains("insert into "))
            && (window.contains("VALUES") || window.contains("values")));
    if has_sql {
        return Some(make_source_core(
            "sql",
            &["sql"],
            "sig.text.sql",
            Verification::Structure,
        ));
    }

    // Lua (.lua)
    if window.contains("local ")
        && window.contains("function ")
        && window.contains("end")
        && !window.contains('{')
        && !window.contains("using ")
    {
        return Some(make_source_core(
            "lua",
            &["lua"],
            "sig.text.lua",
            Verification::Structure,
        ));
    }

    // Julia (.jl)
    if window.contains("using ")
        && window.contains("function ")
        && window.contains("end")
        && (window.contains("::") || window.contains("println(") || window.contains('@'))
    {
        return Some(make_source_core(
            "jl",
            &["jl"],
            "sig.text.julia",
            Verification::Structure,
        ));
    }

    // SCSS (check before CSS since SCSS shares CSS properties + `$` vars / `@mixin`)
    if w_bytes.contains(&b'{')
        && w_bytes.contains(&b'}')
        && (window.contains("@mixin ") || window.contains("@include ") || (w_bytes.contains(&b'$') && window.contains(": ")))
        && (window.contains("color:") || window.contains("margin:") || window.contains("padding:") || window.contains("display:"))
    {
        return Some(make_source_core(
            "scss",
            &["scss"],
            "sig.text.scss",
            Verification::Structure,
        ));
    }

    // CSS detection: braces + >= 2 CSS property/at-rule markers
    if w_bytes.contains(&b'{') && w_bytes.contains(&b'}') {
        let css_markers = [
            "color:",
            "display:",
            "margin:",
            "padding:",
            "font-size:",
            "background:",
            "width:",
            "height:",
            "border:",
            "@media ",
            "@import ",
        ];
        let hits = css_markers.iter().filter(|m| window.contains(*m)).count();
        if hits >= 2 {
            return Some(make_source_core(
                "css",
                &["css"],
                "sig.text.css",
                Verification::Structure,
            ));
        }
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
