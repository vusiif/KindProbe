pub mod engine;
pub mod generated;
pub mod registry;
pub mod types;

pub use engine::{evaluate_rules, evaluate_single_rule, RuleMatch};
pub use generated::GENERATED_RULES;
pub use registry::DECLARATIVE_RULES;
pub use types::{MagicPattern, SignatureRule, ValidationStatus};

/// Fast-path lookup and evaluation for a specific format_id against declarative rules.
pub fn probe_rule_by_format(
    header: &[u8],
    target_fid: &str,
) -> Option<crate::model::DetectionReport> {
    if let Some(rule) = DECLARATIVE_RULES.iter().find(|r| r.format_id == target_fid) {
        if let RuleMatch::Confirmed(rep) = evaluate_single_rule(header, rule) {
            return Some(*rep);
        }
    }
    for group in GENERATED_RULES {
        if let Some(rule) = group.iter().find(|r| r.format_id == target_fid) {
            if let RuleMatch::Confirmed(rep) = evaluate_single_rule(header, rule) {
                return Some(*rep);
            }
        }
    }
    None
}
