use super::types::{SignatureRule, ValidationStatus};
use crate::model::{DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection};

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
                    Some(m) => available
                        .iter()
                        .zip(m)
                        .zip(pat.bytes)
                        .all(|((&b, &mk), &p)| (b & mk) == p),
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
                                pat.bytes[0],
                                pat.bytes[1],
                                pat.bytes[2],
                                pat.bytes[3],
                            ])
                    }
                    8 => {
                        let b = &available[..8];
                        u64::from_ne_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
                            == u64::from_ne_bytes([
                                pat.bytes[0],
                                pat.bytes[1],
                                pat.bytes[2],
                                pat.bytes[3],
                                pat.bytes[4],
                                pat.bytes[5],
                                pat.bytes[6],
                                pat.bytes[7],
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
        RuleMatchCore::Confirmed(core) => RuleMatch::Confirmed(Box::new(
            core.to_report(Default::default(), ExtensionStatus::Undetermined),
        )),
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
    truncated_candidate
        .map(RuleMatchCore::Truncated)
        .unwrap_or(RuleMatchCore::None)
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
    truncated_candidate
        .map(RuleMatch::Truncated)
        .unwrap_or(RuleMatch::None)
}

#[inline]
pub(crate) fn validate_min_bytes(header: &[u8], min_len: usize) -> ValidationStatus {
    if header.len() < min_len {
        ValidationStatus::Truncated
    } else {
        ValidationStatus::Valid
    }
}

pub(crate) fn validate_min_4(header: &[u8]) -> ValidationStatus {
    validate_min_bytes(header, 4)
}

pub(crate) fn validate_min_6(header: &[u8]) -> ValidationStatus {
    validate_min_bytes(header, 6)
}

pub(crate) fn validate_binary_not_ascii(
    header: &[u8],
    min_len: usize,
    ascii_check_from: usize,
) -> ValidationStatus {
    if header.len() < min_len {
        return ValidationStatus::Truncated;
    }
    let slice = &header[ascii_check_from..header.len().min(16)];
    if slice
        .iter()
        .all(|b| b.is_ascii_graphic() || matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
    {
        ValidationStatus::Invalid
    } else {
        ValidationStatus::Valid
    }
}

pub(crate) fn validate_binary_6_from_2(header: &[u8]) -> ValidationStatus {
    validate_binary_not_ascii(header, 6, 2)
}

pub(crate) fn validate_binary_6_from_3(header: &[u8]) -> ValidationStatus {
    validate_binary_not_ascii(header, 6, 3)
}

pub(crate) fn validate_xbm(header: &[u8]) -> ValidationStatus {
    if header.len() < 16 && !header.contains(&b'\n') {
        return ValidationStatus::Truncated;
    }
    if header
        .windows(6)
        .any(|w| w == b"_width" || w == b"_bits[" || w == b"_bits ")
        || header.windows(7).any(|w| w == b"_height")
    {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}

pub(crate) fn validate_ihex(header: &[u8]) -> ValidationStatus {
    if header.len() < 11 {
        return ValidationStatus::Truncated;
    }
    if header[1..11].iter().all(|b| b.is_ascii_hexdigit()) {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}

pub(crate) fn validate_smk(header: &[u8]) -> ValidationStatus {
    if header.len() < 4 {
        return ValidationStatus::Truncated;
    }
    if header[3] == b'2' || header[3] == b'4' {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}

pub(crate) fn validate_sp3(header: &[u8]) -> ValidationStatus {
    if header.len() < 7 {
        return ValidationStatus::Truncated;
    }
    if header[3..7].iter().all(|b| b.is_ascii_digit()) {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}

pub(crate) fn validate_eot(header: &[u8]) -> ValidationStatus {
    validate_binary_not_ascii(header, 16, 0)
}

pub(crate) fn validate_der(header: &[u8]) -> ValidationStatus {
    if header.len() < 4 {
        return ValidationStatus::Truncated;
    }
    let len = u16::from_be_bytes([header[2], header[3]]);
    if len > 0 {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}

pub(crate) fn validate_z_compress(header: &[u8]) -> ValidationStatus {
    if header.len() < 4 {
        return ValidationStatus::Truncated;
    }
    let flags = header[2];
    if (flags & 0x60) == 0 && (9..=16).contains(&(flags & 0x1F)) {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}

pub(crate) fn validate_sgi(header: &[u8]) -> ValidationStatus {
    if header.len() < 6 {
        return ValidationStatus::Truncated;
    }
    if (header[2] == 0 || header[2] == 1) && (header[3] == 1 || header[3] == 2) {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}

pub(crate) fn validate_ac3(header: &[u8]) -> ValidationStatus {
    if header.len() < 4 {
        return ValidationStatus::Truncated;
    }
    // E-AC-3 uses bitstream id (bsid = header[5] >> 3) in 11..=16; yield to the eac3 rule.
    if header.len() >= 6 && (11..=16).contains(&(header[5] >> 3)) {
        ValidationStatus::Invalid
    } else {
        ValidationStatus::Valid
    }
}

pub(crate) fn validate_eac3(header: &[u8]) -> ValidationStatus {
    if header.len() < 6 {
        return ValidationStatus::Invalid;
    }
    if (11..=16).contains(&(header[5] >> 3)) {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}
