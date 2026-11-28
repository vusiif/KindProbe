use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

pub const WASM_MAGIC: &[u8] = &[0x00, 0x61, 0x73, 0x6D]; // \0asm

pub enum WasmMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn make_wasm_core(verification: Verification) -> DetectionCore {
    DetectionCore {
        format_id: Some("wasm"),
        category: "executable",
        container_id: None,
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some("wasm"),
        accepted_extensions: &["wasm"],
        primary_evidence: Some("sig.wasm.magic"),
        secondary_evidence: if verification == Verification::Structure {
            Some("sig.wasm.version")
        } else {
            None
        },
        warning: None,
        candidate: None,
    }
}

pub fn probe_wasm_core(header: &[u8]) -> Option<DetectionCore> {
    if header.len() < 4 || !header.starts_with(WASM_MAGIC) {
        return None;
    }

    let verification = if header.len() >= 8 {
        let version = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        if version == 1 {
            Verification::Structure
        } else {
            Verification::Signature
        }
    } else {
        Verification::Signature
    };

    Some(make_wasm_core(verification))
}

/// Probes WebAssembly binary format (WASM).
pub fn probe_wasm(header: &[u8]) -> WasmMatch {
    if header.len() < 4 {
        if !header.is_empty() && header.starts_with(&[0x00, 0x61]) {
            return WasmMatch::Truncated("wasm".to_string());
        }
        return WasmMatch::None;
    }

    match probe_wasm_core(header) {
        Some(core) => WasmMatch::Confirmed(Box::new(
            core.to_report(Default::default(), ExtensionStatus::Undetermined),
        )),
        None => WasmMatch::None,
    }
}
