use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::time::Instant;

use kindprobe::budget::DetectOptions;
use kindprobe::model::Identification;
use kindprobe::{detect_bytes, detect_reader};

// Helper to synthesize CFB Excel
fn create_cfb_xls(stream_name: &str, biff_version: u16) -> Vec<u8> {
    let cursor = Cursor::new(Vec::new());
    let mut comp = cfb::CompoundFile::create(cursor).expect("create cfb");
    let mut bof = [0u8; 16];
    bof[0..2].copy_from_slice(&0x0809u16.to_le_bytes());
    bof[2..4].copy_from_slice(&16u16.to_le_bytes());
    bof[4..6].copy_from_slice(&biff_version.to_le_bytes());
    bof[6..8].copy_from_slice(&0x0005u16.to_le_bytes());
    {
        let mut stream = comp.create_stream(format!("/{}", stream_name)).expect("create stream");
        stream.write_all(&bof).expect("write bof");
        stream.write_all(&[0u8; 64]).expect("write padding");
    }
    comp.into_inner().into_inner()
}

// Helper to synthesize CFB PowerPoint
fn create_cfb_ppt(has_current_user: bool) -> Vec<u8> {
    let cursor = Cursor::new(Vec::new());
    let mut comp = cfb::CompoundFile::create(cursor).expect("create cfb");
    {
        let mut ppt_data = vec![0xAAu8; 64];
        ppt_data[0..2].copy_from_slice(&0x000Fu16.to_le_bytes());
        ppt_data[2..4].copy_from_slice(&0x03E8u16.to_le_bytes());
        ppt_data[4..8].copy_from_slice(&56u32.to_le_bytes());
        let mut stream = comp.create_stream("/PowerPoint Document").expect("create stream");
        stream.write_all(&ppt_data).expect("write ppt");
    }
    if has_current_user {
        let mut cu = [0u8; 24];
        cu[2..4].copy_from_slice(&0x0FF6u16.to_le_bytes());
        cu[4..8].copy_from_slice(&20u32.to_le_bytes());
        cu[8..12].copy_from_slice(&20u32.to_le_bytes());
        cu[12..16].copy_from_slice(&0xE391_C05Fu32.to_le_bytes());
        let mut user = comp.create_stream("/Current User").expect("create user");
        user.write_all(&cu).expect("write user");
    }
    comp.into_inner().into_inner()
}

