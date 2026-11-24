pub mod dispatch;
pub mod engine;
pub mod generated;
pub mod registry;
pub mod static_index;
pub mod types;

pub use dispatch::{
    evaluate_indexed_rules, evaluate_indexed_rules_core, probe_indexed_rule_by_format,
    probe_indexed_rule_by_format_core,
};
pub use engine::{
    evaluate_rules, evaluate_rules_core, evaluate_single_rule, evaluate_single_rule_core,
    RuleMatch, RuleMatchCore,
};
pub use generated::GENERATED_RULES;
pub use registry::DECLARATIVE_RULES;
pub use types::{MagicPattern, SignatureRule, ValidationStatus};

/// Fast-path lookup and evaluation for a specific format_id against declarative rules returning a zero-allocation DetectionCore.
pub fn probe_rule_by_format_core(
    header: &[u8],
    target_fid: &str,
) -> Option<crate::model::DetectionCore> {
    probe_indexed_rule_by_format_core(header, target_fid)
}

/// Fast-path lookup and evaluation for a specific format_id against declarative rules.
pub fn probe_rule_by_format(
    header: &[u8],
    target_fid: &str,
) -> Option<crate::model::DetectionReport> {
    probe_indexed_rule_by_format(header, target_fid)
}
