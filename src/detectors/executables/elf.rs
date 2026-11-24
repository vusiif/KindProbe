use crate::model::{
    DetectionCore, DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
    Verification,
};

pub const ELF_MAGIC: &[u8] = &[0x7F, 0x45, 0x4C, 0x46];

pub enum ElfMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn make_elf_core(verification: Verification, rec_ext: &'static str) -> DetectionCore {
    DetectionCore {
        format_id: Some("elf"),
        category: "executable",
        container_id: None,
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some(rec_ext),
        accepted_extensions: &["elf", "so", "o", "bin"],
        primary_evidence: Some("sig.elf.magic"),
        secondary_evidence: if verification == Verification::Structure {
            Some("sig.elf.header")
        } else {
            None
        },
        warning: None,
        candidate: None,
    }
}

pub fn probe_elf_core(header: &[u8]) -> Option<DetectionCore> {
    if header.len() < 16 || !header.starts_with(ELF_MAGIC) {
        return None;
    }
    let (class, data, ver) = (header[4], header[5], header[6]);
    let valid_ident = (class == 1 || class == 2) && (data == 1 || data == 2) && ver == 1;

    let mut rec_ext = "elf";
    let verification = if valid_ident {
        if header.len() >= 20 {
            let is_le = data == 1;
            let e_type = if is_le {
                u16::from_le_bytes([header[16], header[17]])
            } else {
                u16::from_be_bytes([header[16], header[17]])
            };
            if e_type == 3 {
                rec_ext = "so";
            }
        }
        Verification::Structure
    } else {
        Verification::Signature
    };

    Some(make_elf_core(verification, rec_ext))
}

/// Probes ELF header (Executable and Linkable Format) and target CPU architectures.
pub fn probe_elf(header: &[u8]) -> ElfMatch {
    if header.len() < 4 {
        return if !header.is_empty() && header.starts_with(&[0x7F, 0x45]) {
            ElfMatch::Truncated("elf".to_string())
        } else {
            ElfMatch::None
        };
    }
    if !header.starts_with(ELF_MAGIC) {
        return ElfMatch::None;
    }
    if header.len() < 16 {
        return ElfMatch::Truncated("elf".to_string());
    }

    let mut evidence = vec![EvidenceItem::new("sig.elf.magic")];
    let (class, data, ver) = (header[4], header[5], header[6]);
    let valid_ident = (class == 1 || class == 2) && (data == 1 || data == 2) && ver == 1;

    let mut rec_ext = "elf";
    let verification = if valid_ident {
        evidence.push(EvidenceItem::new("sig.elf.header"));
        let class_tag = if class == 1 { "sig.elf.class.32bit" } else { "sig.elf.class.64bit" };
        evidence.push(EvidenceItem::with_desc(class_tag, if class == 1 { "ELF 32-bit" } else { "ELF 64-bit" }));

        if header.len() >= 20 {
            let is_le = data == 1;
            let e_type = if is_le { u16::from_le_bytes([header[16], header[17]]) } else { u16::from_be_bytes([header[16], header[17]]) };
            let e_machine = if is_le { u16::from_le_bytes([header[18], header[19]]) } else { u16::from_be_bytes([header[18], header[19]]) };

            if e_type == 3 {
                rec_ext = "so";
                evidence.push(EvidenceItem::with_desc("sig.elf.type.shared_object", "Shared Object (ET_DYN)"));
            } else if e_type == 2 {
                evidence.push(EvidenceItem::with_desc("sig.elf.type.executable", "Executable File (ET_EXEC)"));
            }
            if let Some(arch_ev) = match_elf_machine(e_machine) {
                evidence.push(arch_ev);
            }
        }
        Verification::Structure
    } else {
        Verification::Signature
    };

    ElfMatch::Confirmed(Box::new(DetectionReport {
        schema_version: 1,
        format_id: Some("elf".to_string()),
        category: "executable".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some(rec_ext.to_string()),
        accepted_extensions: vec!["elf".to_string(), "so".to_string(), "o".to_string(), "bin".to_string()],
        extension_status: ExtensionStatus::Undetermined,
        evidence,
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }))
}

fn match_elf_machine(machine: u16) -> Option<EvidenceItem> {
    const ARCHES: &[(u16, &str, &str)] = &[
        (0x03, "sig.elf.arch.x86", "Target Architecture: x86 (Intel 386 / 32-bit)"),
        (0x3E, "sig.elf.arch.x86_64", "Target Architecture: x86-64 (AMD64 / 64-bit)"),
        (0x28, "sig.elf.arch.arm", "Target Architecture: ARM (32-bit)"),
        (0xB7, "sig.elf.arch.arm64", "Target Architecture: ARM64 (AArch64 / 64-bit)"),
        (0xF3, "sig.elf.arch.riscv", "Target Architecture: RISC-V"),
        (0x08, "sig.elf.arch.mips", "Target Architecture: MIPS"),
        (0x14, "sig.elf.arch.powerpc", "Target Architecture: PowerPC 32-bit"),
        (0x15, "sig.elf.arch.powerpc64", "Target Architecture: PowerPC 64-bit"),
        (0x02, "sig.elf.arch.sparc", "Target Architecture: SPARC"),
    ];
    ARCHES.iter().find(|(m, _, _)| *m == machine).map(|(_, c, d)| EvidenceItem::with_desc(*c, *d))
}
