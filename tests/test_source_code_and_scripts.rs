use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Identification, Verification};
use std::ffi::OsStr;

#[test]
fn test_detect_python_and_shell_shebang() {
    let opts = DetectOptions::default();

    // 1. Python with Shebang
    let py_shebang = b"#!/usr/bin/env python3\nimport sys\nprint('hello')\n";
    let rep_py = detect_bytes(py_shebang, &opts).expect("detect python shebang");
    assert_eq!(rep_py.identification, Identification::Format);
    assert_eq!(rep_py.format_id.as_deref(), Some("py"));
    assert_eq!(rep_py.verification, Verification::Signature);
    assert_eq!(
        compare_extension(&rep_py, Some(OsStr::new("py"))),
        ExtensionStatus::Match
    );

    // 2. Shell with Shebang
    let sh_shebang = b"#!/bin/bash\necho 'hello world'\n";
    let rep_sh = detect_bytes(sh_shebang, &opts).expect("detect shell shebang");
    assert_eq!(rep_sh.format_id.as_deref(), Some("sh"));
    assert_eq!(rep_sh.verification, Verification::Signature);
    assert_eq!(
        compare_extension(&rep_sh, Some(OsStr::new("sh"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_sh, Some(OsStr::new("bash"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_c_cpp_java_verilog_rust_go() {
    let opts = DetectOptions::default();

    // 1. C language
    let c_code = b"#include <stdio.h>\nint main() { printf(\"hello\\n\"); return 0; }\n";
    let rep_c = detect_bytes(c_code, &opts).expect("detect c");
    assert_eq!(rep_c.format_id.as_deref(), Some("c"));
    assert_eq!(
        compare_extension(&rep_c, Some(OsStr::new("c"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_c, Some(OsStr::new("h"))),
        ExtensionStatus::Match
    );

    // 2. C++ language
    let cpp_code = b"#include <iostream>\nnamespace test {\nclass MyClass {};\n}\n";
    let rep_cpp = detect_bytes(cpp_code, &opts).expect("detect cpp");
    assert_eq!(rep_cpp.format_id.as_deref(), Some("cpp"));
    assert_eq!(
        compare_extension(&rep_cpp, Some(OsStr::new("cpp"))),
        ExtensionStatus::Match
    );

    // 3. Java
    let java_code = b"package com.kindprobe;\npublic class App {\n    public static void main(String[] args) {}\n}\n";
    let rep_java = detect_bytes(java_code, &opts).expect("detect java");
    assert_eq!(rep_java.format_id.as_deref(), Some("java"));
    assert_eq!(
        compare_extension(&rep_java, Some(OsStr::new("java"))),
        ExtensionStatus::Match
    );

    // 4. Verilog HDL
    let verilog_code = b"module counter(input clk, output reg [3:0] q);\nalways @(posedge clk) q <= q + 1;\nendmodule\n";
    let rep_v = detect_bytes(verilog_code, &opts).expect("detect verilog");
    assert_eq!(rep_v.format_id.as_deref(), Some("v"));
    assert_eq!(
        compare_extension(&rep_v, Some(OsStr::new("v"))),
        ExtensionStatus::Match
    );

    // 5. Rust
    let rs_code = b"use std::io;\nfn main() {\n    println!(\"KindProbe\");\n}\n";
    let rep_rs = detect_bytes(rs_code, &opts).expect("detect rust");
    assert_eq!(rep_rs.format_id.as_deref(), Some("rs"));
    assert_eq!(
        compare_extension(&rep_rs, Some(OsStr::new("rs"))),
        ExtensionStatus::Match
    );

    // 6. Go
    let go_code = b"package main\nimport \"fmt\"\nfunc main() {\n    fmt.Println(\"Go\")\n}\n";
    let rep_go = detect_bytes(go_code, &opts).expect("detect go");
    assert_eq!(rep_go.format_id.as_deref(), Some("go"));
    assert_eq!(
        compare_extension(&rep_go, Some(OsStr::new("go"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_source_code_with_leading_comments_and_attributes() {
    let opts = DetectOptions::default();

    // 1. Rust with leading comment and struct
    let rs = b"// Copyright (c) 2026\n// Open source license\n\npub struct Config;\nimpl Config {\n    pub fn new() -> Self { Self }\n}\n";
    let rep_rs = detect_bytes(rs, &opts).expect("detect rs with comments");
    assert_eq!(rep_rs.format_id.as_deref(), Some("rs"));

    // 2. Java with license comment and import first
    let java = b"/* Multi-line license header */\nimport java.util.List;\npublic class Worker {}\n";
    let rep_java = detect_bytes(java, &opts).expect("detect java with import");
    assert_eq!(rep_java.format_id.as_deref(), Some("java"));

    // 3. C++ with leading comments and headers
    let cpp = b"// Precompiled header\n#include <vector>\nnamespace core {\ntemplate<typename T> class Pool {};\n}\n";
    let rep_cpp = detect_bytes(cpp, &opts).expect("detect cpp with comment");
    assert_eq!(rep_cpp.format_id.as_deref(), Some("cpp"));

    // 4. Python with leading docstring and function
    let py = b"\"\"\"Module documentation\"\"\"\nimport os\n\ndef run():\n    return 42\n";
    let rep_py = detect_bytes(py, &opts).expect("detect py with docstring");
    assert_eq!(rep_py.format_id.as_deref(), Some("py"));
}
