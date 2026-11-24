pub mod dispatch;
pub mod engine;
pub mod generated;
pub mod registry;
pub mod static_index;
pub mod types;

pub use dispatch::{evaluate_indexed_rules, probe_indexed_rule_by_format};
pub use engine::{evaluate_rules, evaluate_single_rule, RuleMatch};
pub use generated::GENERATED_RULES;
pub use registry::DECLARATIVE_RULES;
pub use types::{MagicPattern, SignatureRule, ValidationStatus};

/// Fast-path lookup and evaluation for a specific format_id against declarative rules.
pub fn probe_rule_by_format(
    header: &[u8],
    target_fid: &str,
) -> Option<crate::model::DetectionReport> {
    probe_indexed_rule_by_format(header, target_fid)
}
