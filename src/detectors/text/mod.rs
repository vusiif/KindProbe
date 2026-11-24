pub mod config;
pub mod csv;
pub mod gateway;
pub mod json;
pub mod plain;
pub mod source_code;
pub mod xml_html;

use config::probe_config;
use csv::probe_csv_or_tsv;
use gateway::examine_text_gateway;
use json::probe_json;
use plain::make_plain_text_report;
use source_code::probe_source_code;
use xml_html::probe_xml_or_html;

use crate::model::DetectionReport;

/// Probes whether the header bytes belong to structured, source code, or plain text formats.
pub fn probe_text(header: &[u8]) -> Option<DetectionReport> {
    // 1. Text Gateway check: BOM stripping, UTF-8 validity, control char filtering
    let text = examine_text_gateway(header)?;
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return Some(make_plain_text_report());
    }

    let first = trimmed.as_bytes()[0];

    // 2. High-performance first-character dispatch
    match first {
        b'{' => {
            if let Some(rep) = probe_json(text) {
                return Some(rep);
            }
        }
        b'[' => {
            if let Some(rep) = probe_json(text) {
                return Some(rep);
            }
            if let Some(rep) = probe_config(text) {
                return Some(rep);
            }
        }
        b'<' => {
            if let Some(rep) = probe_xml_or_html(text) {
                return Some(rep);
            }
        }
        b'#' => {
            if let Some(rep) = probe_source_code(text) {
                return Some(rep);
            }
            if let Some(rep) = probe_config(text) {
                return Some(rep);
            }
        }
        b'-' => {
            if let Some(rep) = probe_config(text) {
                return Some(rep);
            }
            if let Some(rep) = probe_source_code(text) {
                return Some(rep);
            }
        }
        _ => {
            if let Some(rep) = probe_source_code(text) {
                return Some(rep);
            }
            if let Some(rep) = probe_config(text) {
                return Some(rep);
            }
            if let Some(rep) = probe_csv_or_tsv(text) {
                return Some(rep);
            }
        }
    }

    // Secondary fallback checks for non-letter prefixes
    if first == b'{' || first == b'[' || first == b'#' || first == b'-' {
        if let Some(rep) = probe_csv_or_tsv(text) {
            return Some(rep);
        }
    }

    // Fallback to confirmed Plain Text
    Some(make_plain_text_report())
}
