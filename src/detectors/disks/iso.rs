use std::io::{Read, Seek, SeekFrom};
use crate::model::{
    DetectionCore, DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
    Verification,
};

pub const ISO_OFFSET: u64 = 0x8000; // 32768 bytes (16 sectors of 2048 bytes)

pub enum IsoMatch {
    Confirmed(Box<DetectionReport>),
    None,
}

pub fn make_iso_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("iso"),
        category: "disk",
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("iso"),
        accepted_extensions: &["iso", "img"],
        primary_evidence: Some("sig.iso.volume_descriptor"),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}

pub fn probe_iso_buffer_core(header: &[u8]) -> Option<DetectionCore> {
    if header.len() >= (ISO_OFFSET as usize) + 6 {
        let pvd = &header[(ISO_OFFSET as usize)..(ISO_OFFSET as usize) + 6];
        if pvd == b"\x01CD001" || pvd == b"\x02CD001" {
            return Some(make_iso_core());
        }
    }
    None
}

fn make_iso_report() -> DetectionReport {
    DetectionReport {
        schema_version: 1,
        format_id: Some("iso".to_string()),
        category: "disk".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("iso".to_string()),
        accepted_extensions: vec!["iso".to_string(), "img".to_string()],
        extension_status: ExtensionStatus::Undetermined,
        evidence: vec![EvidenceItem::new("sig.iso.volume_descriptor")],
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }
}

/// Probes ISO 9660 volume descriptor from memory buffer if sufficiently large.
pub fn probe_iso_buffer(header: &[u8]) -> IsoMatch {
    if header.len() >= (ISO_OFFSET as usize) + 6 {
        let pvd = &header[(ISO_OFFSET as usize)..(ISO_OFFSET as usize) + 6];
        if pvd == b"\x01CD001" || pvd == b"\x02CD001" {
            return IsoMatch::Confirmed(Box::new(make_iso_report()));
        }
    }
    IsoMatch::None
}

/// Probes ISO 9660 volume descriptor from seekable stream.
pub fn probe_iso_stream<R: Read + Seek>(reader: &mut R) -> Option<DetectionReport> {
    if reader.seek(SeekFrom::Start(ISO_OFFSET)).is_ok() {
        let mut buf = [0u8; 6];
        if reader.read_exact(&mut buf).is_ok() && (&buf == b"\x01CD001" || &buf == b"\x02CD001") {
            return Some(make_iso_report());
        }
    }
    None
}
