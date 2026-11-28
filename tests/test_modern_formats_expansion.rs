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
fn test_round4_dmg_utf16_reg_and_powershell() {
    // DMG 512-byte koly trailer
    let mut dmg = vec![0u8; 1024];
    let t_off = dmg.len() - 512;
    dmg[t_off..t_off + 4].copy_from_slice(b"koly");
    dmg[t_off + 4..t_off + 8].copy_from_slice(&4u32.to_be_bytes());
    assert_both_paths(&dmg, "dmg");
    let mut cur = Cursor::new(dmg);
    let rep_reader = kindprobe::detect_reader(&mut cur, &DetectOptions::default()).expect("dmg reader");
    assert_eq!(rep_reader.format_id.as_deref(), Some("dmg"));

    // UTF-16 LE BOM plain text
    let mut utf16_txt = vec![0xFF, 0xFE];
    for unit in "Hello UTF-16 World\r\nLine 2\r\n".encode_utf16() {
        utf16_txt.extend_from_slice(&unit.to_le_bytes());
    }
    assert_both_paths(&utf16_txt, "txt");

    // UTF-16 LE BOM Windows .reg
    let mut utf16_reg = vec![0xFF, 0xFE];
    for unit in "Windows Registry Editor Version 5.00\r\n\r\n[HKEY_CURRENT_USER\\Software\\KindProbe]\r\n".encode_utf16() {
        utf16_reg.extend_from_slice(&unit.to_le_bytes());
    }
    assert_both_paths(&utf16_reg, "reg");

    // ASCII REGEDIT4 & PowerShell
    assert_both_paths(b"REGEDIT4\r\n\r\n[HKEY_LOCAL_MACHINE\\SOFTWARE\\Test]\r\n", "reg");
    assert_both_paths(
        b"[CmdletBinding()]\nparam([string]$Path = $PSScriptRoot)\nWrite-Host \"Scanning $Path\"\n",
        "ps1",
    );
}

#[test]
fn test_round5_raw_onnx_cs_swift_r_env_gitignore() {
    let mut nef = b"II*\x00\x08\x00\x00\x00\x00\x00\x00\x00".to_vec();
    nef.extend_from_slice(b"NIKON CORPORATION\x00NIKON D850");
    assert_both_paths(&nef, "nef");

    let mut arw = b"II*\x00\x08\x00\x00\x00\x00\x00\x00\x00".to_vec();
    arw.extend_from_slice(b"SONY\x00ILCE-7M4");
    assert_both_paths(&arw, "arw");

    let mut dng = b"II*\x00\x08\x00\x00\x00\x00\x00\x00\x00".to_vec();
    dng.extend_from_slice(b"Adobe DNG Converter");
    assert_both_paths(&dng, "dng");

    let mut onnx = vec![0x08, 0x09, 0x12, 0x07];
    onnx.extend_from_slice(b"pytorch");
    onnx.extend_from_slice(b"\x1a\x04onnx\x00\x00\x00\x00");
    assert_both_paths(&onnx, "onnx");

    assert_both_paths(
        b"using System;\nnamespace Demo {\n  class Program { static void Main() {} }\n}\n",
        "cs",
    );
    assert_both_paths(
        b"import Foundation\nstruct User { let id: Int }\nfunc greet() {}\n",
        "swift",
    );
    assert_both_paths(
        b"library(ggplot2)\ndf <- data.frame(x = 1:10, y = 2:11)\n",
        "r",
    );
    assert_both_paths(
        b"DATABASE_URL=postgres://localhost/app\nPORT=8080\nRUST_LOG=info\n",
        "env",
    );
    assert_both_paths(
        b"target/\nnode_modules/\n.DS_Store\n*.log\n",
        "gitignore",
    );
}

#[test]
fn test_round6_lnk_mxf_sln_csproj_tex_rst_adoc_gradle_nginx() {
    let lnk = b"\x4C\x00\x00\x00\x01\x14\x02\x00\x00\x00\x00\x00\xC0\x00\x00\x00\x00\x00\x00\x46\x00\x00\x00\x00";
    assert_both_paths(lnk, "lnk");

    let mxf = b"\x06\x0E\x2B\x34\x02\x05\x01\x01\x0D\x01\x02\x01\x01\x02\x04\x00";
    assert_both_paths(mxf, "mxf");

    assert_both_paths(
        b"Microsoft Visual Studio Solution File, Format Version 12.00\r\n# Visual Studio Version 17\r\n",
        "sln",
    );
    assert_both_paths(
        b"<Project Sdk=\"Microsoft.NET.Sdk\">\n  <PropertyGroup>\n    <TargetFramework>net8.0</TargetFramework>\n  </PropertyGroup>\n</Project>\n",
        "csproj",
    );
    assert_both_paths(
        b"\\documentclass{article}\n\\usepackage{amsmath}\n\\begin{document}\nHello\n\\end{document}\n",
        "tex",
    );
    assert_both_paths(
        b"Title\n=====\n\n.. code-block:: rust\n\n   fn main() {}\n",
        "rst",
    );
    assert_both_paths(
        b"= KindProbe Guide\n\n== Installation\n\n[source,bash]\n----\ncargo install kindprobe\n----\n",
        "adoc",
    );
    assert_both_paths(
        b"plugins {\n    id 'java'\n}\ndependencies {\n    implementation 'com.google.guava:guava:33.0.0-jre'\n}\n",
        "gradle",
    );
    assert_both_paths(
        b"server {\n    listen 80;\n    server_name example.com;\n    location / {\n        return 200 'ok';\n    }\n}\n",
        "nginx",
    );
}

