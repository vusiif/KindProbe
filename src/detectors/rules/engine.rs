use super::types::{SignatureRule, ValidationStatus};
use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
};

pub enum RuleMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn evaluate_rules(header: &[u8], rules: &[SignatureRule]) -> RuleMatch {
    let mut truncated_candidate: Option<String> = None;

    for rule in rules {
        // 1. Check magic patterns
        let mut patterns_matched = true;
        let mut pattern_truncated = false;

        for pat in rule.patterns {
            if header.len() < pat.offset {
                patterns_matched = false;
                break;
            }

            let available = &header[pat.offset..];
            if available.len() < pat.bytes.len() {
                if !available.is_empty() && pat.bytes.starts_with(available) {
                    pattern_truncated = true;
                }
                patterns_matched = false;
                break;
            } else if &available[..pat.bytes.len()] != pat.bytes {
                patterns_matched = false;
                break;
            }
        }

        if pattern_truncated && truncated_candidate.is_none() {
            truncated_candidate = Some(rule.format_id.to_string());
        }

        if !patterns_matched {
            continue;
        }

        // 2. Check minimum header length requirement
        if header.len() < rule.min_header_len {
            if truncated_candidate.is_none() {
                truncated_candidate = Some(rule.format_id.to_string());
            }
            continue;
        }

        // 3. Check optional custom validator
        if let Some(validator) = rule.validator {
            match validator(header) {
                ValidationStatus::Valid => (),
                ValidationStatus::Truncated => {
                    if truncated_candidate.is_none() {
                        truncated_candidate = Some(rule.format_id.to_string());
                    }
                    continue;
                }
                ValidationStatus::Invalid => continue,
            }
        }

        // 4. Fully matched rule
        return RuleMatch::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some(rule.format_id.to_string()),
            category: rule.category.to_string(),
            container_id: rule.container_id.map(|s| s.to_string()),
            identification: Identification::Format,
            verification: rule.verification,
            inspection: Inspection::Complete,
            recommended_extension: Some(rule.recommended_ext.to_string()),
            accepted_extensions: rule.accepted_exts.iter().map(|s| s.to_string()).collect(),
            extension_status: ExtensionStatus::Undetermined,
            evidence: vec![EvidenceItem::new(rule.evidence_code)],
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    if let Some(cand) = truncated_candidate {
        RuleMatch::Truncated(cand)
    } else {
        RuleMatch::None
    }
}
