use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Identification {
    Format,
    ContainerOnly,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verification {
    Signature,
    Structure,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Inspection {
    Complete,
    Limited,
    Ambiguous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionStatus {
    Match,
    Mismatch,
    Missing,
    Unrecognized,
    Undetermined,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceItem {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl EvidenceItem {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            description: None,
        }
    }

    pub fn with_desc(code: impl Into<String>, desc: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            description: Some(desc.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WarningItem {
    pub code: String,
    pub message: String,
}

impl WarningItem {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DetectionStats {
    pub source_bytes_read: usize,
    pub seek_count: usize,
    pub entries_inspected: usize,
    pub decompressed_bytes: usize,
    pub duration_micros: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DetectionReport {
    pub schema_version: u32,
    pub format_id: Option<String>,
    pub category: String,
    pub container_id: Option<String>,
    pub identification: Identification,
    pub verification: Verification,
    pub inspection: Inspection,
    pub recommended_extension: Option<String>,
    pub accepted_extensions: Vec<String>,
    pub extension_status: ExtensionStatus,
    pub evidence: Vec<EvidenceItem>,
    pub warnings: Vec<WarningItem>,
    pub stats: DetectionStats,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates: Option<Vec<String>>,
}

impl DetectionReport {
    /// Validates the legal state invariants according to specification §6.2.
    pub fn validate_invariants(&self) -> Result<(), &'static str> {
        if self.schema_version != 1 {
            return Err("schema_version must be 1");
        }

        // Rule: identification == format <=> format_id is Some
        if self.identification == Identification::Format {
            if self.format_id.is_none() {
                return Err("identification=format requires format_id to be non-null");
            }
        } else if self.format_id.is_some() {
            return Err("identification != format requires format_id to be null");
        }

        // Rule: identification != format => recommended_extension must be None
        if self.identification != Identification::Format && self.recommended_extension.is_some() {
            return Err("identification != format requires recommended_extension to be null");
        }

        // Rule: FORBIDDEN combination: format + ambiguous
        if self.identification == Identification::Format && self.inspection == Inspection::Ambiguous {
            return Err("FORBIDDEN: identification=format cannot be ambiguous");
        }

        // Rule: verification=none cannot coexist with confirmed format or container
        if self.verification == Verification::None {
            if self.identification == Identification::Format || self.container_id.is_some() {
                return Err("verification=none cannot coexist with confirmed format or container");
            }
        }

        Ok(())
    }

    /// Computes the CLI exit code according to specification §8.
    pub fn cli_exit_code(&self) -> i32 {
        if self.identification == Identification::Format && self.inspection == Inspection::Complete {
            0
        } else {
            3
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    pub schema_version: u32,
    pub error: ErrorDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
}

#[derive(Debug)]
pub enum DetectError {
    Io(std::io::Error),
    NotRegularFile(String),
    InvalidParameter(String),
}

impl fmt::Display for DetectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DetectError::Io(e) => write!(f, "I/O error: {}", e),
            DetectError::NotRegularFile(path) => write!(f, "Not a regular file: {}", path),
            DetectError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
        }
    }
}

impl std::error::Error for DetectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DetectError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for DetectError {
    fn from(e: std::io::Error) -> Self {
        DetectError::Io(e)
    }
}
