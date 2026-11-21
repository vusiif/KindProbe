pub mod config;
pub mod csv;
pub mod gateway;
pub mod json;
pub mod plain;
pub mod xml_html;

use config::probe_config;
use csv::probe_csv_or_tsv;
use gateway::examine_text_gateway;
use json::probe_json;
use plain::make_plain_text_report;
use xml_html::probe_xml_or_html;

use crate::model::DetectionReport;

/// Probes whether the header bytes belong to structured or plain text formats.
pub fn probe_text(header: &[u8]) -> Option<DetectionReport> {
    // 1. Text Gateway check: BOM stripping, UTF-8 validity, control char filtering
    let text = examine_text_gateway(header)?;

    // 2. High-confidence structural formats
    if let Some(rep) = probe_json(text) {
        return Some(rep);
    }

    if let Some(rep) = probe_xml_or_html(text) {
        return Some(rep);
    }

    if let Some(rep) = probe_config(text) {
        return Some(rep);
    }

    if let Some(rep) = probe_csv_or_tsv(text) {
        return Some(rep);
    }

    // 3. Fallback to confirmed Plain Text
    Some(make_plain_text_report())
}
