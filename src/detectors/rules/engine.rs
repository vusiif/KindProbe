use super::types::{SignatureRule, ValidationStatus};
use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleMatchCore {
    Confirmed(DetectionCore),
    Truncated(&'static str),
    None,
}

pub enum RuleMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

#[inline]
pub fn evaluate_single_rule_core(header: &[u8], rule: &SignatureRule) -> RuleMatchCore {
    let mut pattern_truncated = false;

    for pat in rule.patterns {
        if header.len() < pat.offset {
            return if pattern_truncated {
                RuleMatchCore::Truncated(rule.format_id)
            } else {
                RuleMatchCore::None
            };
        }

        let available = &header[pat.offset..];
        if available.len() < pat.bytes.len() {
            if available.len() >= 2 || (pat.offset > 0 && !available.is_empty()) {
                let is_trunc = match pat.mask {
                    Some(m) => available.iter().zip(m).zip(pat.bytes).all(|((&b, &mk), &p)| (b & mk) == p),
                    None => pat.bytes.starts_with(available),
                };
                if is_trunc {
                    pattern_truncated = true;
                }
            }
            return if pattern_truncated {
                RuleMatchCore::Truncated(rule.format_id)
            } else {
                RuleMatchCore::None
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
                    RuleMatchCore::Truncated(rule.format_id)
                } else {
                    RuleMatchCore::None
                };
            }
        }
    }

    if header.len() < rule.min_header_len {
        return if header.len() >= 2 {
            RuleMatchCore::Truncated(rule.format_id)
        } else {
            RuleMatchCore::None
        };
    }

    if let Some(validator) = rule.validator {
        match validator(header) {
            ValidationStatus::Valid => (),
            ValidationStatus::Truncated => return RuleMatchCore::Truncated(rule.format_id),
            ValidationStatus::Invalid => return RuleMatchCore::None,
        }
    }

    RuleMatchCore::Confirmed(DetectionCore {
        format_id: Some(rule.format_id),
        category: rule.category,
        container_id: rule.container_id,
        identification: Identification::Format,
        verification: rule.verification,
        inspection: Inspection::Complete,
        recommended_extension: Some(rule.recommended_ext),
        accepted_extensions: rule.accepted_exts,
        primary_evidence: Some(rule.evidence_code),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    })
}

#[inline]
pub fn evaluate_single_rule(header: &[u8], rule: &SignatureRule) -> RuleMatch {
    match evaluate_single_rule_core(header, rule) {
        RuleMatchCore::Confirmed(core) => {
            RuleMatch::Confirmed(Box::new(core.to_report(Default::default(), ExtensionStatus::Undetermined)))
        }
        RuleMatchCore::Truncated(cand) => RuleMatch::Truncated(cand.to_string()),
        RuleMatchCore::None => RuleMatch::None,
    }
}

pub fn evaluate_rules_core(header: &[u8], rules: &[SignatureRule]) -> RuleMatchCore {
    let mut truncated_candidate: Option<&'static str> = None;
    for rule in rules {
        match evaluate_single_rule_core(header, rule) {
            RuleMatchCore::Confirmed(core) => return RuleMatchCore::Confirmed(core),
            RuleMatchCore::Truncated(cand) => {
                if truncated_candidate.is_none() {
                    truncated_candidate = Some(cand);
                }
            }
            RuleMatchCore::None => (),
        }
    }
    truncated_candidate.map(RuleMatchCore::Truncated).unwrap_or(RuleMatchCore::None)
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
