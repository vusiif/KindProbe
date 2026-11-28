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
    if window.starts_with("Microsoft Visual Studio Solution File") {
        return Some(make_source_core("sln", &["sln"], "sig.text.sln", Verification::Structure));
    }
    if window.contains("@import(\"std\")") || (window.contains("pub fn ") && window.contains("@import(")) {
        return Some(make_source_core("zig", &["zig"], "sig.text.zig", Verification::Structure));
    }

    // CUDA (.cu / .cuh) and GLSL (.glsl / .vert / .frag) before C/C++ #include check
    if (window.contains("__global__") || window.contains("__device__"))
        && (window.contains("threadIdx") || window.contains("blockIdx") || window.contains("<<<"))
    {
        return Some(make_source_core(
            "cu",
            &["cu", "cuh"],
            "sig.text.cuda",
            Verification::Structure,
        ));
    }
    if window.contains("#version ")
        && window.contains("void main(")
        && (window.contains("gl_Position")
            || window.contains("gl_FragColor")
            || window.contains("uniform ")
            || window.contains("layout("))
    {
        return Some(make_source_core(
            "glsl",
            &["glsl", "vert", "frag"],
            "sig.text.glsl",
            Verification::Structure,
        ));
    }

    // WebGPU Shading Language (.wgsl) before Rust token scan (`fn ` / `->`)
    if (window.contains("@vertex") || window.contains("@fragment") || window.contains("@compute"))
        && window.contains("fn ")
        && (window.contains("@location(") || window.contains("@builtin(") || window.contains("@group("))
    {
        return Some(make_source_core(
            "wgsl",
            &["wgsl"],
            "sig.text.wgsl",
            Verification::Structure,
        ));
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

    // LaTeX (.tex), reStructuredText (.rst), AsciiDoc (.adoc) before code token scan
    // so embedded code snippets inside docs do not hijack detection.
    if window.contains("\\documentclass")
        && (window.contains("\\begin{document}") || window.contains("\\usepackage"))
    {
        return Some(make_source_core(
            "tex",
            &["tex", "ltx", "sty"],
            "sig.text.latex",
            Verification::Structure,
        ));
    }
    if window.contains(".. code-block::")
        || (window.contains(".. note::") && window.contains("===="))
    {
        return Some(make_source_core(
            "rst",
            &["rst"],
            "sig.text.rst",
            Verification::Structure,
        ));
    }
    if window.starts_with("= ") && (window.contains("\n== ") || window.contains("[source,")) {
        return Some(make_source_core(
            "adoc",
            &["adoc", "asciidoc"],
            "sig.text.adoc",
            Verification::Structure,
        ));
    }

    // Typst (.typ) before code token scan
    if (window.contains("#set ") || window.contains("#show ") || window.contains("#let "))
        && (window.contains("\n= ") || window.starts_with("= ") || window.contains("#import "))
    {
        return Some(make_source_core(
            "typ",
            &["typ"],
            "sig.text.typst",
            Verification::Structure,
        ));
    }

    // MDX (.mdx) before JS/TSX/Python scan
    if (window.contains("import ") || window.contains("export const "))
        && (window.contains("\n# ") || window.contains("\n## "))
        && window.contains('<')
        && window.contains("/>")
    {
        return Some(make_source_core(
            "mdx",
            &["mdx"],
            "sig.text.mdx",
            Verification::Structure,
        ));
    }

    // Mojo (.mojo) before Rust/Python token scan
    if window.contains("fn main():")
        && (window.contains("var ") || window.contains("let "))
        && window.contains("print(")
    {
        return Some(make_source_core(
            "mojo",
            &["mojo"],
            "sig.text.mojo",
            Verification::Structure,
        ));
    }

    // Bazel / Starlark (.bzl) before Python scan
    if (window.contains("load(\"@")
        || window.contains("load(\"//")
        || window.contains("cc_binary(")
        || window.contains("rust_binary(")
        || window.contains("http_archive("))
        && w_bytes.contains(&b'(')
    {
        return Some(make_source_core(
            "bzl",
            &["bzl", "bazel"],
            "sig.text.starlark",
            Verification::Structure,
        ));
    }

    // Gleam (.gleam) before Rust `pub fn ` scan
    if window.contains("import gleam/") && window.contains("pub fn ") {
        return Some(make_source_core(
            "gleam",
            &["gleam"],
            "sig.text.gleam",
            Verification::Structure,
        ));
    }

    // Odin (.odin) before Go `package ` scan
    if window.contains("package ") && window.contains("import \"core:") && window.contains("proc(") {
        return Some(make_source_core(
            "odin",
            &["odin"],
            "sig.text.odin",
            Verification::Structure,
        ));
    }

    // jq filter (.jq) before Python `def ...:` scan
    if window.contains("def ")
        && (window.contains("| select(") || window.contains("| map("))
        && w_bytes.contains(&b';')
    {
        return Some(make_source_core(
            "jq",
            &["jq"],
            "sig.text.jq",
            Verification::Structure,
        ));
    }

    // Racket (.rkt)
    if window.trim_start().starts_with("#lang racket")
        || window.trim_start().starts_with("#lang typed/racket")
        || window.trim_start().starts_with("#lang scribble")
    {
        return Some(make_source_core(
            "rkt",
            &["rkt"],
            "sig.text.racket",
            Verification::Signature,
        ));
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

    // Astro component (.astro)
    let trimmed_win = window.trim_start();
    if trimmed_win.starts_with("---")
        && (trimmed_win.contains("import ") || trimmed_win.contains("const "))
        && trimmed_win[3..].contains("\n---")
        && trimmed_win.contains('<')
    {
        return Some(make_source_core(
            "astro",
            &["astro"],
            "sig.text.astro",
            Verification::Structure,
        ));
    }

    // Prisma Schema (.prisma)
    if window.contains("datasource ")
        && (window.contains("generator ") || window.contains("model "))
        && (window.contains("provider") || window.contains("url"))
    {
        return Some(make_source_core(
            "prisma",
            &["prisma"],
            "sig.text.prisma",
            Verification::Structure,
        ));
    }

    // Solidity (.sol)
    if window.contains("pragma solidity")
        && (window.contains("contract ")
            || window.contains("interface ")
            || window.contains("library "))
    {
        return Some(make_source_core(
            "sol",
            &["sol"],
            "sig.text.solidity",
            Verification::Structure,
        ));
    }

    // Elixir (.ex / .exs)
    if window.contains("defmodule ") && window.contains(" do") && window.contains("end") {
        return Some(make_source_core(
            "ex",
            &["ex", "exs"],
            "sig.text.elixir",
            Verification::Structure,
        ));
    }

    // PureScript (.purs) before Haskell (.hs)
    if window.lines().any(|l| l.trim_start().starts_with("module "))
        && window.contains("where")
        && (window.contains("import Prelude") || window.contains("import Effect"))
        && !w_bytes.contains(&b'{')
    {
        return Some(make_source_core(
            "purs",
            &["purs"],
            "sig.text.purescript",
            Verification::Structure,
        ));
    }

    // Haskell (.hs)
    if window.lines().any(|l| l.trim_start().starts_with("module "))
        && window.contains("where")
        && (window.contains("import ") || window.contains(" :: "))
        && !w_bytes.contains(&b'{')
    {
        return Some(make_source_core(
            "hs",
            &["hs", "lhs"],
            "sig.text.haskell",
            Verification::Structure,
        ));
    }

    // Clojure (.clj / .cljs / .cljc)
    if window.contains("(ns ") && (window.contains("(defn ") || window.contains("(def ")) {
        return Some(make_source_core(
            "clj",
            &["clj", "cljs", "cljc", "edn"],
            "sig.text.clojure",
            Verification::Structure,
        ));
    }

    // Erlang (.erl / .hrl)
    if window.contains("-module(") && window.contains("-export(") {
        return Some(make_source_core(
            "erl",
            &["erl", "hrl"],
            "sig.text.erlang",
            Verification::Structure,
        ));
    }

    // Nim (.nim / .nims)
    if window.contains("proc ")
        && (window.contains("echo ") || window.contains("result ="))
        && !w_bytes.contains(&b'{')
    {
        return Some(make_source_core(
            "nim",
            &["nim", "nims"],
            "sig.text.nim",
            Verification::Structure,
        ));
    }

    // RIS bibliographic citation (.ris)
    if window.lines().any(|l| l.starts_with("TY  - "))
        && window.lines().any(|l| l.starts_with("ER  -"))
    {
        return Some(make_source_core(
            "ris",
            &["ris"],
            "sig.text.ris",
            Verification::Structure,
        ));
    }

    // BibTeX (.bib)
    if (window.contains("@article{")
        || window.contains("@inproceedings{")
        || window.contains("@book{")
        || window.contains("@misc{"))
        && (window.contains("author") || window.contains("title"))
    {
        return Some(make_source_core(
            "bib",
            &["bib", "bibtex"],
            "sig.text.bibtex",
            Verification::Structure,
        ));
    }

    // F# (.fs / .fsx)
    if (window.contains("open System") || window.contains("[<EntryPoint>]"))
        && (window.contains("let ") || window.contains("printfn "))
        && !w_bytes.contains(&b'{')
    {
        return Some(make_source_core(
            "fs",
            &["fs", "fsx", "fsi"],
            "sig.text.fsharp",
            Verification::Structure,
        ));
    }

    // VB.NET (.vb / .vbs)
    if (window.contains("Imports System") || window.contains("Module "))
        && (window.contains("End Module") || window.contains("End Sub"))
    {
        return Some(make_source_core(
            "vb",
            &["vb", "vbs"],
            "sig.text.vb",
            Verification::Structure,
        ));
    }

    // OCaml (.ml / .mli)
    if (window.contains("let rec ") || (window.contains("let ") && window.contains(";;")))
        && !w_bytes.contains(&b'{')
    {
        return Some(make_source_core(
            "ml",
            &["ml", "mli"],
            "sig.text.ocaml",
            Verification::Structure,
        ));
    }

    // Groovy (.groovy / .gvy)
    if (window.contains("@Grab(") || (window.contains("def ") && window.contains("println ")))
        && !window.contains("def ")
        || (window.contains("def ") && window.contains("println ") && !window.contains("def main(") && !window.contains(':'))
    {
        return Some(make_source_core(
            "groovy",
            &["groovy", "gvy"],
            "sig.text.groovy",
            Verification::Structure,
        ));
    }

    // Tcl (.tcl)
    if (window.contains("package require ")
        || (window.contains("proc ") && window.contains("puts ")))
        && w_bytes.contains(&b'{')
        && w_bytes.contains(&b'}')
    {
        return Some(make_source_core(
            "tcl",
            &["tcl"],
            "sig.text.tcl",
            Verification::Structure,
        ));
    }

    // AWK (.awk)
    if (window.contains("BEGIN {") || window.contains("END {"))
        && (window.contains("print ") || window.contains("printf ") || window.contains("$1") || window.contains("$0"))
        && w_bytes.contains(&b'}')
    {
        return Some(make_source_core(
            "awk",
            &["awk"],
            "sig.text.awk",
            Verification::Structure,
        ));
    }

    // WebAssembly Text Format (.wat / .wast)
    if window.contains("(module")
        && (window.contains("(func ")
            || window.contains("(memory ")
            || window.contains("(export ")
            || window.contains("(import "))
    {
        return Some(make_source_core(
            "wat",
            &["wat", "wast"],
            "sig.text.wat",
            Verification::Structure,
        ));
    }

    // VHDL (.vhd / .vhdl)
    if (window.contains("library ieee;") || window.contains("LIBRARY IEEE;"))
        && (window.contains("entity ") || window.contains("ENTITY "))
        && (window.contains("port (") || window.contains("port(") || window.contains("PORT ("))
    {
        return Some(make_source_core(
            "vhdl",
            &["vhd", "vhdl"],
            "sig.text.vhdl",
            Verification::Structure,
        ));
    }

    // PlantUML (.puml / .plantuml)
    if window.contains("@startuml") && window.contains("@enduml") {
        return Some(make_source_core(
            "puml",
            &["puml", "plantuml"],
            "sig.text.plantuml",
            Verification::Structure,
        ));
    }

    // Mermaid diagram (.mmd / .mermaid)
    let trimmed_mmd = window.trim_start();
    if (trimmed_mmd.starts_with("flowchart ")
        || trimmed_mmd.starts_with("graph ")
        || trimmed_mmd.starts_with("sequenceDiagram")
        || trimmed_mmd.starts_with("classDiagram")
        || trimmed_mmd.starts_with("stateDiagram")
        || trimmed_mmd.starts_with("erDiagram"))
        && (window.contains("-->") || window.contains("->") || window.contains("||--") || window.contains("participant "))
    {
        return Some(make_source_core(
            "mmd",
            &["mmd", "mermaid"],
            "sig.text.mermaid",
            Verification::Structure,
        ));
    }

    // Graphviz DOT (.dot / .gv)
    if (window.contains("digraph ") || window.contains("strict digraph ") || window.contains("strict graph "))
        && w_bytes.contains(&b'{')
        && w_bytes.contains(&b'}')
        && (window.contains("->") || window.contains("--"))
    {
        return Some(make_source_core(
            "dot",
            &["dot", "gv"],
            "sig.text.dot",
            Verification::Structure,
        ));
    }

    // Apache Thrift IDL (.thrift)
    if window.contains("namespace ")
        && (window.contains("struct ") || window.contains("service "))
        && (window.contains("1:") || window.contains("2:"))
    {
        return Some(make_source_core(
            "thrift",
            &["thrift"],
            "sig.text.thrift",
            Verification::Structure,
        ));
    }

    // FlatBuffers Schema (.fbs)
    if window.contains("table ")
        && (window.contains("root_type ") || window.contains("file_identifier "))
        && w_bytes.contains(&b'{')
    {
        return Some(make_source_core(
            "fbs",
            &["fbs"],
            "sig.text.fbs",
            Verification::Structure,
        ));
    }

    // Cap'n Proto (.capnp)
    if window.contains("@0x") && window.contains("struct ") && window.contains("@0") {
        return Some(make_source_core(
            "capnp",
            &["capnp"],
            "sig.text.capnp",
            Verification::Structure,
        ));
    }

    // AWS Smithy IDL (.smithy)
    if window.contains("$version:") && window.contains("namespace ") && (window.contains("service ") || window.contains("structure ")) {
        return Some(make_source_core(
            "smithy",
            &["smithy"],
            "sig.text.smithy",
            Verification::Structure,
        ));
    }

    // EditorConfig (.editorconfig)
    if (window.contains("root = true") || window.contains("[*]"))
        && (window.contains("indent_style") || window.contains("indent_size") || window.contains("end_of_line"))
    {
        return Some(make_source_core(
            "editorconfig",
            &["editorconfig"],
            "sig.text.editorconfig",
            Verification::Structure,
        ));
    }

    // Open Policy Agent Rego (.rego)
    if window.contains("package ")
        && (window.contains("default ") || window.contains("allow") || window.contains("deny"))
        && window.contains("input.")
        && w_bytes.contains(&b'{')
    {
        return Some(make_source_core(
            "rego",
            &["rego"],
            "sig.text.rego",
            Verification::Structure,
        ));
    }

    // Jsonnet (.jsonnet / .libsonnet)
    if window.contains("local ")
        && (window.contains("std.") || window.contains("+:") || window.contains("::"))
        && w_bytes.contains(&b'{')
        && w_bytes.contains(&b'}')
    {
        return Some(make_source_core(
            "jsonnet",
            &["jsonnet", "libsonnet"],
            "sig.text.jsonnet",
            Verification::Structure,
        ));
    }

    // Azure Bicep (.bicep)
    if window.contains("param ")
        && window.contains("resource ")
        && (window.contains("'Microsoft.") || window.contains("@description(") || window.contains("= {"))
    {
        return Some(make_source_core(
            "bicep",
            &["bicep"],
            "sig.text.bicep",
            Verification::Structure,
        ));
    }

    // DirectX HLSL (.hlsl / .fx)
    if (window.contains("cbuffer ") || window.contains("StructuredBuffer<") || window.contains("Texture2D"))
        && (window.contains("SV_Position") || window.contains("SV_Target") || window.contains("float4"))
    {
        return Some(make_source_core(
            "hlsl",
            &["hlsl", "fx"],
            "sig.text.hlsl",
            Verification::Structure,
        ));
    }

    // Dhall configuration (.dhall)
    if window.contains("let ")
        && (window.contains(" in ") || window.contains("\nin "))
        && (window.contains(": Type") || window.contains(": Text") || window.contains(": Natural"))
        && !w_bytes.contains(&b';')
    {
        return Some(make_source_core(
            "dhall",
            &["dhall"],
            "sig.text.dhall",
            Verification::Structure,
        ));
    }

    // Justfile (.just)
    if (window.contains("set shell") || window.contains(":="))
        && window.contains(":\n")
        && (window.contains("\n    ") || window.contains("\n\t"))
        && !window.contains(".PHONY:")
        && !w_bytes.contains(&b';')
    {
        return Some(make_source_core(
            "just",
            &["just"],
            "sig.text.just",
            Verification::Structure,
        ));
    }

    // Lean 4 (.lean)
    if (window.contains("theorem ") || window.contains("lemma ") || window.contains("inductive "))
        && window.contains(":=")
        && (window.contains("by ") || window.contains("Prop") || window.contains("Nat"))
    {
        return Some(make_source_core(
            "lean",
            &["lean"],
            "sig.text.lean",
            Verification::Structure,
        ));
    }

    // KDL Document Language (.kdl)
    if w_bytes.contains(&b'{')
        && w_bytes.contains(&b'}')
        && !w_bytes.contains(&b';')
        && window
            .lines()
            .any(|l| {
                let t = l.trim();
                t.ends_with(" {") && t.contains('"')
            })
        && !window.contains("server {")
        && !window.contains("plugins {")
        && !window.contains("job \"")
    {
        return Some(make_source_core(
            "kdl",
            &["kdl"],
            "sig.text.kdl",
            Verification::Structure,
        ));
    }

    // Rusty Object Notation (.ron)
    let ron_trimmed = window.trim();
    if ron_trimmed.starts_with('(')
        && ron_trimmed.ends_with(')')
        && ron_trimmed.contains(": ")
        && !w_bytes.contains(&b';')
        && !ron_trimmed.contains("(ns ")
        && !ron_trimmed.contains("(module")
    {
        return Some(make_source_core(
            "ron",
            &["ron"],
            "sig.text.ron",
            Verification::Structure,
        ));
    }

    // JSON5 / JSONC (.json5 / .jsonc)
    if (ron_trimmed.starts_with('{') || ron_trimmed.starts_with("//") || ron_trimmed.starts_with("/*"))
        && (window.contains("// ") || window.contains("/*"))
        && w_bytes.contains(&b'{')
        && w_bytes.contains(&b'}')
        && window.contains(": ")
        && !w_bytes.contains(&b';')
        && !window.contains("function ")
        && !window.contains("const ")
        && !window.contains("import ")
    {
        return Some(make_source_core(
            "json5",
            &["json5", "jsonc"],
            "sig.text.json5",
            Verification::Structure,
        ));
    }

    // Gherkin BDD (.feature)
    if window.contains("Feature:")
        && (window.contains("Scenario:") || window.contains("Scenario Outline:"))
        && (window.contains("Given ") || window.contains("When ") || window.contains("Then "))
    {
        return Some(make_source_core(
            "gherkin",
            &["feature"],
            "sig.text.gherkin",
            Verification::Structure,
        ));
    }

    // Neo4j Cypher (.cypher / .cql)
    if window.contains("MATCH (")
        && (window.contains("RETURN ") || window.contains("WHERE ") || window.contains("CREATE ("))
    {
        return Some(make_source_core(
            "cypher",
            &["cypher", "cql"],
            "sig.text.cypher",
            Verification::Structure,
        ));
    }

    // W3C SPARQL (.rq / .sparql)
    if (window.contains("SELECT ") || window.contains("PREFIX "))
        && (window.contains("WHERE {") || window.contains("WHERE{"))
        && (window.contains("?") || window.contains("FILTER "))
    {
        return Some(make_source_core(
            "sparql",
            &["rq", "sparql"],
            "sig.text.sparql",
            Verification::Structure,
        ));
    }

    // Well-Known Text Geometry (.wkt)
    if ron_trimmed.starts_with("POINT (")
        || ron_trimmed.starts_with("LINESTRING (")
        || ron_trimmed.starts_with("POLYGON ((")
        || ron_trimmed.starts_with("MULTIPOLYGON ((")
        || ron_trimmed.starts_with("GEOMETRYCOLLECTION (")
        || (ron_trimmed.starts_with("SRID=") && (ron_trimmed.contains(";POINT") || ron_trimmed.contains(";POLYGON")))
    {
        return Some(make_source_core(
            "wkt",
            &["wkt"],
            "sig.text.wkt",
            Verification::Structure,
        ));
    }

    // D programming language (.d)
    if window.contains("import std.")
        && (window.contains("void main(") || window.contains("int main(") || window.contains("writeln("))
    {
        return Some(make_source_core(
            "d",
            &["d", "di"],
            "sig.text.d",
            Verification::Structure,
        ));
    }

    // Caddyfile
    if window.contains("reverse_proxy ")
        && (w_bytes.contains(&b'{') || window.contains("encode ") || window.contains("file_server"))
    {
        return Some(make_source_core(
            "caddy",
            &["caddyfile"],
            "sig.text.caddy",
            Verification::Structure,
        ));
    }

    // HAProxy config (.cfg / haproxy.cfg)
    if window.contains("global\n")
        && window.contains("defaults\n")
        && (window.contains("frontend ") || window.contains("backend ") || window.contains("listen "))
    {
        return Some(make_source_core(
            "haproxy",
            &["cfg"],
            "sig.text.haproxy",
            Verification::Structure,
        ));
    }

    // systemd unit file (.service / .timer / .socket)
    if window.contains("[Unit]")
        && (window.contains("[Service]") || window.contains("[Timer]") || window.contains("[Socket]"))
        && (window.contains("ExecStart=") || window.contains("Description="))
    {
        return Some(make_source_core(
            "service",
            &["service", "timer", "socket"],
            "sig.text.systemd_unit",
            Verification::Structure,
        ));
    }

    // XDG Desktop Entry (.desktop)
    if window.contains("[Desktop Entry]")
        && window.contains("Type=")
        && window.contains("Name=")
    {
        return Some(make_source_core(
            "desktop",
            &["desktop"],
            "sig.text.desktop",
            Verification::Structure,
        ));
    }

    // AppleScript (.applescript / .scpt)
    if window.contains("tell application \"") && window.contains("end tell") {
        return Some(make_source_core(
            "applescript",
            &["applescript", "scpt"],
            "sig.text.applescript",
            Verification::Structure,
        ));
    }

    // Pug template (.pug / .jade)
    if window.trim_start().starts_with("doctype html") && !w_bytes.contains(&b'<') {
        return Some(make_source_core(
            "pug",
            &["pug", "jade"],
            "sig.text.pug",
            Verification::Structure,
        ));
    }

    // Handlebars (.hbs), Liquid (.liquid), Jinja2 (.j2 / .jinja), EJS (.ejs)
    if (window.contains("{{#if ") || window.contains("{{#each ")) && window.contains("{{/") {
        return Some(make_source_core(
            "hbs",
            &["hbs", "handlebars"],
            "sig.text.handlebars",
            Verification::Structure,
        ));
    }
    if (window.contains("{% endif %}") || window.contains("{% endfor %}")) && window.contains("{%") {
        return Some(make_source_core(
            "liquid",
            &["liquid"],
            "sig.text.liquid",
            Verification::Structure,
        ));
    }
    if (window.contains("{% block ") || window.contains("{% extends ") || window.contains("{% for "))
        && window.contains("%}")
    {
        return Some(make_source_core(
            "jinja",
            &["j2", "jinja", "jinja2"],
            "sig.text.jinja",
            Verification::Structure,
        ));
    }
    if (window.contains("<%=") || window.contains("<%-") || window.contains("<% ")) && window.contains("%>") {
        return Some(make_source_core(
            "ejs",
            &["ejs"],
            "sig.text.ejs",
            Verification::Structure,
        ));
    }

    // Jenkinsfile
    if window.contains("pipeline {")
        && window.contains("agent ")
        && window.contains("stages {")
        && window.contains("stage(")
    {
        return Some(make_source_core(
            "jenkinsfile",
            &["jenkinsfile", "groovy"],
            "sig.text.jenkinsfile",
            Verification::Structure,
        ));
    }

    // Earthfile
    if window.contains("VERSION ")
        && window.contains("FROM ")
        && (window.contains("SAVE ARTIFACT") || window.contains("BUILD +") || window.contains("RUN "))
    {
        return Some(make_source_core(
            "earthfile",
            &["earthfile"],
            "sig.text.earthfile",
            Verification::Structure,
        ));
    }

    // Meson build (meson.build)
    if window.contains("project(")
        && (window.contains("executable(") || window.contains("dependency(") || window.contains("library("))
        && !w_bytes.contains(&b'{')
    {
        return Some(make_source_core(
            "meson",
            &["build"],
            "sig.text.meson",
            Verification::Structure,
        ));
    }

    // Procfile
    if window.contains("web: ") && (window.contains("worker: ") || window.contains("release: ")) {
        return Some(make_source_core(
            "procfile",
            &["procfile"],
            "sig.text.procfile",
            Verification::Structure,
        ));
    }

    // Vagrantfile
    if window.contains("Vagrant.configure(") && window.contains("end") {
        return Some(make_source_core(
            "vagrantfile",
            &["vagrantfile"],
            "sig.text.vagrantfile",
            Verification::Structure,
        ));
    }

    // OpenSSH Client Config (ssh_config)
    if window.lines().any(|l| l.trim_start().starts_with("Host "))
        && (window.contains("HostName ") || window.contains("IdentityFile ") || window.contains("User "))
    {
        return Some(make_source_core(
            "ssh_config",
            &["config"],
            "sig.text.ssh_config",
            Verification::Structure,
        ));
    }

    // /etc/hosts
    if window.contains("127.0.0.1") && window.contains("localhost") {
        return Some(make_source_core(
            "hosts",
            &["hosts"],
            "sig.text.hosts",
            Verification::Structure,
        ));
    }

    // Crontab
    if (window.contains("* * * * *")
        || window.contains(" * * * ")
        || window.contains("@reboot ")
        || window.contains("@daily ")
        || window.contains("@hourly "))
        && window.contains('/')
    {
        return Some(make_source_core(
            "crontab",
            &["cron", "crontab"],
            "sig.text.crontab",
            Verification::Structure,
        ));
    }

    // Apache HTTP Server / .htaccess
    if window.contains("RewriteEngine ")
        || window.contains("RewriteRule ")
        || window.contains("<VirtualHost ")
        || window.contains("<Directory ")
    {
        return Some(make_source_core(
            "htaccess",
            &["htaccess", "conf"],
            "sig.text.apache",
            Verification::Structure,
        ));
    }

    // Ninja build (.ninja)
    if window.contains("rule ") && window.contains("command = ") && window.contains("build ") {
        return Some(make_source_core(
            "ninja",
            &["ninja"],
            "sig.text.ninja",
            Verification::Structure,
        ));
    }

    // HashiCorp Nomad (.nomad)
    if window.contains("job \"") && window.contains("group \"") && window.contains("task \"") {
        return Some(make_source_core(
            "nomad",
            &["nomad"],
            "sig.text.nomad",
            Verification::Structure,
        ));
    }

    // Apple Pkl (.pkl)
    if window.contains("amends \"")
        || (window.contains("module ") && window.contains("class ") && window.contains("= new "))
    {
        return Some(make_source_core(
            "pkl",
            &["pkl"],
            "sig.text.pkl",
            Verification::Structure,
        ));
    }

    // Elm (.elm)
    if window.contains("module ")
        && window.contains("exposing (")
        && (window.contains("import Html") || window.contains("import Browser") || window.contains("main ="))
    {
        return Some(make_source_core(
            "elm",
            &["elm"],
            "sig.text.elm",
            Verification::Structure,
        ));
    }

    // Vala (.vala)
    if (window.contains("using GLib;") || window.contains("public static int main("))
        && (window.contains("stdout.printf") || window.contains("print ("))
    {
        return Some(make_source_core(
            "vala",
            &["vala", "vapi"],
            "sig.text.vala",
            Verification::Structure,
        ));
    }

    // Fortran 90+ (.f90 / .f95)
    if window.contains("program ")
        && window.contains("implicit none")
        && window.contains("end program")
    {
        return Some(make_source_core(
            "f90",
            &["f90", "f95", "f03"],
            "sig.text.fortran",
            Verification::Structure,
        ));
    }

    // COBOL (.cob / .cbl)
    if window.contains("IDENTIFICATION DIVISION.") && window.contains("PROGRAM-ID.") {
        return Some(make_source_core(
            "cobol",
            &["cob", "cbl"],
            "sig.text.cobol",
            Verification::Structure,
        ));
    }

    // Ada (.adb / .ads)
    if window.contains("with Ada.") && window.contains("procedure ") && window.contains("begin") {
        return Some(make_source_core(
            "ada",
            &["adb", "ads", "ada"],
            "sig.text.ada",
            Verification::Structure,
        ));
    }

    // Pascal (.pas / .pp)
    if window.contains("program ")
        && window.contains("begin")
        && (window.contains("writeln(") || window.contains("WriteLn("))
        && window.contains("end.")
    {
        return Some(make_source_core(
            "pas",
            &["pas", "pp"],
            "sig.text.pascal",
            Verification::Structure,
        ));
    }

    // Scheme (.scm / .ss)
    if window.contains("(define (")
        && (window.contains("(lambda (") || window.contains("(let (") || window.contains("(cond "))
    {
        return Some(make_source_core(
            "scm",
            &["scm", "ss"],
            "sig.text.scheme",
            Verification::Structure,
        ));
    }

    // .gitmodules
    if window.contains("[submodule \"") && window.contains("path = ") && window.contains("url = ") {
        return Some(make_source_core(
            "gitmodules",
            &["gitmodules"],
            "sig.text.gitmodules",
            Verification::Structure,
        ));
    }

    // robots.txt
    if window.contains("User-agent:")
        && (window.contains("Disallow:") || window.contains("Allow:") || window.contains("Sitemap:"))
    {
        return Some(make_source_core(
            "robots",
            &["txt"],
            "sig.text.robots",
            Verification::Structure,
        ));
    }

    // .gitattributes
    if window.contains("text=auto")
        || (window.contains("filter=lfs") && window.contains("diff=lfs"))
    {
        return Some(make_source_core(
            "gitattributes",
            &["gitattributes"],
            "sig.text.gitattributes",
            Verification::Structure,
        ));
    }

    // CODEOWNERS
    if window.lines().filter(|l| {
        let t = l.trim();
        !t.starts_with('#') && (t.starts_with("* ") || t.starts_with('/')) && t.contains(" @")
    }).count() >= 2
    {
        return Some(make_source_core(
            "codeowners",
            &["codeowners"],
            "sig.text.codeowners",
            Verification::Structure,
        ));
    }

    // .npmrc
    if window.contains("registry=https://") || window.contains("//registry.npmjs.org/:") {
        return Some(make_source_core(
            "npmrc",
            &["npmrc"],
            "sig.text.npmrc",
            Verification::Structure,
        ));
    }

    // yarn.lock
    if window.contains("yarn lockfile v1") && window.contains("resolved \"") {
        return Some(make_source_core(
            "yarnlock",
            &["lock"],
            "sig.text.yarnlock",
            Verification::Structure,
        ));
    }

    // Gemfile
    if (window.contains("source 'https://rubygems.org'") || window.contains("source \"https://rubygems.org\""))
        && window.contains("gem ")
    {
        return Some(make_source_core(
            "gemfile",
            &["gemfile"],
            "sig.text.gemfile",
            Verification::Structure,
        ));
    }

    // CocoaPods Podfile
    if window.contains("platform :ios") && window.contains("target ") && window.contains("pod ") {
        return Some(make_source_core(
            "podfile",
            &["podfile"],
            "sig.text.podfile",
            Verification::Structure,
        ));
    }

    // Homebrew Brewfile
    if (window.contains("tap \"") || window.contains("brew \""))
        && (window.contains("brew \"") || window.contains("cask \""))
        && !w_bytes.contains(&b'{')
    {
        return Some(make_source_core(
            "brewfile",
            &["brewfile"],
            "sig.text.brewfile",
            Verification::Structure,
        ));
    }

    // Haxe (.hx)
    if window.contains("class ")
        && window.contains("static function main(")
        && window.contains("trace(")
    {
        return Some(make_source_core(
            "hx",
            &["hx"],
            "sig.text.haxe",
            Verification::Structure,
        ));
    }

    // PRQL (.prql)
    if window.contains("from ")
        && window.contains("filter ")
        && (window.contains("derive ") || window.contains("select "))
        && !w_bytes.contains(&b';')
    {
        return Some(make_source_core(
            "prql",
            &["prql"],
            "sig.text.prql",
            Verification::Structure,
        ));
    }

    // udev rules (.rules)
    if window.contains("ACTION==\"") && window.contains("SUBSYSTEM==\"") {
        return Some(make_source_core(
            "udev",
            &["rules"],
            "sig.text.udev",
            Verification::Structure,
        ));
    }

    // nftables (.nft)
    if window.contains("table inet ")
        && window.contains("chain ")
        && (window.contains("type filter hook") || window.contains("accept") || window.contains("drop"))
    {
        return Some(make_source_core(
            "nft",
            &["nft", "conf"],
            "sig.text.nftables",
            Verification::Structure,
        ));
    }

    // iptables-save (.rules / .v4)
    if window.contains("*filter") && window.contains(":INPUT ") && window.contains("COMMIT") {
        return Some(make_source_core(
            "iptables",
            &["rules", "v4"],
            "sig.text.iptables",
            Verification::Structure,
        ));
    }

    // DNS Zone file (.zone)
    if window.contains("$TTL ") && window.contains("IN SOA ") && window.contains("IN NS ") {
        return Some(make_source_core(
            "zone",
            &["zone"],
            "sig.text.dns_zone",
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

    // Gradle (.gradle) & Nginx (.conf)
    if (window.contains("plugins {") || window.contains("dependencies {"))
        && (window.contains("implementation")
            || window.contains("testImplementation")
            || window.contains("apply plugin:"))
    {
        return Some(make_source_core(
            "gradle",
            &["gradle"],
            "sig.text.gradle",
            Verification::Structure,
        ));
    }
    if (window.contains("server {") || window.contains("http {"))
        && (window.contains("listen ")
            || window.contains("location ")
            || window.contains("server_name "))
    {
        return Some(make_source_core(
            "nginx",
            &["conf"],
            "sig.text.nginx",
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

    // Less (.less) before CSS
    if w_bytes.contains(&b'{')
        && w_bytes.contains(&b'}')
        && w_bytes.contains(&b'@')
        && window.lines().any(|l| {
            let t = l.trim_start();
            t.starts_with('@') && t.contains(':') && t.ends_with(';') && !t.starts_with("@media") && !t.starts_with("@import")
        })
        && (window.contains("color:") || window.contains("margin:") || window.contains("padding:") || window.contains("display:"))
    {
        return Some(make_source_core(
            "less",
            &["less"],
            "sig.text.less",
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
