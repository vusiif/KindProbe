use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

pub enum TtfOtfMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn make_ttf_otf_core(is_otf: bool, verification: Verification) -> DetectionCore {
    let (fmt_id, ev) = if is_otf {
        ("otf", "sig.font.otf_magic")
    } else {
        ("ttf", "sig.font.ttf_magic")
    };
    DetectionCore {
        format_id: Some(fmt_id),
        category: "font",
        container_id: None,
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some(fmt_id),
        accepted_extensions: if is_otf { &["otf"] } else { &["ttf"] },
        primary_evidence: Some(ev),
        secondary_evidence: if verification == Verification::Structure {
            Some("sig.font.num_tables")
        } else {
            None
        },
        warning: None,
        candidate: None,
    }
}

pub fn probe_ttf_otf_core(header: &[u8]) -> Option<DetectionCore> {
    if header.len() < 4 {
        return None;
    }

    let magic = &header[0..4];
    let is_ttf = magic == [0x00, 0x01, 0x00, 0x00] || magic == b"true";
    let is_otf = magic == b"OTTO";

    if !is_ttf && !is_otf {
        return None;
    }

    // Disambiguate against Microsoft Access Jet/ACE Database (\x00\x01\x00\x00Standard...)
    if header.len() >= 19 && is_ttf && (&header[4..19] == b"Standard Jet DB" || &header[4..19] == b"Standard ACE DB") {
        return None;
    }

    let verification = if header.len() >= 12 {
        let num_tables = u16::from_be_bytes([header[4], header[5]]);
        if (1..=200).contains(&num_tables) {
            Verification::Structure
        } else {
            return None;
        }
    } else {
        Verification::Signature
    };

    Some(make_ttf_otf_core(is_otf, verification))
}

/// Probes TrueType (TTF) and OpenType (OTF) font signatures.
pub fn probe_ttf_otf(header: &[u8]) -> TtfOtfMatch {
    if header.len() < 4 {
        if !header.is_empty() && (header.starts_with(&[0x00, 0x01]) || header.starts_with(b"OT")) {
            return TtfOtfMatch::Truncated("ttf".to_string());
        }
        return TtfOtfMatch::None;
    }

    match probe_ttf_otf_core(header) {
        Some(core) => TtfOtfMatch::Confirmed(Box::new(
            core.to_report(Default::default(), ExtensionStatus::Undetermined),
        )),
        None => TtfOtfMatch::None,
    }
}
