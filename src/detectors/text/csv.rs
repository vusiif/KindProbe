use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

/// Probes whether the text represents a structured CSV or TSV tabular document.
pub fn probe_csv_or_tsv(text: &str) -> Option<DetectionReport> {
    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .take(5)
        .collect();

    if lines.len() < 2 {
        return None;
    }

    // 1. Test Tab-Separated Values (TSV)
    if let Some(col_count) = count_consistent_fields(&lines, '\t') {
        if col_count >= 2 {
            return Some(make_tabular_report("tsv", vec!["tsv", "tab"], "sig.text.tsv"));
        }
    }

    // 2. Test Comma-Separated Values (CSV)
    if let Some(col_count) = count_consistent_fields(&lines, ',') {
        if col_count >= 2 {
            return Some(make_tabular_report("csv", vec!["csv"], "sig.text.csv"));
        }
    }

    None
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

fn make_tabular_report(fid: &str, exts: Vec<&str>, ev: &str) -> DetectionReport {
    DetectionReport {
        schema_version: 1,
        format_id: Some(fid.to_string()),
        category: "text".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some(fid.to_string()),
        accepted_extensions: exts.into_iter().map(|s| s.to_string()).collect(),
        extension_status: ExtensionStatus::Undetermined,
        evidence: vec![EvidenceItem::new(ev)],
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }
}