#[test]
fn test_round7_ipynb_gltf_har_lottie_prisma_sol_hs_ex_astro() {
    assert_both_paths(
        br#"{"cells":[{"cell_type":"code","source":["print(1)"]}],"metadata":{},"nbformat":4,"nbformat_minor":5}"#,
        "ipynb",
    );
    assert_both_paths(
        br#"{"asset":{"version":"2.0"},"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}]}"#,
        "gltf",
    );
    assert_both_paths(
        br#"{"log":{"version":"1.2","creator":{"name":"WebInspector","version":"537.36"},"entries":[]}}"#,
        "har",
    );
    assert_both_paths(
        br#"{"v":"5.7.4","fr":60,"ip":0,"op":120,"w":512,"h":512,"layers":[]}"#,
        "lottie",
    );
    assert_both_paths(
        b"datasource db {\n  provider = \"postgresql\"\n  url = env(\"DATABASE_URL\")\n}\nmodel User {\n  id Int @id\n}\n",
        "prisma",
    );
    assert_both_paths(
        b"// SPDX-License-Identifier: MIT\npragma solidity ^0.8.20;\ncontract Counter {\n    uint256 public count;\n}\n",
        "sol",
    );
    assert_both_paths(
        b"module Main where\n\nimport Data.List (sort)\n\nmain :: IO ()\nmain = putStrLn \"hello\"\n",
        "hs",
    );
    assert_both_paths(
        b"defmodule Greeter do\n  def hello(name) do\n    \"Hello, #{name}\"\n  end\nend\n",
        "ex",
    );
    assert_both_paths(
        b"---\nimport Layout from '../layouts/Layout.astro';\nconst title = 'KindProbe';\n---\n<Layout title={title}><h1>Hi</h1></Layout>\n",
        "astro",
    );
}

#[test]
fn test_round8_pf_jsonl_rss_atom_opml_xslt_xsd_clj_erl_nim() {
    assert_both_paths(b"MAM\x04\x00\x10\x00\x00\x00\x00\x00\x00", "pf");
    assert_both_paths(b"{\"id\":1,\"ok\":true}\n{\"id\":2,\"ok\":false}\n", "jsonl");
    assert_both_paths(
        b"<?xml version=\"1.0\"?><rss version=\"2.0\"><channel><title>Feed</title></channel></rss>",
        "rss",
    );
    assert_both_paths(
        b"<?xml version=\"1.0\"?><feed xmlns=\"http://www.w3.org/2005/Atom\"><title>Atom</title></feed>",
        "atom",
    );
    assert_both_paths(
        b"<?xml version=\"1.0\"?><opml version=\"2.0\"><body><outline text=\"News\"/></body></opml>",
        "opml",
    );
    assert_both_paths(
        b"<?xml version=\"1.0\"?><xsl:stylesheet version=\"1.0\" xmlns:xsl=\"http://www.w3.org/1999/XSL/Transform\"></xsl:stylesheet>",
        "xslt",
    );
    assert_both_paths(
        b"<?xml version=\"1.0\"?><xs:schema xmlns:xs=\"http://www.w3.org/2001/XMLSchema\"></xs:schema>",
        "xsd",
    );
    assert_both_paths(
        b"(ns kindprobe.core\n  (:require [clojure.string :as str]))\n\n(defn greet [name]\n  (str \"Hello \" name))\n",
        "clj",
    );
    assert_both_paths(
        b"-module(hello).\n-export([world/0]).\n\nworld() -> ok.\n",
        "erl",
    );
    assert_both_paths(
        b"proc greet(name: string): string =\n  result = \"Hello \" & name\n  echo result\n",
        "nim",
    );
}

