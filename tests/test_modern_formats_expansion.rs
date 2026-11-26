use std::io::{Cursor, Write};
use kindprobe::budget::DetectOptions;
use kindprobe::extension::find_registered_format;
use kindprobe::{detect_bytes, detect_bytes_core, Identification};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

fn assert_both_paths(data: &[u8], expected_fmt: &str) {
    let opts = DetectOptions::default();
    let rep = detect_bytes(data, &opts).expect("detect_bytes");
    let core = detect_bytes_core(data, &opts).expect("detect_bytes_core");
    assert_eq!(rep.identification, Identification::Format, "fmt={}", expected_fmt);
    assert_eq!(rep.format_id.as_deref(), Some(expected_fmt));
    assert_eq!(core.format_id, Some(expected_fmt));
    assert_eq!(core.identification, Identification::Format);
}

#[test]
fn test_modern_isobmff_images_avif_heic_heif() {
    let avif = b"\x00\x00\x00\x18ftypavif\x00\x00\x00\x00avifmif1miaf";
    let heic = b"\x00\x00\x00\x18ftypheic\x00\x00\x00\x00mif1heic";
    let heif = b"\x00\x00\x00\x18ftypmif1\x00\x00\x00\x00mif1heic";

    assert_both_paths(avif, "avif");
    assert_both_paths(heic, "heic");
    assert_both_paths(heif, "heif");
    assert_both_paths(b"\x00\x00\x00\x18ftypM4B \x00\x00\x00\x00M4B ", "m4b");
    assert_both_paths(b"\x00\x00\x00\x18ftyp3gp5\x00\x00\x00\x003gp5", "3gp");
    assert_both_paths(b"\x00\x00\x00\x18ftyp3g2a\x00\x00\x00\x003g2a", "3g2");
    assert_both_paths(b"\x00\x00\x00\x18ftypcrx \x00\x00\x00\x00crx ", "cr3");
}

