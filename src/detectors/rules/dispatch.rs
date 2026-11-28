use super::engine::{evaluate_single_rule_core, RuleMatch, RuleMatchCore};
use super::static_index::{BUCKETS, BY_FORMAT, OFFSET_GROUPS};
use crate::model::{DetectionCore, DetectionReport, ExtensionStatus};

/// Evaluates signature rules using a compile-time static 256-bucket index, returning a zero-allocation DetectionCore.
pub fn evaluate_indexed_rules_core(header: &[u8]) -> RuleMatchCore {
    if header.is_empty() {
        return RuleMatchCore::None;
    }

    let b0 = header[0] as usize;
    let mut truncated_candidate: Option<&'static str> = None;

    for &rule in BUCKETS[b0].iter() {
        if !rule.patterns.is_empty() {
            let p0 = &rule.patterns[0];
            if p0.offset == 0 && p0.mask.is_none() {
                if header.len() >= 4 && p0.bytes.len() >= 4 {
                    let h4 = u32::from_ne_bytes([header[0], header[1], header[2], header[3]]);
                    let p4 =
                        u32::from_ne_bytes([p0.bytes[0], p0.bytes[1], p0.bytes[2], p0.bytes[3]]);
                    if h4 != p4 {
                        continue;
                    }
                } else if header.len() >= 2 && p0.bytes.len() >= 2 && header[1] != p0.bytes[1] {
                    continue;
                }
            }
        }

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

    for group in OFFSET_GROUPS.iter() {
        if group.offset >= header.len() {
            break;
        }

        let b = header[group.offset];
        let remaining = header.len() - group.offset;

        for &rule in group.rules.iter() {
            if let Some(first_pat) = rule.patterns.first() {
                if first_pat.mask.is_none() {
                    if !first_pat.bytes.is_empty() && b != first_pat.bytes[0] {
                        continue;
                    }
                    if remaining >= 4 && first_pat.bytes.len() >= 4 {
                        let h4 = u32::from_ne_bytes([
                            header[group.offset],
                            header[group.offset + 1],
                            header[group.offset + 2],
                            header[group.offset + 3],
                        ]);
                        let p4 = u32::from_ne_bytes([
                            first_pat.bytes[0],
                            first_pat.bytes[1],
                            first_pat.bytes[2],
                            first_pat.bytes[3],
                        ]);
                        if h4 != p4 {
                            continue;
                        }
                    } else if remaining >= 2
                        && first_pat.bytes.len() >= 2
                        && header[group.offset + 1] != first_pat.bytes[1]
                    {
                        continue;
                    }
                }
            }

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
    }

    if let Some(cand) = truncated_candidate {
        RuleMatchCore::Truncated(cand)
    } else {
        RuleMatchCore::None
    }
}

/// Evaluates signature rules using a compile-time static 256-bucket index.
pub fn evaluate_indexed_rules(header: &[u8]) -> RuleMatch {
    match evaluate_indexed_rules_core(header) {
        RuleMatchCore::Confirmed(core) => RuleMatch::Confirmed(Box::new(
            core.to_report(Default::default(), ExtensionStatus::Undetermined),
        )),
        RuleMatchCore::Truncated(cand) => RuleMatch::Truncated(cand.to_string()),
        RuleMatchCore::None => RuleMatch::None,
    }
}

/// Fast path lookup and evaluation for a specific format_id returning a zero-allocation DetectionCore.
pub fn probe_indexed_rule_by_format_core(header: &[u8], target_fid: &str) -> Option<DetectionCore> {
    if let Ok(idx) = BY_FORMAT.binary_search_by_key(&target_fid, |&(fid, _)| fid) {
        let mut start = idx;
        while start > 0 && BY_FORMAT[start - 1].0 == target_fid {
            start -= 1;
        }
        for item in &BY_FORMAT[start..] {
            if item.0 != target_fid {
                break;
            }
            if let RuleMatchCore::Confirmed(core) = evaluate_single_rule_core(header, item.1) {
                return Some(core);
            }
        }
    }
    None
}

/// Fast path lookup and evaluation for a specific format_id against the static sorted format table.
pub fn probe_indexed_rule_by_format(header: &[u8], target_fid: &str) -> Option<DetectionReport> {
    probe_indexed_rule_by_format_core(header, target_fid)
        .map(|core| core.to_report(Default::default(), ExtensionStatus::Undetermined))
}
