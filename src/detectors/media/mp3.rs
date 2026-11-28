use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mp3MatchCore {
    Confirmed(DetectionCore),
    Truncated(&'static str),
    None,
}

pub enum Mp3Match {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn make_mp3_id3_core(has_frame_sync: bool) -> DetectionCore {
    DetectionCore {
        format_id: Some("mp3"),
        category: "audio",
        container_id: None,
        identification: Identification::Format,
        verification: if has_frame_sync {
            Verification::Structure
        } else {
            Verification::Signature
        },
        inspection: Inspection::Complete,
        recommended_extension: Some("mp3"),
        accepted_extensions: &["mp3"],
        primary_evidence: Some("sig.id3.tag"),
        secondary_evidence: if has_frame_sync {
            Some("sig.mp3.frame_sync")
        } else {
            None
        },
        warning: None,
        candidate: None,
    }
}

pub fn make_mp3_raw_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("mp3"),
        category: "audio",
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Signature,
        inspection: Inspection::Complete,
        recommended_extension: Some("mp3"),
        accepted_extensions: &["mp3"],
        primary_evidence: Some("sig.mp3.frame_sync"),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}

pub fn probe_mp3_core(header: &[u8]) -> Mp3MatchCore {
    // 1. Truncation check
    if header.len() < 3 {
        if !header.is_empty() && header.starts_with(b"ID") {
            return Mp3MatchCore::Truncated("mp3");
        }
        return Mp3MatchCore::None;
    }

    // 2. ID3v2 container probe
    if header.starts_with(b"ID3") {
        if header.len() < 10 {
            return Mp3MatchCore::Truncated("mp3");
        }

        let major_ver = header[3];
        let synchsafe_valid =
            header[6] < 0x80 && header[7] < 0x80 && header[8] < 0x80 && header[9] < 0x80;

        if major_ver < 0xFF && synchsafe_valid {
            let mut has_frame_sync = false;
            for i in 10..header.len().saturating_sub(1) {
                if is_mp3_frame_sync(header[i], header[i + 1]) {
                    has_frame_sync = true;
                    break;
                }
            }

            return Mp3MatchCore::Confirmed(make_mp3_id3_core(has_frame_sync));
        }
    }

    // 3. Raw MPEG Audio Layer III frame sync without ID3 tag
    if header.len() >= 2 && is_mp3_frame_sync(header[0], header[1]) {
        // Extra validation on byte 2 if available: bitrate index not 0x0F, sample rate not 0x03
        if header.len() >= 3 {
            let b2 = header[2];
            let bitrate_idx = (b2 >> 4) & 0x0F;
            let sample_rate_idx = (b2 >> 2) & 0x03;
            if bitrate_idx == 0x0F || sample_rate_idx == 0x03 {
                return Mp3MatchCore::None;
            }
        }

        return Mp3MatchCore::Confirmed(make_mp3_raw_core());
    }

    Mp3MatchCore::None
}

/// Checks if two bytes represent a valid MPEG-1/2 Audio Layer III sync word.
fn is_mp3_frame_sync(b0: u8, b1: u8) -> bool {
    if b0 != 0xFF {
        return false;
    }
    // Top 3 bits of b1 must be 111 (frame sync 11 bits)
    if (b1 & 0xE0) != 0xE0 {
        return false;
    }
    // MPEG Audio Version: bits 3..4: 01 is reserved
    let version_bits = (b1 >> 3) & 0x03;
    if version_bits == 0x01 {
        return false;
    }
    // Layer: bits 1..2: 01 is Layer III
    let layer_bits = (b1 >> 1) & 0x03;
    layer_bits == 0x01
}

/// Probes ID3v2 tag and MPEG-1/2 Audio Layer III frame sync.
pub fn probe_mp3(header: &[u8]) -> Mp3Match {
    match probe_mp3_core(header) {
        Mp3MatchCore::Confirmed(core) => Mp3Match::Confirmed(Box::new(
            core.to_report(Default::default(), ExtensionStatus::Undetermined),
        )),
        Mp3MatchCore::Truncated(cand) => Mp3Match::Truncated(cand.to_string()),
        Mp3MatchCore::None => Mp3Match::None,
    }
}
