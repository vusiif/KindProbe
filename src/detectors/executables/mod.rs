pub mod elf;
pub mod macho;
pub mod pe;
pub mod pe_meta;
pub mod wasm;

use crate::model::{DetectionCore, DetectionReport};
use elf::{probe_elf, probe_elf_core, ElfMatch};
use macho::{probe_macho, probe_macho_core, MachoMatch};
use pe::{probe_pe, probe_pe_core, PeMatch};
use wasm::{probe_wasm, probe_wasm_core, WasmMatch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutableMatchCore {
    Confirmed(DetectionCore),
    Truncated(&'static str),
    None,
}

pub enum ExecutableMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes executable and system binary signatures returning zero-allocation DetectionCore.
pub fn probe_executable_signatures_core(header: &[u8]) -> ExecutableMatchCore {
    // 1. ELF
    if header.len() < 16 && !header.is_empty() && header.starts_with(&[0x7F, 0x45]) {
        return ExecutableMatchCore::Truncated("elf");
    }
    if let Some(core) = probe_elf_core(header) {
        return ExecutableMatchCore::Confirmed(core);
    }

    // 2. PE / MZ
    if header.len() < 2 && !header.is_empty() && header[0] == b'M' {
        return ExecutableMatchCore::Truncated("pe");
    }
    if let Some(core) = probe_pe_core(header) {
        return ExecutableMatchCore::Confirmed(core);
    }

    // 3. WASM
    if header.len() < 4 && !header.is_empty() && header.starts_with(&[0x00, 0x61]) {
        return ExecutableMatchCore::Truncated("wasm");
    }
    if let Some(core) = probe_wasm_core(header) {
        return ExecutableMatchCore::Confirmed(core);
    }

    // 4. Mach-O
    if header.len() < 4 && !header.is_empty() && (header.starts_with(&[0xFE, 0xED]) || header.starts_with(&[0xCF, 0xFA])) {
        return ExecutableMatchCore::Truncated("macho");
    }
    if let Some(core) = probe_macho_core(header) {
        return ExecutableMatchCore::Confirmed(core);
    }

    ExecutableMatchCore::None
}

/// Probes executable and system binary signatures (ELF, PE, WASM, Mach-O).
pub fn probe_executable_signatures(header: &[u8]) -> ExecutableMatch {
    // 1. ELF (Linux / BSD)
    match probe_elf(header) {
        ElfMatch::Confirmed(rep) => return ExecutableMatch::Confirmed(rep),
        ElfMatch::Truncated(fmt) => return ExecutableMatch::Truncated(fmt),
        ElfMatch::None => (),
    }

    // 2. Windows PE / DOS MZ
    match probe_pe(header) {
        PeMatch::Confirmed(rep) => return ExecutableMatch::Confirmed(rep),
        PeMatch::Truncated(fmt) => return ExecutableMatch::Truncated(fmt),
        PeMatch::None => (),
    }

    // 3. WebAssembly (WASM)
    match probe_wasm(header) {
        WasmMatch::Confirmed(rep) => return ExecutableMatch::Confirmed(rep),
        WasmMatch::Truncated(fmt) => return ExecutableMatch::Truncated(fmt),
        WasmMatch::None => (),
    }

    // 4. Mach-O / Java Class (macOS / Darwin)
    match probe_macho(header) {
        MachoMatch::Confirmed(rep) => return ExecutableMatch::Confirmed(rep),
        MachoMatch::Truncated(fmt) => return ExecutableMatch::Truncated(fmt),
        MachoMatch::None => (),
    }

    ExecutableMatch::None
}
