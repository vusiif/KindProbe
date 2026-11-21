use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub enum PcapMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes Packet Capture formats: classic libpcap (PCAP) and PCAP Next Generation (PCAPNG).
pub fn probe_pcap(header: &[u8]) -> PcapMatch {
    if header.len() < 4 {
        if !header.is_empty() && (header[0] == 0xD4 || header[0] == 0xA1 || header[0] == 0x0A) {
            return PcapMatch::Truncated("pcap".to_string());
        }
        return PcapMatch::None;
    }

    let magic = &header[0..4];

    // 1. PCAPNG: Section Header Block type 0x0A0D0D0A
    if magic == [0x0A, 0x0D, 0x0D, 0x0A] {
        let mut evidence = vec![EvidenceItem::new("sig.pcapng.shb")];
        let verification = if header.len() >= 12 {
            let bom = &header[8..12];
            if bom == [0x1A, 0x2B, 0x3C, 0x4D] || bom == [0x4D, 0x3C, 0x2B, 0x1A] {
                evidence.push(EvidenceItem::new("sig.pcapng.bom"));
                Verification::Structure
            } else {
                Verification::Signature
            }
        } else {
            Verification::Signature
        };

        return PcapMatch::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some("pcapng".to_string()),
            category: "capture".to_string(),
            container_id: None,
            identification: Identification::Format,
            verification,
            inspection: Inspection::Complete,
            recommended_extension: Some("pcapng".to_string()),
            accepted_extensions: vec!["pcapng".to_string(), "ntar".to_string()],
            extension_status: ExtensionStatus::Undetermined,
            evidence,
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    // 2. Classic PCAP (microsecond & nanosecond resolutions, LE & BE)
    let is_pcap = magic == [0xD4, 0xC3, 0xB2, 0xA1]
        || magic == [0xA1, 0xB2, 0xC3, 0xD4]
        || magic == [0x4D, 0x3C, 0xB2, 0xA1]
        || magic == [0xA1, 0xB2, 0x3C, 0x4D];

    if is_pcap {
        let mut evidence = vec![EvidenceItem::new("sig.pcap.magic")];
        let verification = if header.len() >= 8 {
            let is_le = magic[0] == 0xD4 || magic[0] == 0x4D;
            let (v_major, v_minor) = if is_le {
                (
                    u16::from_le_bytes([header[4], header[5]]),
                    u16::from_le_bytes([header[6], header[7]]),
                )
            } else {
                (
                    u16::from_be_bytes([header[4], header[5]]),
                    u16::from_be_bytes([header[6], header[7]]),
                )
            };

            if v_major == 2 && v_minor == 4 {
                evidence.push(EvidenceItem::new("sig.pcap.version"));
                Verification::Structure
            } else {
                Verification::Signature
            }
        } else {
            Verification::Signature
        };

        return PcapMatch::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some("pcap".to_string()),
            category: "capture".to_string(),
            container_id: None,
            identification: Identification::Format,
            verification,
            inspection: Inspection::Complete,
            recommended_extension: Some("pcap".to_string()),
            accepted_extensions: vec!["pcap".to_string(), "cap".to_string(), "dmp".to_string()],
            extension_status: ExtensionStatus::Undetermined,
            evidence,
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    PcapMatch::None
}