#[test]
fn test_active_raw_scientific_asar_ova_and_npz() {
    let mut raf = b"FUJIFILMCCD-RAW ".to_vec();
    raf.extend_from_slice(&[0u8; 16]);
    assert_both_paths(&raf, "raf");

    let cr2 = b"II*\x00\x10\x00\x00\x00CR\x02\x00\x00\x00\x00\x00";
    assert_both_paths(cr2, "cr2");

    let npy = b"\x93NUMPY\x01\x00\x10\x00{'descr':'<f4'}";
    assert_both_paths(npy, "npy");

    let mut asar = Vec::new();
    asar.extend_from_slice(&4u32.to_le_bytes());
    asar.extend_from_slice(&40u32.to_le_bytes());
    asar.extend_from_slice(&36u32.to_le_bytes());
    asar.extend_from_slice(&32u32.to_le_bytes());
    asar.extend_from_slice(br#"{"files":{"app.js":{}}}"#);
    assert_both_paths(&asar, "asar");

    // OVA (POSIX TAR with .ovf first entry)
    let mut ova = vec![0u8; 512];
    let ovf_name = b"vm.ovf";
    ova[..ovf_name.len()].copy_from_slice(ovf_name);
    ova[257..263].copy_from_slice(b"ustar\0");
    assert_both_paths(&ova, "ova");

    // NPZ (ZIP containing .npy)
    let mut buf = Cursor::new(Vec::new());
    {
        let mut zw = ZipWriter::new(&mut buf);
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        zw.start_file("weights.npy", opts).expect("start");
        zw.write_all(npy).expect("write");
        zw.finish().expect("finish");
    }
    assert_both_paths(&buf.into_inner(), "npz");
}

#[test]
fn test_active_web_devops_and_language_formats() {
    assert_both_paths(
        b"<template><div>{{ msg }}</div></template>\n<script setup>\nconst msg = 'hi';\n</script>\n",
        "vue",
    );
    assert_both_paths(
        b"<script>\n  export let count = 0;\n  $: doubled = count * 2;\n</script>\n<button>{count}</button>\n",
        "svelte",
    );
    assert_both_paths(
        b"cmake_minimum_required(VERSION 3.20)\nproject(kindprobe LANGUAGES CXX)\nadd_executable(app main.cpp)\n",
        "cmake",
    );
    assert_both_paths(
        b".PHONY: all clean\nall:\n\tcargo build --release\nclean:\n\tcargo clean\n",
        "makefile",
    );
    assert_both_paths(
        b"terraform {\n  required_version = \">= 1.5\"\n}\nresource \"aws_s3_bucket\" \"b\" {\n  bucket = \"demo\"\n}\n",
        "hcl",
    );
    assert_both_paths(
        b"type User {\n  id: ID!\n  name: String!\n  age: Int\n}\n",
        "graphql",
    );
    assert_both_paths(
        b"const std = @import(\"std\");\npub fn main() void {\n    std.debug.print(\"hi\", .{});\n}\n",
        "zig",
    );
    assert_both_paths(
        b"import 'package:flutter/material.dart';\nvoid main() {\n  Widget w;\n}\n",
        "dart",
    );
    assert_both_paths(
        b"package com.example\ndata class User(val id: Int)\nfun greet(u: User) = println(u.id)\n",
        "kt",
    );
    assert_both_paths(
        b"package demo\ncase class Item(id: Int)\nobject Main {\n  def run(): Unit = ()\n}\n",
        "scala",
    );
    assert_both_paths(
        b"$primary: #333;\n@mixin card { padding: 12px; color: $primary; }\n.box { @include card; margin: 8px; }\n",
        "scss",
    );
}

#[test]
fn test_round3_appimage_php_proto_sql_lua_julia_nix_tsx_jsx() {
    let mut appimage = vec![0u8; 64];
    appimage[0..4].copy_from_slice(b"\x7FELF");
    appimage[8..11].copy_from_slice(b"AI\x02");
    assert_both_paths(&appimage, "appimage");

    assert_both_paths(b"<?php\necho 'hello';\n", "php");
    assert_both_paths(
        b"syntax = \"proto3\";\npackage demo;\nmessage Item { string id = 1; }\n",
        "proto",
    );
    assert_both_paths(
        b"CREATE TABLE users (\n  id INT PRIMARY KEY,\n  name TEXT\n);\nSELECT id FROM users;\n",
        "sql",
    );
    assert_both_paths(
        b"local function greet(name)\n  print('hi ' .. name)\nend\n",
        "lua",
    );
    assert_both_paths(
        b"using LinearAlgebra\nfunction norm2(x::Float64)\n  println(x)\nend\n",
        "jl",
    );
    assert_both_paths(
        b"{ pkgs ? import <nixpkgs> {} }:\npkgs.mkShell {\n  buildInputs = [ pkgs.rustc ];\n}\n",
        "nix",
    );
    assert_both_paths(
        b"import React from 'react';\ninterface Props { name: string }\nexport default function App({ name }: Props) {\n  return (<div>{name}</div>);\n}\n",
        "tsx",
    );
    assert_both_paths(
        b"import React from 'react';\nexport default function App() {\n  return (<div>Hello</div>);\n}\n",
        "jsx",
    );
}

#[test]
fn test_modern_ai_models_and_pst() {
    // GGUF v3
    let mut gguf = Vec::new();
    gguf.extend_from_slice(b"GGUF");
    gguf.extend_from_slice(&3u32.to_le_bytes());
    gguf.extend_from_slice(&[0u8; 16]);
    assert_both_paths(&gguf, "gguf");

    // Safetensors
    let header_json = br#"{"__metadata__":{"format":"pt"},"weight":{"dtype":"F16","shape":[4,4],"data_offsets":[0,32]}}"#;
    let mut safetensors = Vec::new();
    safetensors.extend_from_slice(&(header_json.len() as u64).to_le_bytes());
    safetensors.extend_from_slice(header_json);
    safetensors.extend_from_slice(&[0u8; 32]);
    assert_both_paths(&safetensors, "safetensors");

    // TFLite (FlatBuffer file_identifier "TFL3" at offset 4)
    let tflite = b"\x1c\x00\x00\x00TFL3\x00\x00\x00\x00\x00\x00";
    assert_both_paths(tflite, "tflite");

    // Outlook PST / OST
    let pst = b"!BDN\x00\x00\x00\x00SM\x13\x00\x01\x01\x00\x00";
    assert_both_paths(pst, "pst");
}

#[test]
fn test_modern_aab_zip_package() {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut zw = ZipWriter::new(&mut buf);
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        zw.start_file("BundleConfig.pb", opts).expect("start");
        zw.write_all(b"\x0a\x051.0.0").expect("write");
        zw.start_file("base/manifest/AndroidManifest.xml", opts)
            .expect("start2");
        zw.write_all(b"\x03\x00\x08\x00").expect("write2");
        zw.finish().expect("finish");
    }
    let bytes = buf.into_inner();
    assert_both_paths(&bytes, "aab");
}

#[test]
fn test_modern_text_eml_ini_dockerfile_css() {
    let eml = b"From: alice@example.com\r\nTo: bob@example.com\r\nSubject: Hello KindProbe\r\nMIME-Version: 1.0\r\n\r\nBody text.\r\n";
    assert_both_paths(eml, "eml");

    let ini = b"; Application configuration\n[database]\nhost = 127.0.0.1\nport = 5432\n";
    assert_both_paths(ini, "ini");

    let dockerfile = b"# Build stage\nFROM rust:1.85-alpine\nWORKDIR /app\nCOPY . .\nRUN cargo build --release\nCMD [\"./kindprobe\"]\n";
    assert_both_paths(dockerfile, "dockerfile");

    let css = b"body {\n  margin: 0;\n  padding: 16px;\n  color: #222;\n  display: flex;\n}\n";
    assert_both_paths(css, "css");
}

#[test]
fn test_new_extensions_registered() {
    for (ext, fid) in [
        ("aab", "aab"),
        ("avif", "avif"),
        ("css", "css"),
        ("dockerfile", "dockerfile"),
        ("eml", "eml"),
        ("gguf", "gguf"),
        ("heic", "heic"),
        ("heif", "heif"),
        ("ini", "ini"),
        ("ost", "pst"),
        ("pst", "pst"),
        ("safetensors", "safetensors"),
        ("tflite", "tflite"),
    ] {
        assert_eq!(find_registered_format(ext), Some(fid));
    }
}
