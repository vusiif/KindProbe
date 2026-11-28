use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

/// Probes whether the text represents a structured CSV or TSV tabular document, returning zero-allocation DetectionCore.
pub fn probe_csv_or_tsv_core(text: &str) -> Option<DetectionCore> {
    let mut lines_buf = [""; 5];
    let mut count = 0;
    for l in text.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
        lines_buf[count] = l;
        count += 1;
        if count == 5 {
            break;
        }
    }

    if count < 2 {
        return None;
    }
    let lines = &lines_buf[..count];

    // 1. Test Tab-Separated Values (TSV)
    if let Some(col_count) = count_consistent_fields(lines, '\t') {
        if col_count >= 2 {
            return Some(make_tabular_core("tsv", &["tsv", "tab"], "sig.text.tsv"));
        }
    }

    // 2. Test Comma-Separated Values (CSV)
    if let Some(col_count) = count_consistent_fields(lines, ',') {
        if col_count >= 2 {
            return Some(make_tabular_core("csv", &["csv"], "sig.text.csv"));
        }
    }

    None
}

/// Probes whether the text represents a structured CSV or TSV tabular document.
pub fn probe_csv_or_tsv(text: &str) -> Option<DetectionReport> {
    probe_csv_or_tsv_core(text)
        .map(|core| core.to_report(Default::default(), ExtensionStatus::Undetermined))
}

fn count_consistent_fields(lines: &[&str], delimiter: char) -> Option<usize> {
    let first_count = lines[0].split(delimiter).count();
    if first_count < 2 {
        return None;
    }

    for line in &lines[1..] {
        if line.split(delimiter).count() != first_count {
            return None;
        }
    }

    Some(first_count)
}

fn make_tabular_core(
    fid: &'static str,
    exts: &'static [&'static str],
    ev: &'static str,
) -> DetectionCore {
    DetectionCore {
        format_id: Some(fid),
        category: "text",
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some(fid),
        accepted_extensions: exts,
        primary_evidence: Some(ev),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}