#[test]
fn test_round9_crx_whl_vsix_nupkg_xpi_tfstate_ris_bib_fs() {
    assert_both_paths(b"Cr24\x03\x00\x00\x00\x10\x00\x00\x00", "crx");

    let make_zip = |entries: &[(&str, &[u8])]| -> Vec<u8> {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut zw = ZipWriter::new(&mut buf);
            let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            for (name, data) in entries {
                zw.start_file(*name, opts).expect("start");
                zw.write_all(data).expect("write");
            }
            zw.finish().expect("finish");
        }
        buf.into_inner()
    };

    assert_both_paths(
        &make_zip(&[("demo-1.0.dist-info/WHEEL", b"Wheel-Version: 1.0\n")]),
        "whl",
    );
    assert_both_paths(
        &make_zip(&[("extension.vsixmanifest", b"<PackageManifest/>")]),
        "vsix",
    );
    assert_both_paths(
        &make_zip(&[("KindProbe.nuspec", b"<package><metadata/></package>")]),
        "nupkg",
    );
    assert_both_paths(
        &make_zip(&[("install.rdf", b"<RDF/>")]),
        "xpi",
    );

    assert_both_paths(
        br#"{"version":4,"terraform_version":"1.8.0","resources":[{"mode":"managed","type":"aws_s3_bucket"}]}"#,
        "tfstate",
    );
    assert_both_paths(
        b"TY  - JOUR\nTI  - KindProbe Paper\nAU  - Doe, J.\nER  - \n",
        "ris",
    );
    assert_both_paths(
        b"@article{kindprobe2026,\n  author = {Doe, Jane},\n  title = {Fast Format Probing},\n  year = {2026}\n}\n",
        "bib",
    );
    assert_both_paths(
        b"open System\n\n[<EntryPoint>]\nlet main argv =\n    printfn \"Hello\"\n    0\n",
        "fs",
    );
}

#[test]
fn test_round10_pfx_openapi_vb_ml_groovy_tcl_awk_wat_vhdl() {
    assert_both_paths(
        b"\x30\x82\x01\x00\x02\x01\x03\x30\x82\x00\x80\x06\x09\x2A\x86\x48\x86\xF7\x0D\x01\x07\x01\x00\x00",
        "pfx",
    );
    assert_both_paths(
        br#"{"openapi":"3.1.0","info":{"title":"KindProbe API","version":"1.0"},"paths":{"/probe":{}}}"#,
        "openapi",
    );
    assert_both_paths(
        b"Imports System\n\nModule Program\n    Sub Main(args As String())\n        Console.WriteLine(\"Hello\")\n    End Sub\nEnd Module\n",
        "vb",
    );
    assert_both_paths(
        b"let rec fact n =\n  if n <= 1 then 1 else n * fact (n - 1);;\n",
        "ml",
    );
    assert_both_paths(
        b"def name = 'KindProbe'\nprintln \"Hello $name\"\n",
        "groovy",
    );
    assert_both_paths(
        b"proc greet {name} {\n    puts \"Hello $name\"\n}\ngreet World\n",
        "tcl",
    );
    assert_both_paths(
        b"BEGIN { total = 0 }\n{ total += $1 }\nEND { print total }\n",
        "awk",
    );
    assert_both_paths(
        b"(module\n  (func (export \"add\") (param i32 i32) (result i32)\n    local.get 0\n    local.get 1\n    i32.add))\n",
        "wat",
    );
    assert_both_paths(
        b"library ieee;\nuse ieee.std_logic_1164.all;\n\nentity counter is\n    port (clk : in std_logic);\nend entity;\n",
        "vhdl",
    );
}

#[test]
fn test_round11_puml_mmd_dot_thrift_fbs_capnp_smithy_editorconfig_pub() {
    assert_both_paths(b"@startuml\nAlice -> Bob: Hello\n@enduml\n", "puml");
    assert_both_paths(b"flowchart TD\n    A[Start] --> B[Done]\n", "mmd");
    assert_both_paths(b"digraph G {\n  A -> B;\n}\n", "dot");
    assert_both_paths(
        b"namespace rs kindprobe\nstruct Probe {\n  1: required string name\n}\n",
        "thrift",
    );
    assert_both_paths(
        b"namespace KindProbe;\ntable Header {\n  id: uint32;\n}\nroot_type Header;\n",
        "fbs",
    );
    assert_both_paths(
        b"@0xdbb9ad1f14bf0b36;\nstruct Point {\n  x @0 :Float32;\n}\n",
        "capnp",
    );
    assert_both_paths(
        b"$version: \"2.0\"\nnamespace com.example\nservice ProbeService {\n  version: \"2026-01-01\"\n}\n",
        "smithy",
    );
    assert_both_paths(
        b"root = true\n\n[*]\nindent_style = space\nindent_size = 4\nend_of_line = lf\n",
        "editorconfig",
    );
    assert_both_paths(
        b"ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIKindProbeTestKey user@host\n",
        "sshpub",
    );
}

