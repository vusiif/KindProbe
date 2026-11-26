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

    // 1. PHP & HTML detection
    if head.starts_with(b"<?php") {
        return Some(make_core("php", "text", None, "php", &["php", "phtml"], "sig.text.php"));
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
