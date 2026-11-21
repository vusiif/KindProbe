pub mod engine;
pub mod types;

pub use engine::{evaluate_rules, RuleMatch};
pub use types::{MagicPattern, SignatureRule, ValidationStatus};