#[test]
fn test_round12_cu_glsl_wgsl_hlsl_rego_jsonnet_bicep_dhall() {
    assert_both_paths(
        b"#include <stdio.h>\n__global__ void add(int *a) {\n    int i = threadIdx.x + blockIdx.x;\n    a[i] += 1;\n}\n",
        "cu",
    );
    assert_both_paths(
        b"#version 330 core\nlayout(location = 0) in vec3 aPos;\nvoid main() {\n    gl_Position = vec4(aPos, 1.0);\n}\n",
        "glsl",
    );
    assert_both_paths(
        b"@vertex\nfn vs_main(@location(0) pos: vec3<f32>) -> @builtin(position) vec4<f32> {\n    return vec4<f32>(pos, 1.0);\n}\n",
        "wgsl",
    );
    assert_both_paths(
        b"cbuffer Constants : register(b0) { float4x4 mvp; };\nfloat4 main(float3 pos : POSITION) : SV_Position {\n    return float4(pos, 1.0);\n}\n",
        "hlsl",
    );
    assert_both_paths(
        b"package authz\ndefault allow := false\nallow if {\n    input.user == \"admin\"\n}\n",
        "rego",
    );
    assert_both_paths(
        b"local name = 'kindprobe';\n{\n  app: name,\n  upper: std.asciiUpper(name),\n}\n",
        "jsonnet",
    );
    assert_both_paths(
        b"param location string = 'eastus'\nresource stg 'Microsoft.Storage/storageAccounts@2023-01-01' = {\n  name: 'kpstore'\n  location: location\n}\n",
        "bicep",
    );
    assert_both_paths(
        b"let Config : Type = { port : Natural }\nin let cfg : Config = { port = 8080 }\nin cfg\n",
        "dhall",
    );
}

#[test]
fn test_round13_typ_mdx_mojo_bzl_just_lean_kdl_ron() {
    assert_both_paths(
        b"#set page(paper: \"a4\")\n#let title = \"KindProbe\"\n\n= Introduction\nHello Typst.\n",
        "typ",
    );
    assert_both_paths(
        b"import Chart from './Chart'\n\n# Quarterly Report\n\n<Chart data={[1, 2, 3]} />\n",
        "mdx",
    );
    assert_both_paths(
        b"fn main():\n    var x: Int = 42\n    print(x)\n",
        "mojo",
    );
    assert_both_paths(
        b"load(\"@rules_rust//rust:defs.bzl\", \"rust_binary\")\n\nrust_binary(\n    name = \"kindprobe\",\n    srcs = [\"src/main.rs\"],\n)\n",
        "bzl",
    );
    assert_both_paths(
        b"set shell := [\"bash\", \"-cu\"]\n\ntest:\n    cargo test --release\n",
        "just",
    );
    assert_both_paths(
        b"theorem add_zero (n : Nat) : n + 0 = n := by\n  rfl\n",
        "lean",
    );
    assert_both_paths(
        b"package \"kindprobe\" {\n    version \"0.2.0\"\n    edition \"2024\"\n}\n",
        "kdl",
    );
    assert_both_paths(
        b"(\n    window_size: (1280, 720),\n    title: \"KindProbe\",\n    vsync: true,\n)\n",
        "ron",
    );
}

#[test]
fn test_round14_msix_json5_gherkin_cypher_sparql_wkt_d_caddy() {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut zw = ZipWriter::new(&mut buf);
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        zw.start_file("AppxManifest.xml", opts).expect("start");
        zw.write_all(b"<Package/>").expect("write");
        zw.finish().expect("finish");
    }
    assert_both_paths(&buf.into_inner(), "msix");

    assert_both_paths(
        b"{\n  // JSON5 config\n  name: 'kindprobe',\n  enabled: true,\n}\n",
        "json5",
    );
    assert_both_paths(
        b"Feature: Format detection\n  Scenario: Probe PNG\n    Given a PNG header\n    When I probe it\n    Then the format is png\n",
        "gherkin",
    );
    assert_both_paths(
        b"MATCH (u:User)-[:OWNS]->(f:File)\nWHERE u.active = true\nRETURN f.name\n",
        "cypher",
    );
    assert_both_paths(
        b"PREFIX foaf: <http://xmlns.com/foaf/0.1/>\nSELECT ?name WHERE {\n  ?person foaf:name ?name .\n}\n",
        "sparql",
    );
    assert_both_paths(
        b"POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))\n",
        "wkt",
    );
    assert_both_paths(
        b"import std.stdio;\nvoid main() {\n    writeln(\"Hello D\");\n}\n",
        "d",
    );
    assert_both_paths(
        b"example.com {\n    encode gzip\n    reverse_proxy localhost:8080\n}\n",
        "caddy",
    );
}

