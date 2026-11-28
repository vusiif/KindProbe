use super::make_source_core;
use crate::model::{DetectionCore, Verification};

pub(super) fn probe_specialized_and_styles(window: &str, w_bytes: &[u8]) -> Option<DetectionCore> {
    // GNU Autoconf (configure.ac)
    if window.contains("AC_INIT(") && (window.contains("AC_OUTPUT") || window.contains("AC_PROG_"))
    {
        return Some(make_source_core(
            "autoconf",
            &["ac"],
            "sig.text.autoconf",
            Verification::Structure,
        ));
    }

    // Supervisor config (supervisord.conf)
    if window.contains("[program:") && window.contains("command=") {
        return Some(make_source_core(
            "supervisor",
            &["conf"],
            "sig.text.supervisor",
            Verification::Structure,
        ));
    }

    // logrotate config
    if (window.contains("daily") || window.contains("weekly") || window.contains("monthly"))
        && window.contains("rotate ")
        && window.contains("compress")
        && w_bytes.contains(&b'{')
    {
        return Some(make_source_core(
            "logrotate",
            &["conf"],
            "sig.text.logrotate",
            Verification::Structure,
        ));
    }

    // Grafana Alloy (.alloy)
    if window.contains("prometheus.scrape \"") && window.contains("forward_to =") {
        return Some(make_source_core(
            "alloy",
            &["alloy"],
            "sig.text.alloy",
            Verification::Structure,
        ));
    }

    // Mozilla Project Fluent (.ftl)
    if window.contains(" = ")
        && window.contains("{$")
        && window.contains('}')
        && !w_bytes.contains(&b';')
    {
        return Some(make_source_core(
            "ftl",
            &["ftl"],
            "sig.text.fluent",
            Verification::Structure,
        ));
    }

    // Flatpak Reference (.flatpakref)
    if window.contains("[Flatpak Ref]") && window.contains("Url=") {
        return Some(make_source_core(
            "flatpakref",
            &["flatpakref"],
            "sig.text.flatpakref",
            Verification::Structure,
        ));
    }

    // GNU gettext PO/POT (.po / .pot)
    if window.contains("msgid \"") && window.contains("msgstr \"") {
        return Some(make_source_core(
            "po",
            &["po", "pot"],
            "sig.text.gettext_po",
            Verification::Structure,
        ));
    }

    // SPICE circuit netlist (.cir / .sp)
    if (window.contains(".subckt ") && window.contains(".ends"))
        || (window.contains(".tran ") && window.contains(".end"))
    {
        return Some(make_source_core(
            "spice",
            &["cir", "sp", "ckt"],
            "sig.text.spice",
            Verification::Structure,
        ));
    }

    // CNC / 3D Printer G-code (.gcode / .nc)
    if (window.contains("G0 ") || window.contains("G1 ") || window.contains("G28"))
        && (window.contains("M104") || window.contains("M109") || window.contains("G90"))
    {
        return Some(make_source_core(
            "gcode",
            &["gcode", "nc"],
            "sig.text.gcode",
            Verification::Structure,
        ));
    }

    // Buck2 build file (BUCK)
    if (window.contains("rust_library(")
        || window.contains("cxx_binary(")
        || window.contains("python_binary("))
        && window.contains("deps = [")
        && !window.contains("load(")
    {
        return Some(make_source_core(
            "buck",
            &["buck"],
            "sig.text.buck",
            Verification::Structure,
        ));
    }

    // Modelica (.mo)
    if window.contains("model ")
        && window.contains("equation")
        && window.contains("end ")
        && !w_bytes.contains(&b'{')
    {
        return Some(make_source_core(
            "modelica",
            &["mo"],
            "sig.text.modelica",
            Verification::Structure,
        ));
    }

    // sudoers rules
    if window.contains("ALL=(ALL") && (window.contains("NOPASSWD:") || window.contains(" ALL")) {
        return Some(make_source_core(
            "sudoers",
            &["sudoers"],
            "sig.text.sudoers",
            Verification::Structure,
        ));
    }

    // OpenSCAD (.scad)
    if window.contains("module ")
        && (window.contains("cube(")
            || window.contains("cylinder(")
            || window.contains("difference(")
            || window.contains("union("))
        && w_bytes.contains(&b'{')
        && w_bytes.contains(&b';')
    {
        return Some(make_source_core(
            "scad",
            &["scad"],
            "sig.text.openscad",
            Verification::Structure,
        ));
    }

    // GNU Linker Script (.ld / .lds)
    if window.contains("SECTIONS")
        && (window.contains("ENTRY(") || window.contains(".text :") || window.contains("KEEP("))
        && w_bytes.contains(&b'{')
    {
        return Some(make_source_core(
            "lds",
            &["ld", "lds"],
            "sig.text.linker_script",
            Verification::Structure,
        ));
    }

    // Linux Device Tree Source (.dts / .dtsi)
    if window.contains("/dts-v1/;")
        || (window.contains("compatible = \"")
            && window.contains("reg = <")
            && window.contains("};"))
    {
        return Some(make_source_core(
            "dts_src",
            &["dts", "dtsi"],
            "sig.text.devicetree",
            Verification::Structure,
        ));
    }

    // Puppet manifest (.pp)
    if (window.contains("package {") || window.contains("service {") || window.contains("file {"))
        && window.contains("ensure =>")
    {
        return Some(make_source_core(
            "puppet",
            &["pp"],
            "sig.text.puppet",
            Verification::Structure,
        ));
    }

    // MATLAB / Octave (.m)
    if window.contains("function [") && window.contains("] = ") && window.contains("end") {
        return Some(make_source_core(
            "matlab",
            &["m"],
            "sig.text.matlab",
            Verification::Structure,
        ));
    }

    // Slint UI (.slint)
    if window.contains("export component ")
        && (window.contains("inherits Window")
            || window.contains("Text {")
            || window.contains("in-out property"))
    {
        return Some(make_source_core(
            "slint",
            &["slint"],
            "sig.text.slint",
            Verification::Structure,
        ));
    }

    // Qt QML (.qml)
    if window.contains("import QtQuick")
        && (window.contains("Rectangle {")
            || window.contains("Window {")
            || window.contains("Item {")
            || window.contains("ApplicationWindow {"))
    {
        return Some(make_source_core(
            "qml",
            &["qml"],
            "sig.text.qml",
            Verification::Structure,
        ));
    }

    // CUE configuration (.cue)
    if window.contains("package ")
        && window.contains('#')
        && window.contains(": {")
        && (window.contains("string") || window.contains("int") || window.contains("bool"))
    {
        return Some(make_source_core(
            "cuelang",
            &["cue"],
            "sig.text.cue",
            Verification::Structure,
        ));
    }

    // Smalltalk (.st)
    if window.contains("Object subclass:") && window.contains("instanceVariableNames:") {
        return Some(make_source_core(
            "smalltalk",
            &["st"],
            "sig.text.smalltalk",
            Verification::Structure,
        ));
    }

    // Prolog (.pro / .P)
    if window.contains(":-")
        && window.contains(").")
        && window.contains("?-")
        && !window.contains("use strict")
    {
        return Some(make_source_core(
            "prolog",
            &["pro", "P"],
            "sig.text.prolog",
            Verification::Structure,
        ));
    }

    // WebIDL / OMG IDL (.idl / .webidl)
    if (window.contains("interface ")
        && (window.contains("readonly attribute")
            || window.contains("DOMString")
            || window.contains("Promise<")))
        || (window.contains("module ") && window.contains("interface ") && window.contains("};"))
    {
        return Some(make_source_core(
            "idl",
            &["idl", "webidl"],
            "sig.text.idl",
            Verification::Structure,
        ));
    }

    // SAS script (.sas)
    if window.contains("data ") && window.contains("set ") && window.contains("run;") {
        return Some(make_source_core(
            "sas_script",
            &["sas"],
            "sig.text.sas",
            Verification::Structure,
        ));
    }

    // Aleo Leo (.leo)
    if window.contains("program ") && window.contains(".aleo") && window.contains("transition ") {
        return Some(make_source_core(
            "leo",
            &["leo"],
            "sig.text.leo",
            Verification::Structure,
        ));
    }

    // Valve VDF / Steam ACF (.vdf / .acf)
    if (window.contains("\"AppState\"") || window.contains("\"InstallConfigStore\""))
        && (window.contains("\"appid\"") || window.contains("\"Software\""))
        && w_bytes.contains(&b'{')
    {
        return Some(make_source_core(
            "vdf",
            &["vdf", "acf"],
            "sig.text.vdf",
            Verification::Structure,
        ));
    }

    // Wolfram Mathematica Notebook (.nb)
    if window.contains("Notebook[{") && window.contains("Cell[") {
        return Some(make_source_core(
            "nb",
            &["nb"],
            "sig.text.mathematica_nb",
            Verification::Structure,
        ));
    }

    // Eiffel (.e)
    if window.contains("class ")
        && window.contains("feature")
        && window.contains("end")
        && !w_bytes.contains(&b'{')
    {
        return Some(make_source_core(
            "eiffel",
            &["e"],
            "sig.text.eiffel",
            Verification::Structure,
        ));
    }

    // .netrc
    if window.contains("machine ") && window.contains("login ") && window.contains("password ") {
        return Some(make_source_core(
            "netrc",
            &["netrc"],
            "sig.text.netrc",
            Verification::Structure,
        ));
    }

    // MySQL my.cnf
    if window.contains("[mysqld]")
        && (window.contains("datadir") || window.contains("port") || window.contains("socket"))
    {
        return Some(make_source_core(
            "my_cnf",
            &["cnf"],
            "sig.text.my_cnf",
            Verification::Structure,
        ));
    }

    // php.ini
    if window.contains("[PHP]")
        && (window.contains("engine")
            || window.contains("memory_limit")
            || window.contains("display_errors"))
    {
        return Some(make_source_core(
            "php_ini",
            &["ini"],
            "sig.text.php_ini",
            Verification::Structure,
        ));
    }

    // Emacs Lisp (.el)
    if window.contains("(defun ")
        && (window.contains("(interactive")
            || window.contains("(require '")
            || window.contains("(setq "))
    {
        return Some(make_source_core(
            "elisp",
            &["el", "elc"],
            "sig.text.elisp",
            Verification::Structure,
        ));
    }

    // Common Lisp (.lisp / .lsp / .cl)
    if window.contains("(defun ")
        && (window.contains("(defparameter ")
            || window.contains("(defvar ")
            || window.contains("(format t "))
    {
        return Some(make_source_core(
            "lisp",
            &["lisp", "lsp", "cl"],
            "sig.text.lisp",
            Verification::Structure,
        ));
    }

    // ReScript (.res / .resi)
    if window.contains("let ")
        && (window.contains("Js.log(")
            || window.contains("Console.log(")
            || window.contains("@react.component"))
        && !w_bytes.contains(&b';')
    {
        return Some(make_source_core(
            "rescript",
            &["res", "resi"],
            "sig.text.rescript",
            Verification::Structure,
        ));
    }

    // Salesforce Apex (.cls / .trigger)
    if window.contains("public with sharing class ")
        || (window.contains("trigger ")
            && window.contains(" on ")
            && window.contains("before insert"))
    {
        return Some(make_source_core(
            "apex",
            &["cls", "trigger"],
            "sig.text.apex",
            Verification::Structure,
        ));
    }

    // SAP ABAP (.abap)
    if window.contains("REPORT ")
        && (window.contains("DATA:") || window.contains("WRITE:") || window.contains("SELECT "))
    {
        return Some(make_source_core(
            "abap",
            &["abap"],
            "sig.text.abap",
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
                    && k.bytes()
                        .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
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
        && (window.contains("@mixin ")
            || window.contains("@include ")
            || (w_bytes.contains(&b'$') && window.contains(": ")))
        && (window.contains("color:")
            || window.contains("margin:")
            || window.contains("padding:")
            || window.contains("display:"))
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
            t.starts_with('@')
                && t.contains(':')
                && t.ends_with(';')
                && !t.starts_with("@media")
                && !t.starts_with("@import")
        })
        && (window.contains("color:")
            || window.contains("margin:")
            || window.contains("padding:")
            || window.contains("display:"))
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
