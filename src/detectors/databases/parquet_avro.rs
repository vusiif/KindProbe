use crate::model::{
    DetectionCore, DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
    Verification,
};

pub enum ColumnarMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn make_parquet_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("parquet"),
        category: "database",
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("parquet"),
        accepted_extensions: &["parquet"],
        primary_evidence: Some("sig.parquet.magic"),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}

pub fn make_avro_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("avro"),
        category: "database",
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some("avro"),
        accepted_extensions: &["avro"],
        primary_evidence: Some("sig.avro.magic"),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}

pub fn probe_parquet_avro_core(header: &[u8]) -> Option<DetectionCore> {
    if header.starts_with(b"PAR1") {
        return Some(make_parquet_core());
    }
    if header.starts_with(b"Obj\x01") {
        return Some(make_avro_core());
    }
    None
}

/// Probes Apache Parquet and Apache Avro columnar serialization formats.
pub fn probe_parquet_avro(header: &[u8]) -> ColumnarMatch {
    if header.len() < 4 {
        if !header.is_empty() && (header.starts_with(b"PAR") || header.starts_with(b"Obj")) {
            return ColumnarMatch::Truncated(if header.starts_with(b"PAR") {
                "parquet".to_string()
            } else {
                "avro".to_string()
            });
        }
        return ColumnarMatch::None;
    }

    // 1. Apache Parquet ("PAR1")
    if header.starts_with(b"PAR1") {
        return ColumnarMatch::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some("parquet".to_string()),
            category: "database".to_string(),
            container_id: None,
            identification: Identification::Format,
            verification: Verification::Structure,
            inspection: Inspection::Complete,
            recommended_extension: Some("parquet".to_string()),
            accepted_extensions: vec!["parquet".to_string()],
            extension_status: ExtensionStatus::Undetermined,
            evidence: vec![EvidenceItem::new("sig.parquet.magic")],
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    // 2. Apache Avro ("Obj\x01")
    if header.starts_with(b"Obj\x01") {
        return ColumnarMatch::Confirmed(Box::new(DetectionReport {
            schema_version: 1,
            format_id: Some("avro".to_string()),
            category: "database".to_string(),
            container_id: None,
            identification: Identification::Format,
            verification: Verification::Structure,
            inspection: Inspection::Complete,
            recommended_extension: Some("avro".to_string()),
            accepted_extensions: vec!["avro".to_string()],
            extension_status: ExtensionStatus::Undetermined,
            evidence: vec![EvidenceItem::new("sig.avro.magic")],
            warnings: Vec::new(),
            stats: Default::default(),
            candidates: None,
        }));
    }

    ColumnarMatch::None
}