#[test]
fn test_round15_wsdl_pom_mjml_xaml_nuspec_haproxy_service_desktop_applescript() {
    assert_both_paths(
        b"<?xml version=\"1.0\"?><definitions xmlns=\"http://schemas.xmlsoap.org/wsdl/\" xmlns:wsdl=\"http://schemas.xmlsoap.org/wsdl/\"></definitions>",
        "wsdl",
    );
    assert_both_paths(
        b"<project><modelVersion>4.0.0</modelVersion><groupId>com.example</groupId><artifactId>kindprobe</artifactId></project>",
        "pom",
    );
    assert_both_paths(
        b"<mjml><mj-body><mj-section><mj-column><mj-text>Hi</mj-text></mj-column></mj-section></mj-body></mjml>",
        "mjml",
    );
    assert_both_paths(
        b"<Window xmlns=\"http://schemas.microsoft.com/winfx/2006/xaml/presentation\" Title=\"KindProbe\"/>",
        "xaml",
    );
    assert_both_paths(
        b"<?xml version=\"1.0\"?><package><metadata><id>KindProbe</id><version>0.2.0</version></metadata></package>",
        "nuspec",
    );
    assert_both_paths(
        b"global\n    daemon\ndefaults\n    mode http\nfrontend http_in\n    bind *:80\n",
        "haproxy",
    );
    assert_both_paths(
        b"[Unit]\nDescription=KindProbe Daemon\n\n[Service]\nExecStart=/usr/local/bin/kindprobe\n\n[Install]\nWantedBy=multi-user.target\n",
        "service",
    );
    assert_both_paths(
        b"[Desktop Entry]\nType=Application\nName=KindProbe\nExec=kindprobe\n",
        "desktop",
    );
    assert_both_paths(
        b"tell application \"Finder\"\n    activate\nend tell\n",
        "applescript",
    );
}

#[test]
fn test_round16_pug_hbs_jinja_ejs_jenkinsfile_earthfile_meson_procfile() {
    assert_both_paths(
        b"doctype html\nhtml(lang=\"en\")\n  body\n    h1 KindProbe\n",
        "pug",
    );
    assert_both_paths(
        b"<div class=\"list\">{{#each items}}<span>{{this}}</span>{{/each}}</div>\n",
        "hbs",
    );
    assert_both_paths(
        b"{% extends \"base.html\" %}\n{% block content %}<h1>{{ title }}</h1>{% endblock %}\n",
        "jinja",
    );
    assert_both_paths(
        b"<ul><% users.forEach(u => { %><li><%= u.name %></li><% }) %></ul>\n",
        "ejs",
    );
    assert_both_paths(
        b"pipeline {\n    agent any\n    stages {\n        stage('Test') {\n            steps { sh 'cargo test' }\n        }\n    }\n}\n",
        "jenkinsfile",
    );
    assert_both_paths(
        b"VERSION 0.8\nFROM rust:1.85\nbuild:\n    COPY . .\n    RUN cargo build --release\n    SAVE ARTIFACT target/release/kindprobe\n",
        "earthfile",
    );
    assert_both_paths(
        b"project('kindprobe', 'c')\ndep = dependency('zlib')\nexecutable('kp', 'main.c', dependencies: dep)\n",
        "meson",
    );
    assert_both_paths(
        b"web: gunicorn app:app\nworker: celery -A app worker\n",
        "procfile",
    );
}

#[test]
fn test_round17_liquid_less_htaccess_vagrantfile_ssh_config_hosts_crontab() {
    assert_both_paths(
        b"{% if user %}\n<p>Hello {{ user.name }}</p>\n{% endif %}\n",
        "liquid",
    );
    assert_both_paths(
        b"@primary: #428bca;\n.btn {\n  color: @primary;\n  padding: 8px;\n}\n",
        "less",
    );
    assert_both_paths(
        b"RewriteEngine On\nRewriteRule ^old$ /new [R=301,L]\n",
        "htaccess",
    );
    assert_both_paths(
        b"<VirtualHost *:80>\n  ServerName example.com\n</VirtualHost>\n",
        "htaccess",
    );
    assert_both_paths(
        b"Vagrant.configure(\"2\") do |config|\n  config.vm.box = \"ubuntu/jammy64\"\nend\n",
        "vagrantfile",
    );
    assert_both_paths(
        b"Host prod\n    HostName 10.0.0.5\n    User deploy\n    IdentityFile ~/.ssh/id_ed25519\n",
        "ssh_config",
    );
    assert_both_paths(
        b"127.0.0.1 localhost\n::1 localhost\n",
        "hosts",
    );
    assert_both_paths(
        b"0 2 * * * /usr/local/bin/backup.sh\n",
        "crontab",
    );
}

