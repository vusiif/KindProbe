mod devops_and_idl;
mod infra_and_langs;
mod pre_token;
mod specialized_and_styles;
mod token_scan;

use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

fn clamp_window(text: &str) -> &str {
    if text.len() > 4096 {
        let mut end = 4096;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        &text[..end]
    } else {
        text
    }
}

/// High-performance source code and script probe returning zero-allocation DetectionCore.
pub fn probe_source_code_core(text: &str) -> Option<DetectionCore> {
    let window = clamp_window(text);
    let w_bytes = window.as_bytes();

    pre_token::probe_shebang_and_pretoken(text, window, w_bytes)
        .or_else(|| token_scan::probe_token_scan(window))
        .or_else(|| devops_and_idl::probe_devops_and_idl(window, w_bytes))
        .or_else(|| infra_and_langs::probe_infra_and_langs(window, w_bytes))
        .or_else(|| specialized_and_styles::probe_specialized_and_styles(window, w_bytes))
}

/// Probes source code returning an owned DetectionReport.
pub fn probe_source_code(text: &str) -> Option<DetectionReport> {
    probe_source_code_core(text)
        .map(|core| core.to_report(Default::default(), ExtensionStatus::Undetermined))
}

pub(super) fn make_source_core(
    fid: &'static str,
    exts: &'static [&'static str],
    evidence: &'static str,
    ver: Verification,
) -> DetectionCore {
    DetectionCore {
        format_id: Some(fid),
        category: "text",
        container_id: None,
        identification: Identification::Format,
        verification: ver,
        inspection: Inspection::Complete,
        recommended_extension: Some(exts[0]),
        accepted_extensions: exts,
        primary_evidence: Some(evidence),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}
