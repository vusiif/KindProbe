use super::types::{SignatureRule, ValidationStatus};
use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
};

pub enum RuleMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

#[inline]
pub fn evaluate_single_rule(header: &[u8], rule: &SignatureRule) -> RuleMatch {
    let mut pattern_truncated = false;

    for pat in rule.patterns {
        if header.len() < pat.offset {
            return if pattern_truncated {
                RuleMatch::Truncated(rule.format_id.to_string())
            } else {
                RuleMatch::None
            };
        }

        let available = &header[pat.offset..];
        if available.len() < pat.bytes.len() {
            if !available.is_empty() {
                let is_trunc = match pat.mask {
                    Some(m) => available.iter().zip(m).zip(pat.bytes).all(|((&b, &mk), &p)| (b & mk) == p),
                    None => pat.bytes.starts_with(available),
                };
                if is_trunc {
                    pattern_truncated = true;
                }
            }
            return if pattern_truncated {
                RuleMatch::Truncated(rule.format_id.to_string())
            } else {
                RuleMatch::None
            };
        } else {
            let is_match = match pat.mask {
                Some(m) => available[..pat.bytes.len()]
                    .iter()
                    .zip(m)
                    .zip(pat.bytes)
                    .all(|((&b, &mk), &p)| (b & mk) == p),
                None => match pat.bytes.len() {
                    2 => {
                        let b = &available[..2];
                        b[0] == pat.bytes[0] && b[1] == pat.bytes[1]
                    }
                    4 => {
                        let b = &available[..4];
                        u32::from_ne_bytes([b[0], b[1], b[2], b[3]])
                            == u32::from_ne_bytes([
                                pat.bytes[0], pat.bytes[1], pat.bytes[2], pat.bytes[3],
                            ])
                    }
                    8 => {
                        let b = &available[..8];
                        u64::from_ne_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
                            == u64::from_ne_bytes([
                                pat.bytes[0], pat.bytes[1], pat.bytes[2], pat.bytes[3],
                                pat.bytes[4], pat.bytes[5], pat.bytes[6], pat.bytes[7],
                            ])
                    }
                    _ => &available[..pat.bytes.len()] == pat.bytes,
                },
            };
            if !is_match {
                return if pattern_truncated {
                    RuleMatch::Truncated(rule.format_id.to_string())
                } else {
                    RuleMatch::None
                };
            }
        }
    }

    if header.len() < rule.min_header_len {
        return RuleMatch::Truncated(rule.format_id.to_string());
    }

    if let Some(validator) = rule.validator {
        match validator(header) {
            ValidationStatus::Valid => (),
            ValidationStatus::Truncated => return RuleMatch::Truncated(rule.format_id.to_string()),
            ValidationStatus::Invalid => return RuleMatch::None,
        }
    }

    RuleMatch::Confirmed(Box::new(DetectionReport {
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
    }))
}

pub fn evaluate_rules(header: &[u8], rules: &[SignatureRule]) -> RuleMatch {
    let mut truncated_candidate: Option<String> = None;
    for rule in rules {
        match evaluate_single_rule(header, rule) {
            RuleMatch::Confirmed(rep) => return RuleMatch::Confirmed(rep),
            RuleMatch::Truncated(cand) => {
                if truncated_candidate.is_none() {
                    truncated_candidate = Some(cand);
                }
            }
            RuleMatch::None => (),
        }
    }
    truncated_candidate.map(RuleMatch::Truncated).unwrap_or(RuleMatch::None)
}
