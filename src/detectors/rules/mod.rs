pub mod engine;
pub mod generated;
pub mod registry;
pub mod types;

pub use engine::{evaluate_rules, RuleMatch};
pub use generated::GENERATED_RULES;
pub use registry::DECLARATIVE_RULES;
pub use types::{MagicPattern, SignatureRule, ValidationStatus};
