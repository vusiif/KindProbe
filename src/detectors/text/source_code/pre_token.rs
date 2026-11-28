use super::make_source_core;
use crate::model::{DetectionCore, Verification};

pub(super) fn probe_shebang_and_pretoken(
    text: &str,
    window: &str,
    w_bytes: &[u8],
) -> Option<DetectionCore> {
    // 1. Shebang check (deterministic, zero false-positives)
    if let Some(first_line) = text.lines().next() {
        let trimmed = first_line.trim_start();
        if trimmed.starts_with("#!") {
            if trimmed.contains("python") {
                return Some(make_source_core(
                    "py",
                    &["py", "pyw"],
                    "sig.text.shebang",
                    Verification::Signature,
                ));
            }
            if trimmed.contains("bash") || trimmed.contains("sh") || trimmed.contains("zsh") {
                return Some(make_source_core(
                    "sh",
                    &["sh", "bash", "zsh"],
                    "sig.text.shebang",
                    Verification::Signature,
                ));
            }
            if trimmed.contains("node") {
                return Some(make_source_core(
                    "js",
                    &["js", "mjs", "cjs"],
                    "sig.text.shebang",
                    Verification::Signature,
                ));
            }
            if trimmed.contains("perl") {
                return Some(make_source_core(
                    "pl",
                    &["pl", "pm"],
                    "sig.text.shebang",
                    Verification::Signature,
                ));
            }
            if trimmed.contains("ruby") {
                return Some(make_source_core(
                    "rb",
                    &["rb"],
                    "sig.text.shebang",
                    Verification::Signature,
                ));
            }
            if trimmed.contains("sed") {
                return Some(make_source_core(
                    "sed",
                    &["sed"],
                    "sig.text.shebang",
                    Verification::Signature,
                ));
            }
        }
    }

    if window.starts_with("diff --git ") || window.starts_with("--- a/") {
        return Some(make_source_core(
            "diff",
            &["diff", "patch"],
            "sig.text.diff",
            Verification::Structure,
        ));
    }
    if window
        .lines()
        .next()
        .map(|l| l.trim().eq_ignore_ascii_case("@echo off"))
        .unwrap_or(false)
    {
        return Some(make_source_core(
            "bat",
            &["bat", "cmd"],
            "sig.text.batch",
            Verification::Structure,
        ));
    }
    if window.starts_with("Windows Registry Editor Version 5.00") || window.starts_with("REGEDIT4")
    {
        return Some(make_source_core(
            "reg",
            &["reg"],
            "sig.text.reg",
            Verification::Structure,
        ));
    }
    if window.starts_with("Microsoft Visual Studio Solution File") {
        return Some(make_source_core(
            "sln",
            &["sln"],
            "sig.text.sln",
            Verification::Structure,
        ));
    }
    if window.contains("@import(\"std\")")
        || (window.contains("pub fn ") && window.contains("@import("))
    {
        return Some(make_source_core(
            "zig",
            &["zig"],
            "sig.text.zig",
            Verification::Structure,
        ));
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
        && (window.contains("@location(")
            || window.contains("@builtin(")
            || window.contains("@group("))
    {
        return Some(make_source_core(
            "wgsl",
            &["wgsl"],
            "sig.text.wgsl",
            Verification::Structure,
        ));
    }

    // POV-Ray (.pov) before C/C++ `#include "` check
    if window.contains("#include \"colors.inc\"")
        || (window.contains("camera {") && window.contains("light_source {"))
    {
        return Some(make_source_core(
            "pov",
            &["pov", "inc"],
            "sig.text.povray",
            Verification::Structure,
        ));
    }

    // Apple Metal Shader (.metal) before C/C++ `#include <` check
    if window.contains("#include <metal_stdlib>") && window.contains("using namespace metal;") {
        return Some(make_source_core(
            "metal",
            &["metal"],
            "sig.text.metal",
            Verification::Structure,
        ));
    }

    // C / C++ and Markdown fast check (only executed if `#` is present)
    let has_hash = w_bytes.contains(&b'#');
    if has_hash {
        // Check structured Markdown documents before C `#include` so C code blocks inside docs don't hijack Markdown.
        if window.starts_with("# ")
            && (window.contains("](") || window.contains("\n- ") || window.contains("\n```"))
        {
            return Some(make_source_core(
                "md",
                &["md", "markdown"],
                "sig.text.markdown",
                Verification::Structure,
            ));
        }
        if window.contains("#include <") || window.contains("#include \"") {
            if window.contains("std::")
                || window.contains("namespace ")
                || window.contains("template<")
                || window.contains("template <")
                || window.contains("class ")
                || window.contains("using namespace ")
            {
                return Some(make_source_core(
                    "cpp",
                    &["cpp", "cxx", "cc", "hpp", "hxx"],
                    "sig.text.cpp",
                    Verification::Structure,
                ));
            } else {
                return Some(make_source_core(
                    "c",
                    &["c", "h"],
                    "sig.text.c",
                    Verification::Structure,
                ));
            }
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
    if window.contains("package ") && window.contains("import \"core:") && window.contains("proc(")
    {
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

    // Nushell (.nu) before Python/Ruby `def ` scan
    if window.contains("def ")
        && w_bytes.contains(&b'[')
        && w_bytes.contains(&b']')
        && (window.contains("| where ") || window.contains("| select ") || window.contains("mut "))
        && !w_bytes.contains(&b':')
    {
        return Some(make_source_core(
            "nu",
            &["nu"],
            "sig.text.nushell",
            Verification::Structure,
        ));
    }

    // SystemVerilog (.sv / .svh) before Verilog scan
    if window.contains("module ")
        && window.contains("endmodule")
        && (window.contains("logic ")
            || window.contains("always_ff")
            || window.contains("always_comb"))
    {
        return Some(make_source_core(
            "sv",
            &["sv", "svh"],
            "sig.text.systemverilog",
            Verification::Structure,
        ));
    }

    // Vyper (.vy) before Python `def` scan
    if window.contains("@external")
        && window.contains("def ")
        && (window.contains("uint256") || window.contains("public(") || window.contains("@view"))
    {
        return Some(make_source_core(
            "vy",
            &["vy"],
            "sig.text.vyper",
            Verification::Structure,
        ));
    }

    // Move (.move) before Rust/Kotlin `fun` scan
    if window.contains("module ")
        && window.contains("public fun ")
        && (window.contains("has key")
            || window.contains("has store")
            || window.contains("use std::"))
    {
        return Some(make_source_core(
            "move",
            &["move"],
            "sig.text.move",
            Verification::Structure,
        ));
    }

    // Cairo (.cairo) before Rust `fn` scan
    if window.contains("#[starknet::contract]")
        || (window.contains("use starknet::") && window.contains("fn "))
    {
        return Some(make_source_core(
            "cairo",
            &["cairo"],
            "sig.text.cairo",
            Verification::Structure,
        ));
    }

    // Sway (.sw) before Rust `fn` scan
    if (window.contains("contract;") || window.contains("script;"))
        && (window.contains("abi ") || window.contains("fn main("))
    {
        return Some(make_source_core(
            "sway",
            &["sw"],
            "sig.text.sway",
            Verification::Structure,
        ));
    }

    // C3 (.c3) before Rust `fn` scan
    if window.contains("module ") && window.contains("fn ") && window.contains("io::printn(") {
        return Some(make_source_core(
            "c3",
            &["c3", "c3i"],
            "sig.text.c3",
            Verification::Structure,
        ));
    }

    // Cargo.lock & Poetry poetry.lock
    if window.contains("# This file is automatically @generated by Cargo.")
        && window.contains("[[package]]")
    {
        return Some(make_source_core(
            "cargo_lock",
            &["lock"],
            "sig.text.cargo_lock",
            Verification::Structure,
        ));
    }
    if window.contains("automatically @generated by Poetry") && window.contains("[[package]]") {
        return Some(make_source_core(
            "poetry_lock",
            &["lock"],
            "sig.text.poetry_lock",
            Verification::Structure,
        ));
    }

    // Emacs Org-mode (.org)
    if window.contains("#+TITLE:")
        || window.contains("#+AUTHOR:")
        || (window.contains("#+BEGIN_SRC") && window.contains("#+END_SRC"))
    {
        return Some(make_source_core(
            "org",
            &["org"],
            "sig.text.org",
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
    None
}
