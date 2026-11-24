use std::sync::OnceLock;
use super::engine::{evaluate_single_rule, RuleMatch};
use super::generated::GENERATED_RULES;
use super::registry::DECLARATIVE_RULES;
use super::types::SignatureRule;
use crate::model::DetectionReport;

/// Two-tier rule index:
/// 1. 256-bucket array for O(1) first-byte dispatch (offset == 0 rules without general masks)
/// 2. Small fallback list for rules with offset > 0 or complex masks
/// 3. Alphabetically sorted format_id table for O(log N) binary search in fast path
pub struct RuleIndex {
    buckets: [Box<[&'static SignatureRule]>; 256],
    fallback: Box<[&'static SignatureRule]>,
    by_format: Box<[(&'static str, &'static SignatureRule)]>,
}

static RULE_INDEX: OnceLock<RuleIndex> = OnceLock::new();

pub fn get_rule_index() -> &'static RuleIndex {
    RULE_INDEX.get_or_init(build_rule_index)
}

fn build_rule_index() -> RuleIndex {
    let mut temp_buckets: [Vec<&'static SignatureRule>; 256] = std::array::from_fn(|_| Vec::new());
    let mut fallback_vec = Vec::new();
    let mut by_fmt_vec = Vec::new();

    let mut all_rules: Vec<&'static SignatureRule> = Vec::new();
    all_rules.extend(DECLARATIVE_RULES.iter());
    for group in GENERATED_RULES {
        all_rules.extend(group.iter());
    }

    for &rule in &all_rules {
        by_fmt_vec.push((rule.format_id, rule));
        if rule.patterns.is_empty() {
            fallback_vec.push(rule);
            continue;
        }

        let first_pat = &rule.patterns[0];
        if first_pat.offset == 0 {
            if let Some(mask) = first_pat.mask {
                if !mask.is_empty() && mask[0] == 0xFF && !first_pat.bytes.is_empty() {
                    let b = first_pat.bytes[0] as usize;
                    temp_buckets[b].push(rule);
                } else {
                    fallback_vec.push(rule);
                }
            } else if !first_pat.bytes.is_empty() {
                let b = first_pat.bytes[0] as usize;
                temp_buckets[b].push(rule);
            } else {
                fallback_vec.push(rule);
            }
        } else {
            fallback_vec.push(rule);
        }
    }

    fallback_vec.sort_by_key(|r| r.patterns.first().map(|p| p.offset).unwrap_or(0));
    by_fmt_vec.sort_unstable_by_key(|&(fid, _)| fid);

    RuleIndex {
        buckets: std::array::from_fn(|i| temp_buckets[i].clone().into_boxed_slice()),
        fallback: fallback_vec.into_boxed_slice(),
        by_format: by_fmt_vec.into_boxed_slice(),
    }
}

pub fn evaluate_indexed_rules(header: &[u8]) -> RuleMatch {
    if header.is_empty() {
        return RuleMatch::None;
    }

    let index = get_rule_index();
    let b0 = header[0] as usize;
    let mut truncated_candidate: Option<String> = None;

    for &rule in index.buckets[b0].iter() {
        if header.len() >= 2 && !rule.patterns.is_empty() {
            let p0 = &rule.patterns[0];
            if p0.offset == 0 && p0.mask.is_none() && p0.bytes.len() >= 2 && header[1] != p0.bytes[1] {
                continue;
            }
        }

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

    for &rule in index.fallback.iter() {
        if let Some(first_pat) = rule.patterns.first() {
            if first_pat.offset >= header.len() {
                break;
            }
        }
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

    if let Some(cand) = truncated_candidate {
        RuleMatch::Truncated(cand)
    } else {
        RuleMatch::None
    }
}

pub fn probe_indexed_rule_by_format(header: &[u8], target_fid: &str) -> Option<DetectionReport> {
    let index = get_rule_index();
    if let Ok(idx) = index.by_format.binary_search_by_key(&target_fid, |&(fid, _)| fid) {
        let mut start = idx;
        while start > 0 && index.by_format[start - 1].0 == target_fid {
            start -= 1;
        }
        for item in &index.by_format[start..] {
            if item.0 != target_fid {
                break;
            }
            if let RuleMatch::Confirmed(rep) = evaluate_single_rule(header, item.1) {
                return Some(*rep);
            }
        }
    }
    None
}
