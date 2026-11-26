use crate::model::{
    DetectionCore, DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
    Verification,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mp4MatchCore {
    Confirmed(DetectionCore),
    Truncated(&'static str),
    None,
}

pub enum Mp4Match {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn make_mov_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("mov"),
        category: "video",
        container_id: Some("isobmff"),
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("mov"),
        accepted_extensions: &["mov", "qt"],
        primary_evidence: Some("sig.mp4.ftyp"),
        secondary_evidence: Some("sig.mov.brand"),
        warning: None,
        candidate: None,
    }
}

pub fn make_mov_legacy_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("mov"),
        category: "video",
        container_id: Some("isobmff"),
        identification: Identification::Format,
        verification: Verification::Signature,
        inspection: Inspection::Complete,
        recommended_extension: Some("mov"),
        accepted_extensions: &["mov", "qt"],
        primary_evidence: Some("sig.mov.header"),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}

pub fn make_m4a_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("m4a"),
        category: "audio",
        container_id: Some("isobmff"),
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("m4a"),
        accepted_extensions: &["m4a", "mp4"],
        primary_evidence: Some("sig.mp4.ftyp"),
        secondary_evidence: Some("sig.mp4.brand"),
        warning: None,
        candidate: None,
    }
}

pub fn make_isobmff_image_core(
    format_id: &'static str,
    exts: &'static [&'static str],
    secondary_evidence: &'static str,
) -> DetectionCore {
    DetectionCore {
        format_id: Some(format_id),
        category: "image",
        container_id: Some("isobmff"),
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some(format_id),
        accepted_extensions: exts,
        primary_evidence: Some("sig.mp4.ftyp"),
        secondary_evidence: Some(secondary_evidence),
        warning: None,
        candidate: None,
    }
}

pub fn make_mp4_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("mp4"),
        category: "video",
        container_id: Some("isobmff"),
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("mp4"),
        accepted_extensions: &["mp4", "m4v", "m4a"],
        primary_evidence: Some("sig.mp4.ftyp"),
        secondary_evidence: Some("sig.mp4.brand"),
        warning: None,
        candidate: None,
    }
}

pub fn probe_mp4_core(header: &[u8]) -> Mp4MatchCore {
    if header.len() < 8 {
        if header.len() >= 4 && (header.starts_with(&[0, 0, 0]) || header.starts_with(b"ftyp")) {
            return Mp4MatchCore::Truncated("mp4");
        }
        return Mp4MatchCore::None;
    }

    let box_size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
    let box_type = &header[4..8];

    if box_type == b"ftyp" {
        if box_size < 16 && box_size != 1 {
            return Mp4MatchCore::None;
        }

        if header.len() < 12 {
            return Mp4MatchCore::Truncated("mp4");
        }

        let major_brand = &header[8..12];
        if major_brand == b"qt  " {
            return Mp4MatchCore::Confirmed(make_mov_core());
        }

        if major_brand == b"M4A " {
            return Mp4MatchCore::Confirmed(make_m4a_core());
        }

        if major_brand == b"M4B " {
            return Mp4MatchCore::Confirmed(DetectionCore {
                format_id: Some("m4b"),
                category: "audio",
                container_id: Some("isobmff"),
                identification: Identification::Format,
                verification: Verification::Structure,
                inspection: Inspection::Complete,
                recommended_extension: Some("m4b"),
                accepted_extensions: &["m4b", "m4a"],
                primary_evidence: Some("sig.mp4.ftyp"),
                secondary_evidence: Some("sig.m4b.brand"),
                warning: None,
                candidate: None,
            });
        }

        if matches!(major_brand, b"3gp4" | b"3gp5" | b"3gp6" | b"3gp7" | b"3gp8" | b"3gp9") {
            return Mp4MatchCore::Confirmed(DetectionCore {
                format_id: Some("3gp"),
                category: "video",
                container_id: Some("isobmff"),
                identification: Identification::Format,
                verification: Verification::Structure,
                inspection: Inspection::Complete,
                recommended_extension: Some("3gp"),
                accepted_extensions: &["3gp", "3gpp"],
                primary_evidence: Some("sig.mp4.ftyp"),
                secondary_evidence: Some("sig.3gp.brand"),
                warning: None,
                candidate: None,
            });
        }

        if matches!(major_brand, b"3g2a" | b"3g2b" | b"3g2c") {
            return Mp4MatchCore::Confirmed(DetectionCore {
                format_id: Some("3g2"),
                category: "video",
                container_id: Some("isobmff"),
                identification: Identification::Format,
                verification: Verification::Structure,
                inspection: Inspection::Complete,
                recommended_extension: Some("3g2"),
                accepted_extensions: &["3g2"],
                primary_evidence: Some("sig.mp4.ftyp"),
                secondary_evidence: Some("sig.3g2.brand"),
                warning: None,
                candidate: None,
            });
        }

        if major_brand == b"crx " {
            return Mp4MatchCore::Confirmed(make_isobmff_image_core(
                "cr3",
                &["cr3"],
                "sig.cr3.brand",
            ));
        }

        if major_brand == b"avif" || major_brand == b"avis" {
            return Mp4MatchCore::Confirmed(make_isobmff_image_core(
                "avif",
                &["avif", "avifs"],
                "sig.avif.brand",
            ));
        }

        if matches!(
            major_brand,
            b"heic" | b"heix" | b"hevc" | b"hevx" | b"heim" | b"heis"
        ) {
            return Mp4MatchCore::Confirmed(make_isobmff_image_core(
                "heic",
                &["heic", "heif"],
                "sig.heic.brand",
            ));
        }

        if major_brand == b"mif1" || major_brand == b"msf1" || major_brand == b"heif" {
            return Mp4MatchCore::Confirmed(make_isobmff_image_core(
                "heif",
                &["heif", "heic"],
                "sig.heif.brand",
            ));
        }

        return Mp4MatchCore::Confirmed(make_mp4_core());
    }

    if box_type == b"moov" || box_type == b"wide" || box_type == b"free" {
        return Mp4MatchCore::Confirmed(make_mov_legacy_core());
    }

    Mp4MatchCore::None
}

