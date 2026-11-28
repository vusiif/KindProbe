use super::make_source_core;
use crate::model::{DetectionCore, Verification};

pub(super) fn probe_infra_and_langs(window: &str, w_bytes: &[u8]) -> Option<DetectionCore> {
    // D programming language (.d)
    if window.contains("import std.")
        && (window.contains("void main(")
            || window.contains("int main(")
            || window.contains("writeln("))
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
        && (window.contains("frontend ")
            || window.contains("backend ")
            || window.contains("listen "))
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
        && (window.contains("[Service]")
            || window.contains("[Timer]")
            || window.contains("[Socket]"))
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
    if window.contains("[Desktop Entry]") && window.contains("Type=") && window.contains("Name=") {
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
    if (window.contains("{% endif %}") || window.contains("{% endfor %}")) && window.contains("{%")
    {
        return Some(make_source_core(
            "liquid",
            &["liquid"],
            "sig.text.liquid",
            Verification::Structure,
        ));
    }
    if (window.contains("{% block ")
        || window.contains("{% extends ")
        || window.contains("{% for "))
        && window.contains("%}")
    {
        return Some(make_source_core(
            "jinja",
            &["j2", "jinja", "jinja2"],
            "sig.text.jinja",
            Verification::Structure,
        ));
    }
    if (window.contains("<%=") || window.contains("<%-") || window.contains("<% "))
        && window.contains("%>")
    {
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
        && (window.contains("SAVE ARTIFACT")
            || window.contains("BUILD +")
            || window.contains("RUN "))
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
        && (window.contains("executable(")
            || window.contains("dependency(")
            || window.contains("library("))
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
        && (window.contains("HostName ")
            || window.contains("IdentityFile ")
            || window.contains("User "))
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
        && (window.contains("import Html")
            || window.contains("import Browser")
            || window.contains("main ="))
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
        && (window.contains("Disallow:")
            || window.contains("Allow:")
            || window.contains("Sitemap:"))
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
    if window
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.starts_with('#') && (t.starts_with("* ") || t.starts_with('/')) && t.contains(" @")
        })
        .count()
        >= 2
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
    if (window.contains("source 'https://rubygems.org'")
        || window.contains("source \"https://rubygems.org\""))
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
        && (window.contains("type filter hook")
            || window.contains("accept")
            || window.contains("drop"))
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

    // MediaWiki (.wiki / .mediawiki)
    if window.contains("== ")
        && window.contains(" ==")
        && window.contains("[[")
        && window.contains("]]")
        && window.contains("'''")
    {
        return Some(make_source_core(
            "mediawiki",
            &["wiki", "mediawiki"],
            "sig.text.mediawiki",
            Verification::Structure,
        ));
    }

    // Fennel (.fnl)
    if window.contains("(fn ")
        && (window.contains("(local ") || window.contains("(let "))
        && w_bytes.contains(&b')')
    {
        return Some(make_source_core(
            "fennel",
            &["fnl"],
            "sig.text.fennel",
            Verification::Structure,
        ));
    }

    // Hy (.hy)
    if window.contains("(import ") && window.contains("(defn ") && w_bytes.contains(&b')') {
        return Some(make_source_core(
            "hy",
            &["hy"],
            "sig.text.hy",
            Verification::Structure,
        ));
    }

    // LFE - Lisp Flavored Erlang (.lfe)
    if window.contains("(defmodule ") && window.contains("(export ") {
        return Some(make_source_core(
            "lfe",
            &["lfe"],
            "sig.text.lfe",
            Verification::Structure,
        ));
    }

    // Raku (.raku / .p6)
    if window.contains("use v6;") || (window.contains("my Int $") && window.contains("say ")) {
        return Some(make_source_core(
            "raku",
            &["raku", "rakumod", "p6"],
            "sig.text.raku",
            Verification::Structure,
        ));
    }

    // Crystal (.cr)
    if (window.contains("class ") || window.contains("struct "))
        && window.contains("property ")
        && window.contains("end")
        && !w_bytes.contains(&b'{')
    {
        return Some(make_source_core(
            "cr",
            &["cr"],
            "sig.text.crystal",
            Verification::Structure,
        ));
    }

    // Fish shell (.fish)
    if window.contains("function ")
        && (window.contains("set -l ") || window.contains("set -gx ") || window.contains("set -U "))
        && window.contains("end")
    {
        return Some(make_source_core(
            "fish",
            &["fish"],
            "sig.text.fish",
            Verification::Structure,
        ));
    }

    // tmux.conf
    if window.contains("set -g ") && (window.contains("bind ") || window.contains("bind-key ")) {
        return Some(make_source_core(
            "tmux",
            &["conf"],
            "sig.text.tmux",
            Verification::Structure,
        ));
    }

    // .vimrc
    if window.contains("set nocompatible")
        || (window.contains("set number")
            && (window.contains("nnoremap ")
                || window.contains("inoremap ")
                || window.contains("autocmd ")))
    {
        return Some(make_source_core(
            "vimrc",
            &["vim", "vimrc"],
            "sig.text.vimrc",
            Verification::Structure,
        ));
    }

    // .gitconfig
    if window.contains("[user]") && window.contains("name =") && window.contains("email =") {
        return Some(make_source_core(
            "gitconfig",
            &["gitconfig"],
            "sig.text.gitconfig",
            Verification::Structure,
        ));
    }

    // sysctl.conf
    if (window.contains("net.ipv4.")
        || window.contains("net.core.")
        || window.contains("vm.swappiness"))
        && window.contains(" = ")
    {
        return Some(make_source_core(
            "sysctl",
            &["conf"],
            "sig.text.sysctl",
            Verification::Structure,
        ));
    }

    // resolv.conf
    if window.contains("nameserver ")
        && (window.contains("search ") || window.contains("domain ") || window.contains("options "))
    {
        return Some(make_source_core(
            "resolv",
            &["conf"],
            "sig.text.resolv",
            Verification::Structure,
        ));
    }

    // Circom ZK circuits (.circom)
    if window.contains("pragma circom ")
        && window.contains("template ")
        && window.contains("signal ")
    {
        return Some(make_source_core(
            "circom",
            &["circom"],
            "sig.text.circom",
            Verification::Structure,
        ));
    }

    // InfluxDB Flux (.flux)
    if window.contains("from(bucket:")
        && window.contains("|> range(")
        && window.contains("|> filter(")
    {
        return Some(make_source_core(
            "flux",
            &["flux"],
            "sig.text.flux",
            Verification::Structure,
        ));
    }

    // Azure Kusto Query Language (.kql / .csl)
    if window.contains("| where ")
        && (window.contains("| summarize ") || window.contains("| project "))
        && !window.contains("def ")
        && !w_bytes.contains(&b'{')
    {
        return Some(make_source_core(
            "kql",
            &["kql", "csl"],
            "sig.text.kql",
            Verification::Structure,
        ));
    }

    // /etc/fstab
    if (window.contains("UUID=") || window.contains("/dev/"))
        && (window.contains("ext4")
            || window.contains("xfs")
            || window.contains("btrfs")
            || window.contains("swap")
            || window.contains("tmpfs"))
        && window.contains("defaults")
    {
        return Some(make_source_core(
            "fstab",
            &["fstab"],
            "sig.text.fstab",
            Verification::Structure,
        ));
    }

    // Textile markup (.textile)
    if window.contains("h1. ")
        && (window.contains("h2. ") || window.contains("\n* ") || window.contains("\n# "))
    {
        return Some(make_source_core(
            "textile",
            &["textile"],
            "sig.text.textile",
            Verification::Structure,
        ));
    }

    // CoffeeScript (.coffee)
    if window.contains("->")
        && window.contains("console.log ")
        && !w_bytes.contains(&b'{')
        && !w_bytes.contains(&b';')
    {
        return Some(make_source_core(
            "coffee",
            &["coffee"],
            "sig.text.coffeescript",
            Verification::Structure,
        ));
    }

    // Expect script (.exp)
    if window.contains("spawn ") && window.contains("expect ") && window.contains("send ") {
        return Some(make_source_core(
            "expect",
            &["exp"],
            "sig.text.expect",
            Verification::Structure,
        ));
    }

    // GNU Automake (Makefile.am)
    if (window.contains("bin_PROGRAMS")
        || window.contains("lib_LTLIBRARIES")
        || window.contains("SUBDIRS ="))
        && window.contains('_')
    {
        return Some(make_source_core(
            "automake",
            &["am"],
            "sig.text.automake",
            Verification::Structure,
        ));
    }
    None
}