#[test]
fn test_round18_mathml_gleam_rkt_ninja_nomad_pkl_elm_odin_vala_f90_cobol_ada_pas_scm_gitmodules_robots() {
    assert_both_paths(
        b"<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><msup><mi>x</mi><mn>2</mn></msup></math>",
        "mathml",
    );
    assert_both_paths(
        b"import gleam/io\npub fn main() {\n  io.println(\"hello\")\n}\n",
        "gleam",
    );
    assert_both_paths(
        b"#lang racket\n(define (square x) (* x x))\n",
        "rkt",
    );
    assert_both_paths(
        b"rule cc\n  command = gcc -c $in -o $out\nbuild foo.o: cc foo.c\n",
        "ninja",
    );
    assert_both_paths(
        b"job \"api\" {\n  group \"web\" {\n    task \"server\" {}\n  }\n}\n",
        "nomad",
    );
    assert_both_paths(
        b"amends \"package://pkg.pkl-lang.org/pkl-k8s/k8s@1.0.0#/Deployment.pkl\"\nname = \"kindprobe\"\n",
        "pkl",
    );
    assert_both_paths(
        b"module Main exposing (main)\nimport Html exposing (text)\nmain = text \"Hello\"\n",
        "elm",
    );
    assert_both_paths(
        b"package main\nimport \"core:fmt\"\nmain :: proc() {\n  fmt.println(\"hi\")\n}\n",
        "odin",
    );
    assert_both_paths(
        b"using GLib;\npublic static int main(string[] args) {\n  stdout.printf(\"Hello\\n\");\n  return 0;\n}\n",
        "vala",
    );
    assert_both_paths(
        b"program hello\n  implicit none\n  print *, 'Hello'\nend program hello\n",
        "f90",
    );
    assert_both_paths(
        b"       IDENTIFICATION DIVISION.\n       PROGRAM-ID. HELLO.\n       PROCEDURE DIVISION.\n           STOP RUN.\n",
        "cobol",
    );
    assert_both_paths(
        b"with Ada.Text_IO;\nprocedure Hello is\nbegin\n  Ada.Text_IO.Put_Line(\"Hi\");\nend Hello;\n",
        "ada",
    );
    assert_both_paths(
        b"program Hello;\nbegin\n  writeln('Hello');\nend.\n",
        "pas",
    );
    assert_both_paths(
        b"(define (make-adder x)\n  (lambda (y) (+ x y)))\n",
        "scm",
    );
    assert_both_paths(
        b"[submodule \"vendor/foo\"]\n\tpath = vendor/foo\n\turl = https://github.com/example/foo.git\n",
        "gitmodules",
    );
    assert_both_paths(
        b"User-agent: *\nDisallow: /admin/\n",
        "robots",
    );
}

#[test]
fn test_round19_sitemap_resx_wadl_purs_gitattributes_codeowners_npmrc_yarnlock_gemfile_podfile_brewfile_hx_prql_jq_udev_nft_iptables_zone() {
    assert_both_paths(
        b"<?xml version=\"1.0\"?><urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\"><url><loc>https://example.com/</loc></url></urlset>",
        "sitemap",
    );
    assert_both_paths(
        b"<?xml version=\"1.0\"?><root><resheader name=\"reader\"><value>System.Resources.ResXResourceReader</value></resheader></root>",
        "resx",
    );
    assert_both_paths(
        b"<?xml version=\"1.0\"?><application xmlns=\"http://wadl.dev.java.net/2009/02\"></application>",
        "wadl",
    );
    assert_both_paths(
        b"module Main where\n\nimport Prelude\nimport Effect.Console (log)\n",
        "purs",
    );
    assert_both_paths(
        b"* text=auto\n*.png binary\n*.rs text eol=lf\n",
        "gitattributes",
    );
    assert_both_paths(
        b"* @org/core\n/src/ @org/rust-team\n",
        "codeowners",
    );
    assert_both_paths(
        b"registry=https://registry.npmjs.org/\nsave-exact=true\n",
        "npmrc",
    );
    assert_both_paths(
        b"# yarn lockfile v1\n\nfoo@^1.0.0:\n  version \"1.0.0\"\n  resolved \"https://registry.yarnpkg.com/foo/-/foo-1.0.0.tgz\"\n",
        "yarnlock",
    );
    assert_both_paths(
        b"source 'https://rubygems.org'\ngem 'rails', '~> 7.1'\n",
        "gemfile",
    );
    assert_both_paths(
        b"platform :ios, '16.0'\ntarget 'MyApp' do\n  pod 'Alamofire'\nend\n",
        "podfile",
    );
    assert_both_paths(
        b"tap \"homebrew/cask\"\nbrew \"ripgrep\"\ncask \"ghostty\"\n",
        "brewfile",
    );
    assert_both_paths(
        b"class Main {\n  static function main() {\n    trace(\"Hello Haxe\");\n  }\n}\n",
        "hx",
    );
    assert_both_paths(
        b"from invoices\nfilter total > 100\nderive { tax = total * 0.1 }\nselect { id, tax }\n",
        "prql",
    );
    assert_both_paths(
        b"def active_names: .[] | select(.active) | .name;\n",
        "jq",
    );
    assert_both_paths(
        b"ACTION==\"add\", SUBSYSTEM==\"usb\", ATTR{idVendor}==\"1234\", MODE=\"0666\"\n",
        "udev",
    );
    assert_both_paths(
        b"table inet filter {\n  chain input {\n    type filter hook input priority 0; accept;\n  }\n}\n",
        "nft",
    );
    assert_both_paths(
        b"*filter\n:INPUT ACCEPT [0:0]\n-A INPUT -p tcp --dport 22 -j ACCEPT\nCOMMIT\n",
        "iptables",
    );
    assert_both_paths(
        b"$TTL 86400\n@ IN SOA ns1.example.com. admin.example.com. ( 1 3600 900 604800 86400 )\n@ IN NS ns1.example.com.\n",
        "zone",
    );
}

