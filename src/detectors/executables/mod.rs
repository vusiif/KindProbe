pub mod elf;
pub mod macho;
pub mod pe;
pub mod pe_meta;
pub mod wasm;

use crate::model::DetectionReport;
use elf::{probe_elf, ElfMatch};
use macho::{probe_macho, MachoMatch};
use pe::{probe_pe, PeMatch};
use wasm::{probe_wasm, WasmMatch};

pub enum ExecutableMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
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
