use std::io::Read;
use flate2::read::GzDecoder;

use crate::model::{
    DetectionCore, DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
    Verification,
};
use super::gzip::GZIP_MAGIC;
use super::tar::check_tar_header;

/// Probes nested archive stream returning zero-allocation DetectionCore.
pub fn probe_nested_gzip_core(header: &[u8]) -> Option<DetectionCore> {
    if header.len() < 10 || !header.starts_with(GZIP_MAGIC) {
        return None;
    }

    let mut decoder = GzDecoder::new(header);
    let mut decomp_buf = [0u8; 512];
    let n = decoder.read(&mut decomp_buf).ok()?;
    if n == 0 {
        return None;
    }

    if n == 512 && check_tar_header(&decomp_buf).is_some() {
        return Some(DetectionCore {
            format_id: Some("tar.gz"),
            category: "archive",
            container_id: Some("gzip"),
            identification: Identification::Format,
            verification: Verification::Structure,
            inspection: Inspection::Complete,
            recommended_extension: Some("tar.gz"),
            accepted_extensions: &["tar.gz", "tgz", "gz"],
            primary_evidence: Some("sig.gzip.magic"),
            secondary_evidence: Some("sig.tar.ustar"),
            warning: None,
            candidate: None,
        });
    }

    let limit = n.min(256);
    let mut buf = [0u8; 256];
    for (i, &b) in decomp_buf[..limit].iter().enumerate() {
        buf[i] = b.to_ascii_lowercase();
    }
    let head = &buf[..limit];
    let is_xml = head.starts_with(b"<?xml");
    let has_svg = head.windows(4).any(|w| w == b"<svg");

    if head.starts_with(b"<svg") || (is_xml && has_svg) {
        return Some(DetectionCore {
            format_id: Some("svgz"),
            category: "image",
            container_id: Some("gzip"),
            identification: Identification::Format,
            verification: Verification::Structure,
            inspection: Inspection::Complete,
            recommended_extension: Some("svgz"),
            accepted_extensions: &["svgz", "svg", "gz"],
            primary_evidence: Some("sig.gzip.magic"),
            secondary_evidence: Some("sig.gzip.header"),
            warning: None,
            candidate: None,
        });
    }

    None
}

/// Probes nested archive stream within compressed containers (e.g. tar.gz, svgz).
pub fn probe_nested_gzip(header: &[u8]) -> Option<DetectionReport> {
    if header.len() < 10 || !header.starts_with(GZIP_MAGIC) {
        return None;
    }

    let mut decoder = GzDecoder::new(header);
    let mut decomp_buf = [0u8; 512];
    let n = decoder.read(&mut decomp_buf).ok()?;
    if n == 0 {
        return None;
    }

    if n == 512 {
        if let Some(ver) = check_tar_header(&decomp_buf) {
            let mut ev = vec![
                EvidenceItem::new("sig.gzip.magic"),
                EvidenceItem::new("sig.gzip.header"),
                EvidenceItem::with_desc("sig.tar.embedded", "Embedded POSIX ustar archive detected"),
                EvidenceItem::new("sig.tar.ustar"),
            ];
            if ver == Verification::Structure {
                ev.push(EvidenceItem::new("sig.tar.checksum"));
            }
            return Some(make_nested_rep("tar.gz", "archive", "tar.gz", &["tar.gz", "tgz", "gz"], ev));
        }
    }

    let head_str = std::str::from_utf8(&decomp_buf[..n]).ok()?.to_ascii_lowercase();
    if head_str.starts_with("<svg") || (head_str.starts_with("<?xml") && head_str.contains("<svg")) {
        let ev = vec![
            EvidenceItem::new("sig.gzip.magic"),
            EvidenceItem::new("sig.gzip.header"),
            EvidenceItem::with_desc("sig.svg.embedded", "Embedded SVG image detected"),
        ];
        return Some(make_nested_rep("svgz", "image", "svgz", &["svgz", "svg", "gz"], ev));
    }

    None
}

fn make_nested_rep(fmt: &str, cat: &str, rec: &str, exts: &[&str], ev: Vec<EvidenceItem>) -> DetectionReport {
    DetectionReport {
        schema_version: 1,
        format_id: Some(fmt.to_string()),
        category: cat.to_string(),
        container_id: Some("gzip".to_string()),
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some(rec.to_string()),
        accepted_extensions: exts.iter().map(|s| s.to_string()).collect(),
        extension_status: ExtensionStatus::Undetermined,
        evidence: ev,
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }
}