/// Probes ISO-BMFF box structure for MP4 and QuickTime MOV.
pub fn probe_mp4(header: &[u8]) -> Mp4Match {
    // 1. Truncation checks
    if header.len() < 8 {
        if header.len() >= 4 && (header.starts_with(&[0, 0, 0]) || header.starts_with(b"ftyp")) {
            return Mp4Match::Truncated("mp4".to_string());
        }
        return Mp4Match::None;
    }

    let box_size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
    let box_type = &header[4..8];

    // Check for ftyp box
    if box_type == b"ftyp" {
        if box_size < 16 && box_size != 1 {
            return Mp4Match::None;
        }

        // Header must be at least 12 bytes to inspect major brand
        if header.len() < 12 {
            return Mp4Match::Truncated("mp4".to_string());
        }

        let major_brand = &header[8..12];
        let mut evidence = vec![EvidenceItem::new("sig.mp4.ftyp")];

        // Check if QuickTime MOV: major_brand == b"qt  "
        if major_brand == b"qt  " {
            evidence.push(EvidenceItem::new("sig.mov.brand"));
            return Mp4Match::Confirmed(Box::new(DetectionReport {
                schema_version: 1,
                format_id: Some("mov".to_string()),
                category: "video".to_string(),
                container_id: Some("isobmff".to_string()),
                identification: Identification::Format,
                verification: Verification::Structure,
                inspection: Inspection::Complete,
                recommended_extension: Some("mov".to_string()),
                accepted_extensions: vec!["mov".to_string(), "qt".to_string()],
                extension_status: ExtensionStatus::Undetermined,
                evidence,
                warnings: Vec::new(),
                stats: Default::default(),
                candidates: None,
            }));
        }

        if major_brand == b"avif" || major_brand == b"avis" {
            return Mp4Match::Confirmed(Box::new(
                make_isobmff_image_core("avif", &["avif", "avifs"], "sig.avif.brand")
                    .to_report(Default::default(), ExtensionStatus::Undetermined),
            ));
        }

        if matches!(
            major_brand,
            b"heic" | b"heix" | b"hevc" | b"hevx" | b"heim" | b"heis"
        ) {
            return Mp4Match::Confirmed(Box::new(
                make_isobmff_image_core("heic", &["heic", "heif"], "sig.heic.brand")
                    .to_report(Default::default(), ExtensionStatus::Undetermined),
            ));
        }

        if major_brand == b"mif1" || major_brand == b"msf1" || major_brand == b"heif" {
            return Mp4Match::Confirmed(Box::new(
                make_isobmff_image_core("heif", &["heif", "heic"], "sig.heif.brand")
                    .to_report(Default::default(), ExtensionStatus::Undetermined),
            ));
        }

        if major_brand == b"M4B " {
            return Mp4Match::Confirmed(Box::new(
                DetectionCore {
                    format_id: Some("m4b"),
                    category: "audio",
                    container_id: Some("isobmff"),
                    identification: Identification::Format,
                    verification: Verification::Structure,
                    inspection: Inspection::Complete,
                    recommended_extension: Some("m4b"),
                    accepted_extensions: &["m4b", "m4a"],
                    primary_evidence: Some("sig.mp4.ftyp"),
                    secondary_evidence: Some("sig.m4b.brand"),
                    warning: None,
                    candidate: None,
                }
                .to_report(Default::default(), ExtensionStatus::Undetermined),
            ));
        }

        if matches!(major_brand, b"3gp4" | b"3gp5" | b"3gp6" | b"3gp7" | b"3gp8" | b"3gp9") {
            return Mp4Match::Confirmed(Box::new(
                DetectionCore {
                    format_id: Some("3gp"),
                    category: "video",
                    container_id: Some("isobmff"),
                    identification: Identification::Format,
                    verification: Verification::Structure,
                    inspection: Inspection::Complete,
                    recommended_extension: Some("3gp"),
                    accepted_extensions: &["3gp", "3gpp"],
                    primary_evidence: Some("sig.mp4.ftyp"),
                    secondary_evidence: Some("sig.3gp.brand"),
                    warning: None,
                    candidate: None,
                }
                .to_report(Default::default(), ExtensionStatus::Undetermined),
            ));
        }

        if matches!(major_brand, b"3g2a" | b"3g2b" | b"3g2c") {
            return Mp4Match::Confirmed(Box::new(
                DetectionCore {
                    format_id: Some("3g2"),
                    category: "video",
                    container_id: Some("isobmff"),
                    identification: Identification::Format,
                    verification: Verification::Structure,
                    inspection: Inspection::Complete,
                    recommended_extension: Some("3g2"),
                    accepted_extensions: &["3g2"],
                    primary_evidence: Some("sig.mp4.ftyp"),
                    secondary_evidence: Some("sig.3g2.brand"),
                    warning: None,
                    candidate: None,
                }
                .to_report(Default::default(), ExtensionStatus::Undetermined),
            ));
        }

        if major_brand == b"crx " {
            return Mp4Match::Confirmed(Box::new(
                make_isobmff_image_core("cr3", &["cr3"], "sig.cr3.brand")
                    .to_report(Default::default(), ExtensionStatus::Undetermined),
            ));
        }

        evidence.push(EvidenceItem::new("sig.mp4.brand"));
        let (format_id, accepted_exts, rec_ext) = if major_brand == b"M4A " {
            ("m4a", vec!["m4a".to_string(), "mp4".to_string()], "m4a")
        } else {
            ("mp4", vec!["mp4".to_string(), "m4v".to_string(), "m4a".to_string()], "mp4")
        };

        return Mp4Match::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some(format_id.to_string()),
            category: if format_id == "m4a" {
                "audio".to_string()
            } else {
                "video".to_string()
            },
            container_id: Some("isobmff".to_string()),
            identification: Identification::Format,
            verification: Verification::Structure,
            inspection: Inspection::Complete,
            recommended_extension: Some(rec_ext.to_string()),
            accepted_extensions: accepted_exts,
            extension_status: ExtensionStatus::Undetermined,
            evidence,
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    // Legacy QuickTime MOV starting with moov, wide, or free at file offset 0
    if box_type == b"moov" || box_type == b"wide" || box_type == b"free" {
        return Mp4Match::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some("mov".to_string()),
            category: "video".to_string(),
            container_id: Some("isobmff".to_string()),
            identification: Identification::Format,
            verification: Verification::Signature,
            inspection: Inspection::Complete,
            recommended_extension: Some("mov".to_string()),
            accepted_extensions: vec!["mov".to_string(), "qt".to_string()],
            extension_status: ExtensionStatus::Undetermined,
            evidence: vec![EvidenceItem::new("sig.mov.header")],
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    Mp4Match::None
}
