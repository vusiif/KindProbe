use super::engine::{evaluate_single_rule_core, RuleMatch, RuleMatchCore};
use super::static_index::{BUCKETS, BY_FORMAT, OFFSET_GROUPS};
use super::types::SignatureRule;
use crate::model::{DetectionCore, DetectionReport, ExtensionStatus};

const DENSE_BUCKET_MIN: usize = 8;
const MASK_BITS: usize = u64::BITS as usize;

const fn required_byte(rule: &SignatureRule, offset: usize, byte_index: usize) -> Option<u8> {
    match rule.patterns.first() {
        Some(pattern)
            if pattern.offset == offset
                && pattern.mask.is_none()
                && pattern.bytes.len() > byte_index =>
        {
            Some(pattern.bytes[byte_index])
        }
        _ => None,
    }
}

// The intersection selects rules matching both halves of a byte. Bit order is rule order.
struct ByteCandidates {
    low: [u64; 16],
    high: [u64; 16],
}

impl ByteCandidates {
    const fn new(rules: &[&SignatureRule], offset: usize, byte_index: usize) -> Self {
        let mut candidates = Self {
            low: [0; 16],
            high: [0; 16],
        };
        let mut index = 0;
        let mut unfiltered = 0;
        while index < rules.len() {
            let bit = 1 << index;
            match required_byte(rules[index], offset, byte_index) {
                Some(byte) => {
                    let byte = byte as usize;
                    candidates.low[byte & 15] |= bit;
                    candidates.high[byte >> 4] |= bit;
                }
                _ => unfiltered |= bit,
            }
            index += 1;
        }
        let mut nibble = 0;
        while nibble < 16 {
            candidates.low[nibble] |= unfiltered;
            candidates.high[nibble] |= unfiltered;
            nibble += 1;
        }
        candidates
    }

    #[inline]
    fn select(&self, byte: u8) -> u64 {
        self.low[(byte & 15) as usize] & self.high[(byte >> 4) as usize]
    }
}

const fn is_dense(len: usize) -> bool {
    len >= DENSE_BUCKET_MIN && len <= MASK_BITS
}

const fn dense_bucket_count() -> usize {
    let mut index = 0;
    let mut count = 0;
    while index < BUCKETS.len() {
        if is_dense(BUCKETS[index].len()) {
            count += 1;
        }
        index += 1;
    }
    count
}

const fn collision_count(rules: &[&SignatureRule], byte_index: usize) -> usize {
    let mut counts = [0; 256];
    let mut unfiltered = 0;
    let mut collisions = 0;
    let mut index = 0;
    while index < rules.len() {
        match required_byte(rules[index], 0, byte_index) {
            Some(byte) => {
                collisions += counts[byte as usize] + unfiltered;
                counts[byte as usize] += 1;
            }
            None => {
                // An unconstrained rule can coexist with every earlier candidate.
                collisions += index;
                unfiltered += 1;
            }
        }
        index += 1;
    }
    collisions
}

const fn best_filter_byte(rules: &[&SignatureRule]) -> usize {
    let mut best = 1;
    let mut best_score = collision_count(rules, best);
    let mut index = 2;
    // Prefer the earlier byte on ties so short input can still use the filter.
    while index < 4 {
        let score = collision_count(rules, index);
        if score < best_score {
            best = index;
            best_score = score;
        }
        index += 1;
    }
    best
}

struct DenseBucket {
    byte_index: u8,
    candidates: ByteCandidates,
}

struct DenseBuckets {
    slots: [u8; 256],
    filters: [DenseBucket; dense_bucket_count()],
}

