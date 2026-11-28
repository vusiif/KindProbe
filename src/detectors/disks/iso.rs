use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};
use std::io::{Read, Seek, SeekFrom};

pub const ISO_OFFSET: u64 = 0x8000; // 32768 bytes (16 sectors of 2048 bytes)

pub enum IsoMatch {
    Confirmed(Box<DetectionReport>),
    None,
}

impl IsoMatch {
    pub fn into_report(self) -> Option<DetectionReport> {
        match self {
            IsoMatch::Confirmed(rep) => Some(*rep),
            IsoMatch::None => None,
        }
    }
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

/// Probes ISO 9660 volume descriptor from memory buffer if sufficiently large.
pub fn probe_iso_buffer(header: &[u8]) -> IsoMatch {
    match probe_iso_buffer_core(header) {
        Some(core) => IsoMatch::Confirmed(Box::new(
            core.to_report(Default::default(), ExtensionStatus::Undetermined),
        )),
        None => IsoMatch::None,
    }
}

/// Probes ISO 9660 volume descriptor from seekable stream returning zero-allocation DetectionCore.
pub fn probe_iso_stream_core<R: Read + Seek>(reader: &mut R) -> Option<DetectionCore> {
    if reader.seek(SeekFrom::Start(ISO_OFFSET)).is_ok() {
        let mut buf = [0u8; 6];
        if reader.read_exact(&mut buf).is_ok() && (&buf == b"\x01CD001" || &buf == b"\x02CD001") {
            return Some(make_iso_core());
        }
    }
    None
}

/// Probes ISO 9660 volume descriptor from seekable stream.
pub fn probe_iso_stream<R: Read + Seek>(reader: &mut R) -> Option<DetectionReport> {
    probe_iso_stream_core(reader)
        .map(|core| core.to_report(Default::default(), ExtensionStatus::Undetermined))
}