#[test]
fn test_round20_xbel_docbook_ivy_nu_sv_org_mediawiki_fennel_hy_lfe_raku_cr_fish_tmux_vimrc_gitconfig_sysctl_resolv() {
    assert_both_paths(
        b"<?xml version=\"1.0\"?><xbel version=\"1.0\"><bookmark href=\"https://example.com\"><title>Ex</title></bookmark></xbel>",
        "xbel",
    );
    assert_both_paths(
        b"<?xml version=\"1.0\"?><book xmlns=\"http://docbook.org/ns/docbook\"><title>Guide</title></book>",
        "docbook",
    );
    assert_both_paths(
        b"<?xml version=\"1.0\"?><ivy-module version=\"2.0\"><info organisation=\"com.example\" module=\"kp\"/></ivy-module>",
        "ivy",
    );
    assert_both_paths(
        b"def active-procs [limit] {\n  ps | where mem > $limit\n}\n",
        "nu",
    );
    assert_both_paths(
        b"module counter(input logic clk, output logic [7:0] q);\n  always_ff @(posedge clk) q <= q + 1;\nendmodule\n",
        "sv",
    );
    assert_both_paths(
        b"#+TITLE: KindProbe Notes\n#+AUTHOR: Dev\n* Overview\n",
        "org",
    );
    assert_both_paths(
        b"== History ==\n[[KindProbe]] is a '''fast''' file detector.\n",
        "mediawiki",
    );
    assert_both_paths(
        b"(local greet (fn [name] (print (.. \"Hi \" name))))\n",
        "fennel",
    );
    assert_both_paths(
        b"(import os)\n(defn main [] (print \"Hello Hy\"))\n",
        "hy",
    );
    assert_both_paths(
        b"(defmodule hello (export (world 0)))\n(defun world () 'ok)\n",
        "lfe",
    );
    assert_both_paths(
        b"use v6;\nmy Int $x = 42;\nsay $x;\n",
        "raku",
    );
    assert_both_paths(
        b"class User\n  property name : String\n  def initialize(@name)\n  end\nend\n",
        "cr",
    );
    assert_both_paths(
        b"function greet\n    set -l name World\n    echo \"Hello $name\"\nend\n",
        "fish",
    );
    assert_both_paths(
        b"set -g mouse on\nbind r source-file ~/.tmux.conf\n",
        "tmux",
    );
    assert_both_paths(
        b"set number\nnnoremap <leader>w :w<CR>\n",
        "vimrc",
    );
    assert_both_paths(
        b"[user]\n\tname = Jane Doe\n\temail = jane@example.com\n",
        "gitconfig",
    );
    assert_both_paths(
        b"net.ipv4.ip_forward = 1\nvm.swappiness = 10\n",
        "sysctl",
    );
    assert_both_paths(
        b"search example.com\nnameserver 1.1.1.1\n",
        "resolv",
    );
}

