use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

pub enum PcapMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn make_pcapng_core(verification: Verification) -> DetectionCore {
    DetectionCore {
        format_id: Some("pcapng"),
        category: "capture",
        container_id: None,
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some("pcapng"),
        accepted_extensions: &["pcapng", "ntar"],
        primary_evidence: Some("sig.pcapng.shb"),
        secondary_evidence: if verification == Verification::Structure {
            Some("sig.pcapng.bom")
        } else {
            None
        },
        warning: None,
        candidate: None,
    }
}

pub fn make_pcap_core(verification: Verification) -> DetectionCore {
    DetectionCore {
        format_id: Some("pcap"),
        category: "capture",
        container_id: None,
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some("pcap"),
        accepted_extensions: &["pcap", "cap", "dmp"],
        primary_evidence: Some("sig.pcap.magic"),
        secondary_evidence: if verification == Verification::Structure {
            Some("sig.pcap.version")
        } else {
            None
        },
        warning: None,
        candidate: None,
    }
}

pub fn probe_pcap_core(header: &[u8]) -> Option<DetectionCore> {
    if header.len() < 4 {
        return None;
    }

    let magic = &header[0..4];

    // 1. PCAPNG: Section Header Block type 0x0A0D0D0A
    if magic == [0x0A, 0x0D, 0x0D, 0x0A] {
        let verification = if header.len() >= 12 {
            let bom = &header[8..12];
            if bom == [0x1A, 0x2B, 0x3C, 0x4D] || bom == [0x4D, 0x3C, 0x2B, 0x1A] {
                Verification::Structure
            } else {
                Verification::Signature
            }
        } else {
            Verification::Signature
        };

        return Some(make_pcapng_core(verification));
    }

    // 2. Classic PCAP magic variants
    let is_pcap_micro_le = magic == [0xD4, 0xC3, 0xB2, 0xA1];
    let is_pcap_micro_be = magic == [0xA1, 0xB2, 0xC3, 0xD4];
    let is_pcap_nano_le = magic == [0x4D, 0x3C, 0xB2, 0xA1];
    let is_pcap_nano_be = magic == [0xA1, 0xB2, 0x3C, 0x4D];
    let is_pcap = is_pcap_micro_le || is_pcap_micro_be || is_pcap_nano_le || is_pcap_nano_be;

    if is_pcap {
        let is_le = is_pcap_micro_le || is_pcap_nano_le;
        let verification = if header.len() >= 8 {
            let ver_major = if is_le {
                u16::from_le_bytes([header[4], header[5]])
            } else {
                u16::from_be_bytes([header[4], header[5]])
            };
            let ver_minor = if is_le {
                u16::from_le_bytes([header[6], header[7]])
            } else {
                u16::from_be_bytes([header[6], header[7]])
            };
            if ver_major == 2 && ver_minor == 4 {
                Verification::Structure
            } else {
                Verification::Signature
            }
        } else {
            Verification::Signature
        };

        return Some(make_pcap_core(verification));
    }

    None
}

/// Probes Packet Capture formats: classic libpcap (PCAP) and PCAP Next Generation (PCAPNG).
pub fn probe_pcap(header: &[u8]) -> PcapMatch {
    if header.len() < 4 {
        if header.len() >= 2
            && (header.starts_with(&[0xD4, 0xC3])
                || header.starts_with(&[0xA1, 0xB2])
                || header.starts_with(&[0x4D, 0x3C])
                || header.starts_with(&[0x0A, 0x0D]))
        {
            return PcapMatch::Truncated("pcap".to_string());
        }
        return PcapMatch::None;
    }

    match probe_pcap_core(header) {
        Some(core) => PcapMatch::Confirmed(Box::new(
            core.to_report(Default::default(), ExtensionStatus::Undetermined),
        )),
        None => PcapMatch::None,
    }
}
