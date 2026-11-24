use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

/// Makes a zero-allocation DetectionCore for plain text files.
pub const fn make_plain_text_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("txt"),
        category: "text",
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("txt"),
        accepted_extensions: &["txt", "text", "log"],
        primary_evidence: Some("sig.text.plain"),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}

/// Makes a detection report for plain text files.
pub fn make_plain_text_report() -> DetectionReport {
    make_plain_text_core().to_report(Default::default(), ExtensionStatus::Undetermined)
}
