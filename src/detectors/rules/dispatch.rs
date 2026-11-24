use super::engine::{evaluate_single_rule, RuleMatch};
use super::static_index::{BUCKETS, FALLBACK, BY_FORMAT};
use crate::model::DetectionReport;

/// Evaluates signature rules using a compile-time static 256-bucket index.
/// Zero heap allocations, zero OnceLock atomic sync overhead.
pub fn evaluate_indexed_rules(header: &[u8]) -> RuleMatch {
    if header.is_empty() {
        return RuleMatch::None;
    }

    let b0 = header[0] as usize;
    let mut truncated_candidate: Option<String> = None;

    for &rule in BUCKETS[b0].iter() {
        if !rule.patterns.is_empty() {
            let p0 = &rule.patterns[0];
            if p0.offset == 0 && p0.mask.is_none() {
                if header.len() >= 4 && p0.bytes.len() >= 4 {
                    let h4 = u32::from_ne_bytes([header[0], header[1], header[2], header[3]]);
                    let p4 = u32::from_ne_bytes([p0.bytes[0], p0.bytes[1], p0.bytes[2], p0.bytes[3]]);
                    if h4 != p4 {
                        continue;
                    }
                } else if header.len() >= 2 && p0.bytes.len() >= 2 && header[1] != p0.bytes[1] {
                    continue;
                }
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

    for &rule in FALLBACK.iter() {
        if let Some(first_pat) = rule.patterns.first() {
            if first_pat.offset >= header.len() {
                break;
            }
            if first_pat.mask.is_none() && !first_pat.bytes.is_empty() && header[first_pat.offset] != first_pat.bytes[0] {
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

    if let Some(cand) = truncated_candidate {
        RuleMatch::Truncated(cand)
    } else {
        RuleMatch::None
    }
}

/// Fast path lookup and evaluation for a specific format_id against the static sorted format table.
pub fn probe_indexed_rule_by_format(header: &[u8], target_fid: &str) -> Option<DetectionReport> {
    if let Ok(idx) = BY_FORMAT.binary_search_by_key(&target_fid, |&(fid, _)| fid) {
        let mut start = idx;
        while start > 0 && BY_FORMAT[start - 1].0 == target_fid {
            start -= 1;
        }
        for item in &BY_FORMAT[start..] {
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
