pub mod config;
pub mod csv;
pub mod gateway;
pub mod json;
pub mod plain;
pub mod source_code;
pub mod xml_html;

use config::probe_config_core;
use csv::probe_csv_or_tsv_core;
use gateway::examine_text_gateway;
use json::probe_json_core;
use plain::make_plain_text_core;
use source_code::probe_source_code_core;
use xml_html::probe_xml_or_html_core;

use crate::model::{DetectionCore, DetectionReport, ExtensionStatus};

fn probe_utf16_bom_text_core(header: &[u8]) -> Option<DetectionCore> {
    if header.len() < 4 {
        return None;
    }
    let is_le = header.starts_with(b"\xFF\xFE");
    let is_be = header.starts_with(b"\xFE\xFF");
    if !is_le && !is_be {
        return None;
    }
    // Reject UTF-32 LE BOM (FF FE 00 00)
    if is_le && header[2] == 0 && header[3] == 0 {
        return None;
    }

    let payload = &header[2..header.len().min(4098)];
    let even_len = payload.len() & !1;
    if even_len == 0 {
        return None;
    }
    let units = payload[..even_len].as_chunks::<2>().0.iter().map(|c| {
        if is_le {
            u16::from_le_bytes([c[0], c[1]])
        } else {
            u16::from_be_bytes([c[0], c[1]])
        }
    });

    let mut decoded = String::with_capacity(even_len / 2);
    let mut control_count = 0usize;
    let mut char_count = 0usize;
    for res in char::decode_utf16(units) {
        let ch = res.ok()?;
        if ch == '\0' {
            return None;
        }
        char_count += 1;
        if (ch < ' ' && ch != '\t' && ch != '\n' && ch != '\r') || ch == '\x7F' {
            control_count += 1;
        }
        decoded.push(ch);
    }

    if char_count == 0 || control_count > char_count / 100 {
        return None;
    }

    probe_text_str_core(&decoded)
}

/// Probes whether the header bytes belong to structured, source code, or plain text formats, returning zero-allocation DetectionCore.
pub fn probe_text_core(header: &[u8]) -> Option<DetectionCore> {
    // 1. Text Gateway check: BOM stripping, UTF-8 validity, control char filtering
    let text = match examine_text_gateway(header) {
        Some(t) => t,
        None => return probe_utf16_bom_text_core(header),
    };
    probe_text_str_core(text)
}

fn probe_text_str_core(text: &str) -> Option<DetectionCore> {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return Some(make_plain_text_core());
    }

    let first = trimmed.as_bytes()[0];

    // 2. High-performance first-character dispatch
    match first {
        b'{' => {
            if let Some(core) = probe_json_core(text) {
                return Some(core);
            }
            if let Some(core) = probe_source_code_core(text) {
                return Some(core);
            }
        }
        b'[' => {
            if let Some(core) = probe_json_core(text) {
                return Some(core);
            }
            if let Some(core) = probe_config_core(text) {
                return Some(core);
            }
            if let Some(core) = probe_source_code_core(text) {
                return Some(core);
            }
        }
        b'<' => {
            if let Some(core) = probe_xml_or_html_core(text) {
                return Some(core);
            }
        }
        b'#' => {
            if let Some(core) = probe_source_code_core(text) {
                return Some(core);
            }
            if let Some(core) = probe_config_core(text) {
                return Some(core);
            }
        }
        b'-' => {
            if let Some(core) = probe_config_core(text) {
                return Some(core);
            }
            if let Some(core) = probe_source_code_core(text) {
                return Some(core);
            }
        }
        _ => {
            if let Some(core) = probe_source_code_core(text) {
                return Some(core);
            }
            if let Some(core) = probe_config_core(text) {
                return Some(core);
            }
            if let Some(core) = probe_csv_or_tsv_core(text) {
                return Some(core);
            }
        }
    }

    // Secondary fallback checks for non-letter prefixes
    if first == b'{' || first == b'[' || first == b'#' || first == b'-' {
        if let Some(core) = probe_csv_or_tsv_core(text) {
            return Some(core);
        }
    }

    // Fallback to confirmed Plain Text
    Some(make_plain_text_core())
}

/// Probes whether the header bytes belong to structured, source code, or plain text formats.
pub fn probe_text(header: &[u8]) -> Option<DetectionReport> {
    probe_text_core(header).map(|core| core.to_report(Default::default(), ExtensionStatus::Undetermined))
}
