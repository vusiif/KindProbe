use crate::model::EvidenceItem;

/// Parses Windows PE Architecture, Characteristics, and Subsystems.
pub fn extract_pe_metadata(header: &[u8], pe_offset: usize) -> (bool, Vec<EvidenceItem>) {
    let mut items = Vec::new();
    let mut is_dll = false;

    // COFF Header starts at pe_offset + 4 (length: 20 bytes)
    if let Some(opt_start) = pe_offset.checked_add(24).filter(|&end| end <= header.len()) {
        let machine = u16::from_le_bytes([header[pe_offset + 4], header[pe_offset + 5]]);
        if let Some(arch_ev) = match_pe_machine(machine) {
            items.push(arch_ev);
        }

        let characteristics = u16::from_le_bytes([header[pe_offset + 22], header[pe_offset + 23]]);
        is_dll = (characteristics & 0x2000) != 0; // IMAGE_FILE_DLL

        let size_of_opt = u16::from_le_bytes([header[pe_offset + 20], header[pe_offset + 21]]) as usize;

        if size_of_opt >= 2 && opt_start.checked_add(2).is_some_and(|end| end <= header.len()) {
            let opt_magic = u16::from_le_bytes([header[opt_start], header[opt_start + 1]]);
            if opt_magic == 0x010B {
                items.push(EvidenceItem::with_desc("sig.pe.opt.pe32", "PE32 (32-bit address space)"));
            } else if opt_magic == 0x020B {
                items.push(EvidenceItem::with_desc("sig.pe.opt.pe32plus", "PE32+ (64-bit address space)"));
            }
        }

        if size_of_opt >= 70 && opt_start.checked_add(70).is_some_and(|end| end <= header.len()) {
            let subsystem = u16::from_le_bytes([header[opt_start + 68], header[opt_start + 69]]);
            match subsystem {
                2 => items.push(EvidenceItem::with_desc("sig.pe.subsystem.gui", "Windows GUI Subsystem")),
                3 => items.push(EvidenceItem::with_desc("sig.pe.subsystem.console", "Windows Console / CUI Subsystem")),
                _ => {}
            }
        }
    }

    (is_dll, items)
}

fn match_pe_machine(machine: u16) -> Option<EvidenceItem> {
    match machine {
        0x014C => Some(EvidenceItem::with_desc("sig.pe.arch.x86", "Target Architecture: x86 (Intel 386 / 32-bit)")),
        0x8664 => Some(EvidenceItem::with_desc("sig.pe.arch.x64", "Target Architecture: x86-64 (AMD64 / 64-bit)")),
        0xAA64 => Some(EvidenceItem::with_desc("sig.pe.arch.arm64", "Target Architecture: ARM64 (AArch64 / 64-bit)")),
        0xA641 => Some(EvidenceItem::with_desc("sig.pe.arch.arm64ec", "Target Architecture: ARM64EC (Emulation Compatible)")),
        0x01C0 => Some(EvidenceItem::with_desc("sig.pe.arch.arm", "Target Architecture: ARM Little-Endian")),
        0x01C4 => Some(EvidenceItem::with_desc("sig.pe.arch.arm_thumb2", "Target Architecture: ARMv7 Thumb-2 (Windows RT)")),
        0x0200 => Some(EvidenceItem::with_desc("sig.pe.arch.ia64", "Target Architecture: Intel Itanium (IA-64)")),
        0x5032 => Some(EvidenceItem::with_desc("sig.pe.arch.riscv32", "Target Architecture: RISC-V 32-bit")),
        0x5064 => Some(EvidenceItem::with_desc("sig.pe.arch.riscv64", "Target Architecture: RISC-V 64-bit")),
        0x5128 => Some(EvidenceItem::with_desc("sig.pe.arch.riscv128", "Target Architecture: RISC-V 128-bit")),
        0x01F0 | 0x01F1 => Some(EvidenceItem::with_desc("sig.pe.arch.powerpc", "Target Architecture: PowerPC")),
        0x0266 | 0x0366 => Some(EvidenceItem::with_desc("sig.pe.arch.mips", "Target Architecture: MIPS")),
        _ => None,
    }
}
