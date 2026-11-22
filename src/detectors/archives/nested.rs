use std::io::Read;
use flate2::read::GzDecoder;

use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};
use super::gzip::GZIP_MAGIC;
use super::tar::check_tar_header;

/// Probes nested archive stream within compressed containers (e.g. tar.gz).
pub fn probe_nested_gzip(header: &[u8]) -> Option<DetectionReport> {
    if header.len() < 10 || !header.starts_with(GZIP_MAGIC) {
        return None;
    }

    // Try decompressing at most 512 bytes to inspect the embedded stream header
    let mut decoder = GzDecoder::new(header);
    let mut tar_block = [0u8; 512];
    if decoder.read_exact(&mut tar_block).is_err() {
        return None;
    }

    let ver = check_tar_header(&tar_block)?;

    let mut evidence = vec![
        EvidenceItem::new("sig.gzip.magic"),
        EvidenceItem::new("sig.gzip.header"),
        EvidenceItem::with_desc(
            "sig.tar.embedded",
            "Embedded POSIX ustar archive detected inside compression stream",
        ),
        EvidenceItem::new("sig.tar.ustar"),
    ];
    if ver == Verification::Structure {
        evidence.push(EvidenceItem::new("sig.tar.checksum"));
    }

    Some(DetectionReport {
        schema_version: 1,
        format_id: Some("tar.gz".to_string()),
        category: "archive".to_string(),
        container_id: Some("gzip".to_string()),
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("tar.gz".to_string()),
        accepted_extensions: vec![
            "tar.gz".to_string(),
            "tgz".to_string(),
            "gz".to_string(),
        ],
        extension_status: ExtensionStatus::Undetermined,
        evidence,
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    })
}
