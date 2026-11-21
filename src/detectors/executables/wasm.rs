use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub const WASM_MAGIC: &[u8] = &[0x00, 0x61, 0x73, 0x6D]; // \0asm

pub enum WasmMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes WebAssembly binary format (WASM).
pub fn probe_wasm(header: &[u8]) -> WasmMatch {
    if header.len() < 4 {
        if !header.is_empty() && header.starts_with(&[0x00, 0x61]) {
            return WasmMatch::Truncated("wasm".to_string());
        }
        return WasmMatch::None;
    }

    if !header.starts_with(WASM_MAGIC) {
        return WasmMatch::None;
    }

    let mut evidence = vec![EvidenceItem::new("sig.wasm.magic")];

    // Check version at offset 4..8 (version 1 is [1, 0, 0, 0])
    let verification = if header.len() >= 8 {
        let version = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        if version == 1 {
            evidence.push(EvidenceItem::new("sig.wasm.version"));
            Verification::Structure
        } else {
            Verification::Signature
        }
    } else {
        Verification::Signature
    };

    WasmMatch::Confirmed(Box::new(DetectionReport {
        schema_version: 1,
        format_id: Some("wasm".to_string()),
        category: "executable".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some("wasm".to_string()),
        accepted_extensions: vec!["wasm".to_string()],
        extension_status: ExtensionStatus::Undetermined,
        evidence,
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }))
}
