use crate::model::Verification;

/// Represents a slice pattern at a specific offset in the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MagicPattern {
    pub offset: usize,
    pub bytes: &'static [u8],
    pub mask: Option<&'static [u8]>,
}

impl MagicPattern {
    pub const fn new(offset: usize, bytes: &'static [u8]) -> Self {
        Self {
            offset,
            bytes,
            mask: None,
        }
    }

    pub const fn with_mask(offset: usize, bytes: &'static [u8], mask: &'static [u8]) -> Self {
        Self {
            offset,
            bytes,
            mask: Some(mask),
        }
    }
}

/// Three-state validation result for custom header verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationStatus {
    /// Fully valid header structure.
    Valid,
    /// Definitely does not match the format specification.
    Invalid,
    /// Prefix or magic matches, but header buffer is too short to complete verification.
    Truncated,
}

/// Declarative format signature rule.
#[derive(Debug, Clone, Copy)]
pub struct SignatureRule {
    pub format_id: &'static str,
    pub category: &'static str,
    pub container_id: Option<&'static str>,
    pub recommended_ext: &'static str,
    pub accepted_exts: &'static [&'static str],
    pub patterns: &'static [MagicPattern],
    pub min_header_len: usize,
    pub priority: u8,
    pub evidence_code: &'static str,
    pub verification: Verification,
    pub validator: Option<fn(header: &[u8]) -> ValidationStatus>,
}