#[test]
fn test_anti_overfitting_comprehensive_benchmark() {
    let opts = DetectOptions::default();

    println!("\n==================================================================");
    println!("     KindProbe Anti-Overfitting & Diverse Benchmark Suite         ");
    println!("==================================================================");

    // =========================================================================
    // Part 1: Real-World Mixed Dynamic Workload (55 Distinct Formats in Shuffle)
    // =========================================================================
    // Solves the L1-cache / CPU branch predictor overfitting problem of micro-benchmarks
    let mixed_samples: Vec<(&'static str, Vec<u8>)> = vec![
        ("png", b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15c4".to_vec()),
        ("jpeg", b"\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01\x01\x01\x00`\x00`\x00\x00\xff\xdb\x00C\x00".to_vec()),
        ("gif", b"GIF89a\x01\x00\x01\x00\x80\x00\x00\x00\x00\x00\xff\xff\xff!\xf9\x04\x01\x00\x00\x00\x00,\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x02D\x01\x00;".to_vec()),
        ("webp", b"RIFF\x24\x00\x00\x00WEBPVP8 \x18\x00\x00\x00\x30\x01\x00\x9d\x01\x2a\x01\x00\x01\x00\x02\x00\x34\x25\xa4\x00\x03\x70\x00\xfe\xfb\xfd\x50\x00".to_vec()),
        ("pdf", b"%PDF-1.7\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\nxref\n0 2\ntrailer\n<< /Root 1 0 R >>\n%%EOF\n".to_vec()),
        ("zip", b"PK\x03\x04\x14\x00\x00\x00\x08\x00\x00\x00!\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x04\x00\x00\x00test\x00\x00PK\x01\x02\x14\x00\x14\x00\x00\x00\x08\x00\x00\x00!\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00testPK\x05\x06\x00\x00\x00\x00\x01\x00\x01\x002\x00\x00\x00&\x00\x00\x00\x00\x00".to_vec()),
        ("tar", {
            let mut v = vec![0u8; 512];
            v[0..4].copy_from_slice(b"file");
            v[257..263].copy_from_slice(b"ustar\x00");
            v
        }),
        ("gz", b"\x1f\x8b\x08\x08\x00\x00\x00\x00\x02\x03test.txt\x00\x0b\xc9\xc8\x2c\x56\x00\xa2\x44\x85\x92\xd4\xe2\x12\x00\x2b\x6f\x9a\x9d\x0e\x00\x00\x00".to_vec()),
        ("bz2", b"BZh91AY&SY\x94\x4b\x8d\x00\x00\x00\x01\x00\x80\x00\x20\x00\x21\x9a\x68\x33\x4d\x04\x04\x80".to_vec()),
        ("xz", b"\xfd7zXZ\x00\x00\x01\x69\x22\xde\x36\x02".to_vec()),
        ("7z", b"7z\xbc\xaf\x27\x1c\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec()),
        ("rar", b"Rar!\x1a\x07\x00\xcf\x90\x73\x00\x00\x0d\x00\x00\x00\x00\x00\x00\x00".to_vec()),
        ("elf", b"\x7fELF\x02\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02\x00\x3e\x00\x01\x00\x00\x00\x40\x00\x00\x00\x00\x00\x00\x00".to_vec()),
        ("pe", {
            let mut v = vec![0u8; 1024];
            v[0..2].copy_from_slice(b"MZ");
            v[0x3C..0x40].copy_from_slice(&0x80u32.to_le_bytes());
            v[0x80..0x84].copy_from_slice(b"PE\x00\x00");
            v
        }),
        ("mach_o", b"\xcf\xfa\xed\xfe\x07\x00\x00\x01\x03\x00\x00\x80\x02\x00\x00\x00\x04\x00\x00\x00\x20\x02\x00\x00".to_vec()),
        ("wasm", b"\x00asm\x01\x00\x00\x00\x01\x05\x01\x60\x00\x01\x7f".to_vec()),
        ("sqlite", b"SQLite format 3\x00\x10\x00\x01\x01\x00\x40\x20\x20\x00\x00\x00\x00".to_vec()),
        ("pcap", b"\xd4\xc3\xb2\xa1\x02\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x04\x00\x01\x00\x00\x00".to_vec()),
        ("flac", b"fLaC\x00\x00\x00\x22\x10\x00\x10\x00\x00\x00\x00\x00\x00\x00\x0b\xb8".to_vec()),
        ("ogg", b"OggS\x00\x02\x00\x00\x00\x00\x00\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00\x01\x1e\x01\x76\x6f\x72\x62\x69\x73".to_vec()),
        ("mp3", b"ID3\x04\x00\x00\x00\x00\x00\x20TIT2\x00\x00\x00\x0c\x00\x00\x00Test Track\x00".to_vec()),
        ("mp4", b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00isommp42".to_vec()),
        ("blend", b"BLENDER_v280\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec()),
        ("h5", b"\x89HDF\r\n\x1a\n\x00\x00\x00\x00\x00\x08\x08\x00\x04\x00\x10\x00\x00\x00".to_vec()),
        ("fits", b"SIMPLE  =                    T / file does conform to FITS standard             BITPIX  =                   16 / number of bits per data pixel                  NAXIS   =                    2 / number of data axes                            END                                                                             ".to_vec()),
        ("dicom", {
            let mut v = vec![0u8; 132];
            v[128..132].copy_from_slice(b"DICM");
            v
        }),
        ("can_sym", b"FormatVersion=5.0 // CAN Matrix symbol file\nTitle=\"Test CAN Network\"\n".to_vec()),
        ("sharp_xdf", b"XDF\x00\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec()),
        ("rust", b"// SPDX-License-Identifier: MIT\n//! KindProbe Rust module\npub fn calculate() -> u32 {\n    42\n}\n".to_vec()),
        ("python", b"#!/usr/bin/env python3\n\"\"\"Module docstring\"\"\"\nimport os, sys\n\ndef main():\n    return 42\n".to_vec()),
        ("c_source", b"/* License notice */\n#include <stdio.h>\n#include <stdlib.h>\n\nint main(void) {\n    printf(\"Hello\\n\");\n    return 0;\n}\n".to_vec()),
        ("cpp_source", b"// C++ Stream source\n#include <iostream>\n#include <vector>\n\ntemplate<typename T>\nclass Buffer {};\n".to_vec()),
        ("java", b"// Java Source\npackage com.example.probe;\nimport java.util.List;\n\npublic class App {\n    public static void main(String[] args) {}\n}\n".to_vec()),
        ("go", b"// Go Source\npackage main\n\nimport \"fmt\"\n\nfunc main() {\n    fmt.Println(\"Hello\")\n}\n".to_vec()),
        ("javascript", b"/** @license Apache-2.0 */\nimport React from 'react';\n\nexport default function App() {\n    return <div>Hello</div>;\n}\n".to_vec()),
        ("html", b"<!DOCTYPE html>\n<html lang=\"en\">\n<head><title>KindProbe</title></head>\n<body><h1>Test</h1></body>\n</html>\n".to_vec()),
        ("xml", b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<root id=\"100\">\n  <entry name=\"test\"/>\n</root>\n".to_vec()),
        ("json", b"{\n  \"engine\": \"KindProbe\",\n  \"version\": \"1.0.0\",\n  \"formats\": 1006,\n  \"active\": true\n}\n".to_vec()),
        ("yaml", b"---\nname: KindProbe\nversion: 1.0.0\nenabled: true\ntags:\n  - fast\n  - safe\n".to_vec()),
        ("toml", b"[package]\nname = \"kindprobe\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nquick-xml = \"0.37\"\n".to_vec()),
        ("markdown", b"# KindProbe Project\n\nThis is a **high-performance** file probe engine supporting *1006* formats.\n".to_vec()),
        ("csv", b"id,format,latency,throughput\n1,png,337,2965\n2,unknown,225,4432\n3,doc,7020,142516\n".to_vec()),
        ("tsv", b"id\tformat\tscore\n1\tRust\t99.5\n2\tGo\t98.0\n3\tPython\t95.0\n".to_vec()),
        ("shell", b"#!/usr/bin/env bash\n# Shell script\nset -euo pipefail\necho \"Starting task\"\nexit 0\n".to_vec()),
        ("verilog", b"// Hardware module\nmodule counter (input clk, rst, output reg [3:0] q);\n  always @(posedge clk) q <= q + 1;\nendmodule\n".to_vec()),
        ("sql", b"-- Database migration\nCREATE TABLE detection_logs (\n    id BIGSERIAL PRIMARY KEY,\n    format_id VARCHAR(64) NOT NULL\n);\n".to_vec()),
        ("css", b"/* Stylesheet */\n:root {\n    --primary-color: #007acc;\n}\nbody {\n    margin: 0;\n    font-family: sans-serif;\n}\n".to_vec()),
        ("diff", b"--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,3 +1,3 @@\n-pub fn old();\n+pub fn new();\n".to_vec()),
        ("ini", b"[server]\nport = 8080\nhost = \"0.0.0.0\"\nworkers = 4\n".to_vec()),
        ("near_miss_zip", b"PK\x03\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec()),
        ("near_miss_png", b"\x89PNG\r\n\x1a\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec()),
        ("near_miss_pdf", b"%PDF-corrupted_trailer_without_any_valid_objects_here".to_vec()),
        ("near_miss_elf", b"\x7fELF\x03\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec()),
        ("unknown_random_64", (0..64).map(|i| (i * 37 + 13) as u8).collect()),
        ("unknown_random_1k", (0..1024).map(|i| (i * 101 + 7) as u8).collect()),
        ("unknown_zeros", vec![0u8; 512]),
    ];

    let total_samples = mixed_samples.len();
    let rounds = 2_000;
    let total_operations = total_samples * rounds;

    println!("[1] Dynamic Mixed-Stream Detection (55 Formats Interleaved):");
    println!("    Sample Pool Size:     {} distinct formats/payloads", total_samples);
    println!("    Interleaved Rounds:   {} rounds", rounds);
    println!("    Total Operations:     {} detections", total_operations);

    let start_mixed = Instant::now();
    for _ in 0..rounds {
        for (_name, data) in &mixed_samples {
            let _ = detect_bytes(data, &opts);
        }
    }
    let elapsed_mixed = start_mixed.elapsed();
    let ns_per_op_mixed = elapsed_mixed.as_nanos() as f64 / total_operations as f64;
    let qps_mixed = total_operations as f64 / elapsed_mixed.as_secs_f64();

    println!("    Mixed Stream Latency:    {:.2} ns / detection", ns_per_op_mixed);
    println!("    Mixed Stream Throughput: {:.2} thousand files / sec", qps_mixed / 1000.0);

    // =========================================================================
    // Part 2: Complex Real-World Source Code with Multi-Line Preamble Permutations
    // =========================================================================
    println!("\n[2] Complex Real-World Code Permutations (Anti-Overfitting Preambles):");
    let complex_text_cases: Vec<(&'static str, &'static str, Vec<u8>)> = vec![
        ("Python with 30-line Apache License & imports", "py", {
            let mut s = String::new();
            s.push_str("# Copyright 2026 The Authors\n");
            s.push_str("# Licensed under the Apache License, Version 2.0 (the \"License\");\n");
            for i in 0..25 {
                s.push_str(&format!("# Comment line {}\n", i));
            }
            s.push_str("\n\nimport os\nimport sys\nfrom typing import Optional, Dict\n\n@dataclass\nclass Config:\n    name: str = 'test'\n");
            s.into_bytes()
        }),
        ("Java with package, 20 imports and Javadoc", "java", {
            let mut s = String::new();
            s.push_str("/**\n * Enterprise Java Application\n * Multi-line Javadoc documentation\n */\n");
            s.push_str("package com.enterprise.service.impl;\n\n");
            for i in 0..20 {
                s.push_str(&format!("import org.springframework.beans.factory.annotation.Import{};\n", i));
            }
            s.push_str("\npublic class ServiceImpl {\n    public void execute() {}\n}\n");
            s.into_bytes()
        }),
        ("Rust with module docs, inner attributes and crates", "rs", {
            let mut s = String::new();
            s.push_str("//! High performance probe library\n//! Crate level documentation\n\n");
            s.push_str("#![allow(dead_code)]\n#![warn(missing_docs)]\n\n");
            s.push_str("use std::collections::HashMap;\nuse std::io::{Read, Write};\n\npub struct Engine {\n    pub version: u32,\n}\n");
            s.into_bytes()
        }),
        ("C++ with Header Guard, Namespaces and Templates", "cpp", {
            let mut s = String::new();
            s.push_str("/* C++ Header Guard File */\n#ifndef KINDPROBE_CORE_H\n#define KINDPROBE_CORE_H\n\n");
            s.push_str("#include <vector>\n#include <string>\n#include <memory>\n\n");
            s.push_str("namespace kindprobe::core {\n  template<typename T>\n  class Engine {};\n}\n#endif\n");
            s.into_bytes()
        }),
        ("Verilog with Timescale, Parameters and Comments", "v", {
            let mut s = String::new();
            s.push_str("// EDA Standard Simulation\n`timescale 1ns / 1ps\n\n");
            s.push_str("module fifo_buffer #(\n    parameter DATA_WIDTH = 32,\n    parameter ADDR_WIDTH = 8\n) (\n    input wire clk\n);\nendmodule\n");
            s.into_bytes()
        }),
    ];

    let text_iterations = 20_000;
    for (desc, expected_ext, data) in &complex_text_cases {
        let rep = detect_bytes(data, &opts).expect("detect complex text");
        let detected = rep.recommended_extension.as_deref().unwrap_or("none");
        let status = if detected == *expected_ext { "MATCH" } else { "MISMATCH" };
        assert_eq!(detected, *expected_ext, "Complex code must be correctly identified");

        let start = Instant::now();
        for _ in 0..text_iterations {
            let _ = detect_bytes(data, &opts);
        }
        let dur = start.elapsed();
        let ns = dur.as_nanos() as f64 / text_iterations as f64;
        println!("    {:<50}: {:6.2} ns (Ext: {:<4} [{}])", desc, ns, detected, status);
    }

    // =========================================================================
    // Part 3: Adversarial Near-Miss Resistance Matrix (Rejection Latency)
    // =========================================================================
    println!("\n[3] Adversarial Near-Miss Rejection Matrix (Fast Negative Bailout):");
    let near_misses: Vec<(&'static str, Vec<u8>, &'static str)> = vec![
        ("Near-Miss ZIP (PK\\x03\\x09)", b"PK\x03\x09\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(), "zip"),
        ("Near-Miss PNG (\\x89PNG\\r\\n\\x1a\\x00)", b"\x89PNG\r\n\x1a\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(), "png"),
        ("Near-Miss PDF (%PDG-1.5)", b"%PDG-1.5 fake pdf stream without catalog".to_vec(), "pdf"),
        ("Near-Miss ELF (\\x7fEL\\x00)", b"\x7fEL\x00\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(), "elf"),
        ("Near-Miss CFB (D0CF11E0A1B11A00)", b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\x00\x00\x00\x00\x00".to_vec(), "cfb"),
        ("Near-Miss GIF (GIF87x)", b"GIF87x\x01\x00\x01\x00\x80\x00\x00".to_vec(), "gif"),
        ("Near-Miss JPEG (FFD80000)", b"\xff\xd8\x00\x00\x10JFIF\x00".to_vec(), "jpeg"),
        ("Near-Miss JSON ({ not_quoted)", b"{\n  not_quoted_key: 123\n}".to_vec(), "json"),
    ];

    let miss_iterations = 50_000;
    for (desc, data, forbidden) in &near_misses {
        let rep = detect_bytes(data, &opts).expect("detect near miss");
        assert_ne!(
            rep.format_id.as_deref(),
            Some(*forbidden),
            "Near miss for '{}' must not produce false positive '{}'",
            desc,
            forbidden
        );

        let start = Instant::now();
        for _ in 0..miss_iterations {
            let _ = detect_bytes(data, &opts);
        }
        let dur = start.elapsed();
        let ns = dur.as_nanos() as f64 / miss_iterations as f64;
        let qps = miss_iterations as f64 / dur.as_secs_f64();
        println!("    {:<35}: {:6.2} ns ({:.2} M refusals/sec)", desc, ns, qps / 1_000_000.0);
    }

    // =========================================================================
    // Part 4: Variable Payload Size Scaling
    // =========================================================================
    println!("\n[4] Payload Size Scaling (Unknown Binary Stress):");
    let sizes = [16, 64, 512, 4096, 65536, 262144];
    let size_iterations = 20_000;
    for &sz in &sizes {
        let data: Vec<u8> = (0..sz).map(|i| (i * 71 + 3) as u8).collect();
        let start = Instant::now();
        for _ in 0..size_iterations {
            let _ = detect_bytes(&data, &opts);
        }
        let dur = start.elapsed();
        let ns = dur.as_nanos() as f64 / size_iterations as f64;
        let mb_per_sec = (sz as f64 * size_iterations as f64) / dur.as_secs_f64() / (1024.0 * 1024.0);
        println!("    Payload {:>6} bytes: {:6.2} ns ({:8.2} MB/s scan speed)", sz, ns, mb_per_sec);
    }

    // =========================================================================
    // Part 5: Diverse Container Real & Synthetic Fixtures
    // =========================================================================
    println!("\n[5] Diverse Container Parsing (DOCX, Strict DOCX, DOC, XLS, PPT):");
    let fixtures_dir = Path::new("fixtures");
    let docx_path = fixtures_dir.join("true_docx.docx");
    let docx_strict_path = fixtures_dir.join("true_docx_strict.docx");
    let doc_path = fixtures_dir.join("true_doc.doc");

    let mut containers: Vec<(&'static str, Vec<u8>)> = Vec::new();
    if docx_path.exists() {
        let mut buf = Vec::new();
        File::open(&docx_path).unwrap().read_to_end(&mut buf).unwrap();
        containers.push(("Transitional DOCX", buf));
    }
    if docx_strict_path.exists() {
        let mut buf = Vec::new();
        File::open(&docx_strict_path).unwrap().read_to_end(&mut buf).unwrap();
        containers.push(("Strict ISO/IEC 29500 DOCX", buf));
    }
    if doc_path.exists() {
        let mut buf = Vec::new();
        File::open(&doc_path).unwrap().read_to_end(&mut buf).unwrap();
        containers.push(("MS-DOC WordDocument CFB", buf));
    }
    containers.push(("MS-XLS Workbook (BIFF8) CFB", create_cfb_xls("Workbook", 0x0600)));
    containers.push(("MS-XLS Book (BIFF5) CFB", create_cfb_xls("Book", 0x0500)));
    containers.push(("MS-PPT PowerPoint CFB", create_cfb_ppt(true)));

    let container_iter = 3_000;
    for (desc, buf) in &containers {
        let mut cur = Cursor::new(buf);
        let rep = detect_reader(&mut cur, &opts).expect("detect container");
        assert_eq!(rep.identification, Identification::Format, "Must confirm container format");

        let start = Instant::now();
        for _ in 0..container_iter {
            let mut cur = Cursor::new(buf);
            let _ = detect_reader(&mut cur, &opts);
        }
        let dur = start.elapsed();
        let us = dur.as_micros() as f64 / container_iter as f64;
        let qps = container_iter as f64 / dur.as_secs_f64();
        println!("    {:<32}: {:6.2} μs / doc ({:6.0} docs/sec)", desc, us, qps);
    }

    println!("==================================================================\n");
}