static DENSE_BUCKETS: DenseBuckets = {
    let mut buckets = DenseBuckets {
        slots: [0; 256],
        filters: [const {
            DenseBucket {
                byte_index: 1,
                candidates: ByteCandidates::new(&[], 0, 1),
            }
        }; dense_bucket_count()],
    };
    let mut byte = 0;
    let mut slot = 0;
    while byte < BUCKETS.len() {
        if is_dense(BUCKETS[byte].len()) {
            buckets.slots[byte] = slot as u8;
            let byte_index = best_filter_byte(BUCKETS[byte]);
            buckets.filters[slot] = DenseBucket {
                byte_index: byte_index as u8,
                candidates: ByteCandidates::new(BUCKETS[byte], 0, byte_index),
            };
            slot += 1;
        }
        byte += 1;
    }
    buckets
};

static OFFSET_CANDIDATES: [ByteCandidates; OFFSET_GROUPS.len()] = {
    let mut candidates = [const { ByteCandidates::new(&[], 0, 0) }; OFFSET_GROUPS.len()];
    let mut index = 0;
    while index < OFFSET_GROUPS.len() {
        let group = &OFFSET_GROUPS[index];
        if group.rules.len() <= MASK_BITS {
            candidates[index] = ByteCandidates::new(group.rules, group.offset, 0);
        }
        index += 1;
    }
    candidates
};

#[inline]
fn evaluate_candidates(
    header: &[u8],
    rules: &[&SignatureRule],
    mut candidates: u64,
    truncated: &mut Option<&'static str>,
) -> Option<DetectionCore> {
    while candidates != 0 {
        let index = candidates.trailing_zeros() as usize;
        candidates &= candidates - 1;
        match evaluate_single_rule_core(header, rules[index]) {
            RuleMatchCore::Confirmed(core) => return Some(core),
            RuleMatchCore::Truncated(candidate) => {
                truncated.get_or_insert(candidate);
            }
            RuleMatchCore::None => (),
        }
    }
    None
}

/// Evaluates signature rules using a compile-time static 256-bucket index, returning a zero-allocation DetectionCore.
pub fn evaluate_indexed_rules_core(header: &[u8]) -> RuleMatchCore {
    if header.is_empty() {
        return RuleMatchCore::None;
    }

    let b0 = header[0] as usize;
    let mut truncated_candidate: Option<&'static str> = None;
    let rules = BUCKETS[b0];

    let matched = if is_dense(rules.len()) {
        let slot = DENSE_BUCKETS.slots[b0] as usize;
        let filter = &DENSE_BUCKETS.filters[slot];
        if let Some(&byte) = header.get(filter.byte_index as usize) {
            let candidates = filter.candidates.select(byte);
            evaluate_candidates(header, rules, candidates, &mut truncated_candidate)
        } else {
            evaluate_bucket(header, rules, &mut truncated_candidate)
        }
    } else {
        evaluate_bucket(header, rules, &mut truncated_candidate)
    };
    if let Some(core) = matched {
        return RuleMatchCore::Confirmed(core);
    }

    for (group, index) in OFFSET_GROUPS.iter().zip(OFFSET_CANDIDATES.iter()) {
        let Some(&byte) = header.get(group.offset) else {
            break;
        };
        if group.rules.len() <= MASK_BITS {
            let candidates = index.select(byte);
            if let Some(core) =
                evaluate_candidates(header, group.rules, candidates, &mut truncated_candidate)
            {
                return RuleMatchCore::Confirmed(core);
            }
        } else if let Some(core) = evaluate_bucket(header, group.rules, &mut truncated_candidate) {
            return RuleMatchCore::Confirmed(core);
        }
    }

    truncated_candidate.map_or(RuleMatchCore::None, RuleMatchCore::Truncated)
}

fn evaluate_bucket(
    header: &[u8],
    rules: &[&SignatureRule],
    truncated: &mut Option<&'static str>,
) -> Option<DetectionCore> {
    for &rule in rules {
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
            RuleMatchCore::Confirmed(core) => return Some(core),
            RuleMatchCore::Truncated(cand) => {
                truncated.get_or_insert(cand);
            }
            RuleMatchCore::None => (),
        }
    }

    None
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
