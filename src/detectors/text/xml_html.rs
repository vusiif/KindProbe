use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

/// Probes whether the text represents an XML or HTML document, returning zero-allocation DetectionCore.
pub fn probe_xml_or_html_core(text: &str) -> Option<DetectionCore> {
    let trimmed = text.trim_start();
    if !trimmed.starts_with('<') {
        return None;
    }

    // Zero-heap-allocation lowercased ASCII head window (up to 256 bytes)
    let limit = trimmed.len().min(256);
    let mut buf = [0u8; 256];
    let src = &trimmed.as_bytes()[..limit];
    for (i, &b) in src.iter().enumerate() {
        buf[i] = b.to_ascii_lowercase();
    }
    let head = &buf[..limit];

    let contains_sub = |pattern: &[u8]| -> bool {
        head.windows(pattern.len()).any(|w| w == pattern)
    };

    // 1. PHP, Templates (HBS, Jinja, EJS) & HTML detection
    if head.starts_with(b"<?php") {
        return Some(make_core("php", "text", None, "php", &["php", "phtml"], "sig.text.php"));
    }
    if (trimmed.contains("{{#if ") || trimmed.contains("{{#each ")) && trimmed.contains("{{/") {
        return Some(make_core("hbs", "text", None, "hbs", &["hbs", "handlebars"], "sig.text.handlebars"));
    }
    if (trimmed.contains("{% endif %}") || trimmed.contains("{% endfor %}")) && trimmed.contains("{%") {
        return Some(make_core("liquid", "text", None, "liquid", &["liquid"], "sig.text.liquid"));
    }
    if (trimmed.contains("{% block ") || trimmed.contains("{% extends ") || trimmed.contains("{% for "))
        && trimmed.contains("%}")
    {
        return Some(make_core("jinja", "text", None, "j2", &["j2", "jinja", "jinja2"], "sig.text.jinja"));
    }
    if (trimmed.contains("<%=") || trimmed.contains("<%-") || trimmed.contains("<% ")) && trimmed.contains("%>") {
        return Some(make_core("ejs", "text", None, "ejs", &["ejs"], "sig.text.ejs"));
    }
    if head.starts_with(b"<virtualhost")
        || head.starts_with(b"<directory")
        || head.starts_with(b"<ifmodule")
    {
        return Some(make_core("htaccess", "configuration", None, "htaccess", &["htaccess", "conf"], "sig.text.apache"));
    }
    if head.starts_with(b"<!doctype html")
        || head.starts_with(b"<html")
        || head.starts_with(b"<head")
        || head.starts_with(b"<body")
    {
        return Some(make_core("html", "text", None, "html", &["html", "htm"], "sig.text.html"));
    }

    let is_xml_head = head.starts_with(b"<?xml");

    // 2. Specialized XML dialects (SVG, KML, GPX, COLLADA DAE, Apple Plist) and SFCs (Vue, Svelte)
    if head.starts_with(b"<template") && (trimmed.contains("<script") || trimmed.contains("<style")) {
        return Some(make_core("vue", "text", None, "vue", &["vue"], "sig.text.vue"));
    }
    if head.starts_with(b"<script")
        && (trimmed.contains("export let ")
            || trimmed.contains("$:")
            || trimmed.contains("{#if ")
            || trimmed.contains("{#each "))
    {
        return Some(make_core("svelte", "text", None, "svelte", &["svelte"], "sig.text.svelte"));
    }
    if head.starts_with(b"<svg") || (is_xml_head && contains_sub(b"<svg")) {
        return Some(make_core("svg", "image", Some("xml"), "svg", &["svg"], "sig.text.svg"));
    }
    if head.starts_with(b"<kml") || (is_xml_head && contains_sub(b"<kml")) {
        return Some(make_core("kml", "geospatial", Some("xml"), "kml", &["kml"], "sig.text.kml"));
    }
    if head.starts_with(b"<gpx") || (is_xml_head && contains_sub(b"<gpx")) {
        return Some(make_core("gpx", "geospatial", Some("xml"), "gpx", &["gpx"], "sig.text.gpx"));
    }
    if head.starts_with(b"<collada") || (is_xml_head && contains_sub(b"<collada")) {
        return Some(make_core("dae", "model", Some("xml"), "dae", &["dae"], "sig.text.collada"));
    }
    if head.starts_with(b"<!doctype plist") || head.starts_with(b"<plist") || (is_xml_head && contains_sub(b"<plist")) {
        return Some(make_core("plist", "configuration", Some("xml"), "plist", &["plist"], "sig.text.plist"));
    }
    if (head.starts_with(b"<project") || (is_xml_head && contains_sub(b"<project")))
        && (contains_sub(b"<propertygroup") || contains_sub(b"sdk="))
    {
        return Some(make_core("csproj", "text", Some("xml"), "csproj", &["csproj", "fsproj", "vbproj"], "sig.text.csproj"));
    }
    if head.starts_with(b"<rss") || (is_xml_head && contains_sub(b"<rss")) {
        return Some(make_core("rss", "text", Some("xml"), "rss", &["rss", "xml"], "sig.text.rss"));
    }
    if (head.starts_with(b"<feed") || (is_xml_head && contains_sub(b"<feed"))) && contains_sub(b"2005/atom") {
        return Some(make_core("atom", "text", Some("xml"), "atom", &["atom", "xml"], "sig.text.atom"));
    }
    if head.starts_with(b"<opml") || (is_xml_head && contains_sub(b"<opml")) {
        return Some(make_core("opml", "text", Some("xml"), "opml", &["opml", "xml"], "sig.text.opml"));
    }
    if head.starts_with(b"<xsl:stylesheet")
        || head.starts_with(b"<xsl:transform")
        || (is_xml_head && (contains_sub(b"<xsl:stylesheet") || contains_sub(b"<xsl:transform")))
    {
        return Some(make_core("xslt", "text", Some("xml"), "xslt", &["xslt", "xsl"], "sig.text.xslt"));
    }
    if head.starts_with(b"<xs:schema")
        || head.starts_with(b"<xsd:schema")
        || (is_xml_head && (contains_sub(b"<xs:schema") || contains_sub(b"<xsd:schema")))
    {
        return Some(make_core("xsd", "text", Some("xml"), "xsd", &["xsd", "xml"], "sig.text.xsd"));
    }
    if (head.starts_with(b"<definitions") || (is_xml_head && contains_sub(b"<definitions")))
        && (trimmed.contains("schemas.xmlsoap.org/wsdl") || trimmed.contains("xmlns:wsdl"))
    {
        return Some(make_core("wsdl", "text", Some("xml"), "wsdl", &["wsdl", "xml"], "sig.text.wsdl"));
    }
    if (head.starts_with(b"<project") || (is_xml_head && contains_sub(b"<project")))
        && trimmed.contains("<modelVersion>4.0.0</modelVersion>")
        && trimmed.contains("<artifactId>")
    {
        return Some(make_core("pom", "text", Some("xml"), "xml", &["xml", "pom"], "sig.text.pom"));
    }
    if head.starts_with(b"<mjml") && trimmed.contains("<mj-body") {
        return Some(make_core("mjml", "text", Some("xml"), "mjml", &["mjml"], "sig.text.mjml"));
    }
    if (head.starts_with(b"<window")
        || head.starts_with(b"<usercontrol")
        || head.starts_with(b"<application")
        || (is_xml_head && (contains_sub(b"<window") || contains_sub(b"<usercontrol"))))
        && trimmed.contains("schemas.microsoft.com/winfx/2006/xaml")
    {
        return Some(make_core("xaml", "text", Some("xml"), "xaml", &["xaml", "xml"], "sig.text.xaml"));
    }
    if (head.starts_with(b"<package") || (is_xml_head && contains_sub(b"<package")))
        && trimmed.contains("<metadata>")
        && trimmed.contains("<version>")
    {
        return Some(make_core("nuspec", "package", Some("xml"), "nuspec", &["nuspec"], "sig.text.nuspec"));
    }
    if (head.starts_with(b"<math") || (is_xml_head && contains_sub(b"<math")))
        && trimmed.contains("w3.org/1998/Math/MathML")
    {
        return Some(make_core("mathml", "document", Some("xml"), "mml", &["mml", "mathml", "xml"], "sig.text.mathml"));
    }
    if (head.starts_with(b"<urlset") || head.starts_with(b"<sitemapindex") || (is_xml_head && (contains_sub(b"<urlset") || contains_sub(b"<sitemapindex"))))
        && trimmed.contains("sitemaps.org/schemas/sitemap")
    {
        return Some(make_core("sitemap", "text", Some("xml"), "xml", &["xml"], "sig.text.sitemap"));
    }
    if (head.starts_with(b"<root") || (is_xml_head && contains_sub(b"<root")))
        && trimmed.contains("System.Resources.ResXResourceReader")
    {
        return Some(make_core("resx", "text", Some("xml"), "resx", &["resx", "xml"], "sig.text.resx"));
    }
    if (head.starts_with(b"<application") || (is_xml_head && contains_sub(b"<application")))
        && trimmed.contains("wadl.dev.java.net")
    {
        return Some(make_core("wadl", "text", Some("xml"), "wadl", &["wadl", "xml"], "sig.text.wadl"));
    }
    if (head.starts_with(b"<xbel") || (is_xml_head && contains_sub(b"<xbel")))
        && trimmed.contains("<bookmark")
    {
        return Some(make_core("xbel", "text", Some("xml"), "xbel", &["xbel", "xml"], "sig.text.xbel"));
    }
    if (head.starts_with(b"<book")
        || head.starts_with(b"<article")
        || (is_xml_head && (contains_sub(b"<book") || contains_sub(b"<article"))))
        && (trimmed.contains("docbook.org/ns/docbook") || trimmed.contains("OASIS//DTD DocBook"))
    {
        return Some(make_core("docbook", "document", Some("xml"), "xml", &["xml", "dbk"], "sig.text.docbook"));
    }
    if (head.starts_with(b"<ivy-module") || (is_xml_head && contains_sub(b"<ivy-module")))
        && trimmed.contains("<info")
    {
        return Some(make_core("ivy", "text", Some("xml"), "xml", &["xml"], "sig.text.ivy"));
    }
    if (head.starts_with(b"<testsuite") || (is_xml_head && contains_sub(b"<testsuite")))
        && trimmed.contains("<testcase")
    {
        return Some(make_core("junit", "data", Some("xml"), "xml", &["xml"], "sig.text.junit"));
    }
    if (head.starts_with(b"<coverage") || (is_xml_head && contains_sub(b"<coverage")))
        && trimmed.contains("<packages")
        && trimmed.contains("line-rate=")
    {
        return Some(make_core("cobertura", "data", Some("xml"), "xml", &["xml"], "sig.text.cobertura"));
    }
    if (head.starts_with(b"<checkstyle") || (is_xml_head && contains_sub(b"<checkstyle")))
        && trimmed.contains("<file")
        && trimmed.contains("<error")
    {
        return Some(make_core("checkstyle", "data", Some("xml"), "xml", &["xml"], "sig.text.checkstyle"));
    }

    // 3. Generic XML detection
    if is_xml_head
        || head.starts_with(b"<feed")
        || head.starts_with(b"<rss")
        || (head.starts_with(b"<") && contains_sub(b"xmlns="))
    {
        return Some(make_core("xml", "text", None, "xml", &["xml"], "sig.text.xml"));
    }

    None
}

/// Probes whether the text represents an XML or HTML document and specialized dialects.
pub fn probe_xml_or_html(text: &str) -> Option<DetectionReport> {
    probe_xml_or_html_core(text).map(|core| core.to_report(Default::default(), ExtensionStatus::Undetermined))
}

fn make_core(
    fmt: &'static str,
    cat: &'static str,
    cont: Option<&'static str>,
    rec: &'static str,
    exts: &'static [&'static str],
    ev: &'static str,
) -> DetectionCore {
    DetectionCore {
        format_id: Some(fmt),
        category: cat,
        container_id: cont,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some(rec),
        accepted_extensions: exts,
        primary_evidence: Some(ev),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}