#[test]
fn test_round21_sarif_cyclonedx_spdx_sourcemap_webmanifest_junit_cobertura_checkstyle_vy_move_cairo_circom_flux_kql_fstab_textile_coffee() {
    assert_both_paths(
        br#"{"version":"2.1.0","runs":[{"tool":{"driver":{"name":"clippy"}},"results":[]}]}"#,
        "sarif",
    );
    assert_both_paths(
        br#"{"bomFormat":"CycloneDX","specVersion":"1.5","version":1,"components":[]}"#,
        "cyclonedx",
    );
    assert_both_paths(
        br#"{"spdxVersion":"SPDX-2.3","SPDXID":"SPDXRef-DOCUMENT","packages":[]}"#,
        "spdx",
    );
    assert_both_paths(
        br#"{"version":3,"sources":["app.ts"],"mappings":"AAAA"}"#,
        "sourcemap",
    );
    assert_both_paths(
        br#"{"short_name":"KindProbe","start_url":"/","icons":[{"src":"/icon.png"}]}"#,
        "webmanifest",
    );
    assert_both_paths(
        b"<?xml version=\"1.0\"?><testsuite name=\"unit\" tests=\"1\"><testcase name=\"ok\"/></testsuite>",
        "junit",
    );
    assert_both_paths(
        b"<?xml version=\"1.0\"?><coverage line-rate=\"0.95\"><packages><package name=\"kp\"/></packages></coverage>",
        "cobertura",
    );
    assert_both_paths(
        b"<?xml version=\"1.0\"?><checkstyle version=\"10.0\"><file name=\"A.java\"><error line=\"1\"/></file></checkstyle>",
        "checkstyle",
    );
    assert_both_paths(
        b"@external\ndef get_owner() -> address:\n    return self.owner\nowner: public(address)\n",
        "vy",
    );
    assert_both_paths(
        b"module 0x1::coin {\n    use std::signer;\n    struct Coin has key { val: u64 }\n    public fun mint(acc: &signer) {}\n}\n",
        "move",
    );
    assert_both_paths(
        b"#[starknet::contract]\nmod Counter {\n    fn get() -> u32 { 0 }\n}\n",
        "cairo",
    );
    assert_both_paths(
        b"pragma circom 2.1.0;\ntemplate Multiplier() {\n    signal input a;\n    signal output c;\n}\n",
        "circom",
    );
    assert_both_paths(
        b"from(bucket: \"metrics\")\n  |> range(start: -1h)\n  |> filter(fn: (r) => r._measurement == \"cpu\")\n",
        "flux",
    );
    assert_both_paths(
        b"Logs\n| where Level == \"Error\"\n| summarize Count = count() by Service\n| project Service, Count\n",
        "kql",
    );
    assert_both_paths(
        b"UUID=1234-5678 / ext4 defaults 0 1\n",
        "fstab",
    );
    assert_both_paths(
        b"h1. Title\n\nh2. Section\n\n* Item one\n",
        "textile",
    );
    assert_both_paths(
        b"square = (x) -> x * x\nconsole.log square 5\n",
        "coffee",
    );
}

#[test]
fn test_round22_jsonschema_jsonld_vega_postman_asyncapi_glade_qtui_xliff_tmx_musicxml_expect_automake_autoconf_supervisor_logrotate_alloy_ftl() {
    assert_both_paths(
        br#"{"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"id":{"type":"string"}}}"#,
        "jsonschema",
    );
    assert_both_paths(
        br#"{"@context":"https://schema.org","@type":"Person","name":"Jane Doe"}"#,
        "jsonld",
    );
    assert_both_paths(
        br#"{"$schema":"https://vega.github.io/schema/vega-lite/v5.json","mark":"bar","data":{"values":[]}}"#,
        "vega",
    );
    assert_both_paths(
        br#"{"info":{"_postman_id":"1234","name":"API"},"item":[]}"#,
        "postman",
    );
    assert_both_paths(
        br#"{"asyncapi":"3.0.0","info":{"title":"Events","version":"1.0"},"channels":{}}"#,
        "asyncapi",
    );
    assert_both_paths(
        b"<?xml version=\"1.0\"?><interface><object class=\"GtkWindow\" id=\"win\"/></interface>",
        "glade",
    );
    assert_both_paths(
        b"<?xml version=\"1.0\"?><ui version=\"4.0\"><widget class=\"QMainWindow\" name=\"MainWindow\"/></ui>",
        "qt_ui",
    );
    assert_both_paths(
        b"<?xml version=\"1.0\"?><xliff version=\"1.2\"><file source-language=\"en\" target-language=\"zh\"><body><trans-unit id=\"1\"/></body></file></xliff>",
        "xliff",
    );
    assert_both_paths(
        b"<?xml version=\"1.0\"?><tmx version=\"1.4\"><body><tu><tuv xml:lang=\"en\"><seg>Hi</seg></tuv></tu></body></tmx>",
        "tmx",
    );
    assert_both_paths(
        b"<?xml version=\"1.0\"?><score-partwise version=\"4.0\"><part id=\"P1\"/></score-partwise>",
        "musicxml",
    );
    assert_both_paths(
        b"spawn ssh user@host\nexpect \"password:\"\nsend \"secret\\r\"\n",
        "expect",
    );
    assert_both_paths(
        b"SUBDIRS = src tests\nbin_PROGRAMS = kindprobe\n",
        "automake",
    );
    assert_both_paths(
        b"AC_INIT([kindprobe], [0.2.0])\nAC_PROG_CC\nAC_OUTPUT\n",
        "autoconf",
    );
    assert_both_paths(
        b"[program:kindprobe]\ncommand=/usr/local/bin/kindprobe\nautostart=true\n",
        "supervisor",
    );
    assert_both_paths(
        b"/var/log/kindprobe.log {\n    daily\n    rotate 7\n    compress\n}\n",
        "logrotate",
    );
    assert_both_paths(
        b"prometheus.scrape \"default\" {\n  targets = []\n  forward_to = [prometheus.remote_write.default.receiver]\n}\n",
        "alloy",
    );
    assert_both_paths(
        b"welcome = Hello, {$user}!\n",
        "ftl",
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
