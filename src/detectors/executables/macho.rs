use crate::model::{
    DetectionCore, DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
    Verification,
};

pub enum MachoMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn make_macho_core(verification: Verification) -> DetectionCore {
    DetectionCore {
        format_id: Some("macho"),
        category: "executable",
        container_id: None,
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some("dylib"),
        accepted_extensions: &["dylib", "bundle", "so", "bin"],
        primary_evidence: Some("sig.macho.magic"),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}

pub fn make_macho_fat_core(verification: Verification) -> DetectionCore {
    DetectionCore {
        format_id: Some("macho"),
        category: "executable",
        container_id: None,
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some("dylib"),
        accepted_extensions: &["dylib", "bundle", "so", "bin"],
        primary_evidence: Some("sig.macho.fat_magic"),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}

pub fn make_java_class_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("class"),
        category: "executable",
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("class"),
        accepted_extensions: &["class"],
        primary_evidence: Some("sig.java.class"),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}

fn is_java_class_version(header: &[u8]) -> bool {
    if header.len() < 8 {
        return false;
    }
    let minor = u16::from_be_bytes([header[4], header[5]]);
    let major = u16::from_be_bytes([header[6], header[7]]);
    // Java class: minor == 0 (standard) or 0xFFFF (--enable-preview), major version 43..=100
    (minor == 0 || minor == 0xFFFF) && (43..=100).contains(&major)
}

pub fn probe_macho_core(header: &[u8]) -> Option<DetectionCore> {
    if header.len() < 4 {
        return None;
    }

    let magic = &header[..4];
    let is_macho_le = magic == [0xCE, 0xFA, 0xED, 0xFE] || magic == [0xCF, 0xFA, 0xED, 0xFE];
    let is_macho_be = magic == [0xFE, 0xED, 0xFA, 0xCE] || magic == [0xFE, 0xED, 0xFA, 0xCF];
    let ver = if header.len() >= 8 {
        Verification::Structure
    } else {
        Verification::Signature
    };

    if is_macho_le || is_macho_be {
        return Some(make_macho_core(ver));
    }

    if magic == [0xCA, 0xFE, 0xBA, 0xBE] {
        if is_java_class_version(header) {
            return Some(make_java_class_core());
        }
        return Some(make_macho_fat_core(ver));
    }

    if magic == [0xBE, 0xBA, 0xFE, 0xCA] {
        return Some(make_macho_fat_core(ver));
    }

    None
}

/// Probes Mach-O binary format (macOS / iOS / Darwin) and Java class disambiguation.
pub fn probe_macho(header: &[u8]) -> MachoMatch {
    if header.len() < 4 {
        return if header.len() >= 2
            && (header.starts_with(&[0xFE, 0xED]) || header.starts_with(&[0xCF, 0xFA]))
        {
            MachoMatch::Truncated("macho".to_string())
        } else {
            MachoMatch::None
        };
    }

    let magic = &header[..4];
    let is_macho_le = magic == [0xCE, 0xFA, 0xED, 0xFE] || magic == [0xCF, 0xFA, 0xED, 0xFE];
    let is_macho_be = magic == [0xFE, 0xED, 0xFA, 0xCE] || magic == [0xFE, 0xED, 0xFA, 0xCF];
    let ver = if header.len() >= 8 {
        Verification::Structure
    } else {
        Verification::Signature
    };

    if is_macho_le || is_macho_be {
        let mut evidence = vec![EvidenceItem::new("sig.macho.magic")];
        if header.len() >= 8 {
            let cpu = if is_macho_le {
                u32::from_le_bytes([header[4], header[5], header[6], header[7]])
            } else {
                u32::from_be_bytes([header[4], header[5], header[6], header[7]])
            };
            if let Some(arch_ev) = match_macho_cputype(cpu) {
                evidence.push(arch_ev);
            }
        }
        return build_macho_report("macho", "dylib", ver, evidence);
    }

    if magic == [0xCA, 0xFE, 0xBA, 0xBE] {
        if is_java_class_version(header) {
            return build_macho_report(
                "class",
                "class",
                Verification::Structure,
                vec![EvidenceItem::new("sig.java.class")],
            );
        }
        return build_macho_report(
            "macho",
            "dylib",
            ver,
            vec![
                EvidenceItem::new("sig.macho.fat_magic"),
                EvidenceItem::with_desc(
                    "sig.macho.universal",
                    "Universal Binary (Multi-architecture FAT)",
                ),
            ],
        );
    }

    if magic == [0xBE, 0xBA, 0xFE, 0xCA] {
        return build_macho_report(
            "macho",
            "dylib",
            ver,
            vec![
                EvidenceItem::new("sig.macho.fat_magic"),
                EvidenceItem::with_desc(
                    "sig.macho.universal",
                    "Universal Binary (Multi-architecture FAT)",
                ),
            ],
        );
    }

    MachoMatch::None
}

fn match_macho_cputype(cputype: u32) -> Option<EvidenceItem> {
    const ARCHES: &[(u32, &str, &str)] = &[
        (
            0x00000007,
            "sig.macho.arch.x86",
            "Target Architecture: x86 (i386)",
        ),
        (
            0x01000007,
            "sig.macho.arch.x86_64",
            "Target Architecture: x86-64 (AMD64)",
        ),
        (
            0x0000000C,
            "sig.macho.arch.arm",
            "Target Architecture: ARM (32-bit)",
        ),
        (
            0x0100000C,
            "sig.macho.arch.arm64",
            "Target Architecture: ARM64 (Apple Silicon)",
        ),
        (
            0x00000012,
            "sig.macho.arch.powerpc",
            "Target Architecture: PowerPC",
        ),
        (
            0x01000012,
            "sig.macho.arch.powerpc64",
            "Target Architecture: PowerPC 64-bit",
        ),
    ];
    ARCHES
        .iter()
        .find(|(c, _, _)| *c == cputype)
        .map(|(_, k, d)| EvidenceItem::with_desc(*k, *d))
}

fn build_macho_report(
    fmt: &str,
    ext: &str,
    verification: Verification,
    evidence: Vec<EvidenceItem>,
) -> MachoMatch {
    MachoMatch::Confirmed(Box::new(DetectionReport {
        schema_version: 1,
        format_id: Some(fmt.to_string()),
        category: "executable".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some(ext.to_string()),
        accepted_extensions: if fmt == "class" {
            vec!["class".to_string()]
        } else {
            vec![
                "dylib".to_string(),
                "bundle".to_string(),
                "so".to_string(),
                "bin".to_string(),
            ]
        },
        extension_status: ExtensionStatus::Undetermined,
        evidence,
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }))
}
