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

/// Probes whether the header bytes belong to structured, source code, or plain text formats, returning zero-allocation DetectionCore.
pub fn probe_text_core(header: &[u8]) -> Option<DetectionCore> {
    // 1. Text Gateway check: BOM stripping, UTF-8 validity, control char filtering
    let text = examine_text_gateway(header)?;
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
