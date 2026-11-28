use super::make_source_core;
use crate::model::{DetectionCore, Verification};

pub(super) fn probe_devops_and_idl(window: &str, w_bytes: &[u8]) -> Option<DetectionCore> {
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
    if window
        .lines()
        .any(|l| l.trim_start().starts_with("module "))
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
    if window
        .lines()
        .any(|l| l.trim_start().starts_with("module "))
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
    if window.contains("@Grab(")
        || (window.contains("def ")
            && window.contains("println ")
            && !window.contains("def main(")
            && !window.contains(':'))
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
        && (window.contains("print ")
            || window.contains("printf ")
            || window.contains("$1")
            || window.contains("$0"))
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
        && (window.contains("-->")
            || window.contains("->")
            || window.contains("||--")
            || window.contains("participant "))
    {
        return Some(make_source_core(
            "mmd",
            &["mmd", "mermaid"],
            "sig.text.mermaid",
            Verification::Structure,
        ));
    }

    // Graphviz DOT (.dot / .gv)
    if (window.contains("digraph ")
        || window.contains("strict digraph ")
        || window.contains("strict graph "))
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
    if window.contains("$version:")
        && window.contains("namespace ")
        && (window.contains("service ") || window.contains("structure "))
    {
        return Some(make_source_core(
            "smithy",
            &["smithy"],
            "sig.text.smithy",
            Verification::Structure,
        ));
    }

    // EditorConfig (.editorconfig)
    if (window.contains("root = true") || window.contains("[*]"))
        && (window.contains("indent_style")
            || window.contains("indent_size")
            || window.contains("end_of_line"))
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
        && (window.contains("'Microsoft.")
            || window.contains("@description(")
            || window.contains("= {"))
    {
        return Some(make_source_core(
            "bicep",
            &["bicep"],
            "sig.text.bicep",
            Verification::Structure,
        ));
    }

    // DirectX HLSL (.hlsl / .fx)
    if (window.contains("cbuffer ")
        || window.contains("StructuredBuffer<")
        || window.contains("Texture2D"))
        && (window.contains("SV_Position")
            || window.contains("SV_Target")
            || window.contains("float4"))
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
        && window.lines().any(|l| {
            let t = l.trim();
            t.ends_with(" {") && t.contains('"')
        })
        && !window.contains("server {")
        && !window.contains("plugins {")
        && !window.contains("job \"")
        && !window.contains(" = ")
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
    if (ron_trimmed.starts_with('{')
        || ron_trimmed.starts_with("//")
        || ron_trimmed.starts_with("/*"))
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
        || (ron_trimmed.starts_with("SRID=")
            && (ron_trimmed.contains(";POINT") || ron_trimmed.contains(";POLYGON")))
    {
        return Some(make_source_core(
            "wkt",
            &["wkt"],
            "sig.text.wkt",
            Verification::Structure,
        ));
    }
    None
}
