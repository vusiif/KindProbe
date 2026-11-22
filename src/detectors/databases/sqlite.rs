use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};

pub const SQLITE3_MAGIC: &[u8] = b"SQLite format 3\0";

pub enum SqliteMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes SQLite 3 database header and page size configuration.
pub fn probe_sqlite(header: &[u8]) -> SqliteMatch {
    if header.len() < 16 {
        if !header.is_empty() && SQLITE3_MAGIC.starts_with(header) {
            return SqliteMatch::Truncated("sqlite3".to_string());
        }
        return SqliteMatch::None;
    }

    if !header.starts_with(SQLITE3_MAGIC) {
        return SqliteMatch::None;
    }

    let mut evidence = vec![EvidenceItem::new("sig.sqlite.header")];

    // Read page size (offset 16..18) and write/read versions (offsets 18, 19)
    let verification = if header.len() >= 20 {
        let raw_page_size = u16::from_be_bytes([header[16], header[17]]);
        let is_valid_page_size = raw_page_size == 1 || (raw_page_size >= 512 && raw_page_size.is_power_of_two());
        let write_ver = header[18];
        let read_ver = header[19];
        let is_valid_versions = (write_ver == 1 || write_ver == 2) && (read_ver == 1 || read_ver == 2);

        if is_valid_page_size && is_valid_versions {
            evidence.push(EvidenceItem::new("sig.sqlite.page_size"));
            Verification::Structure
        } else {
            Verification::Signature
        }
    } else {
        Verification::Signature
    };

    let (fmt_id, cat, container, rec_ext, accepted) = if header.len() >= 72
        && (&header[68..72] == b"GPKG" || &header[68..72] == b"GP10")
    {
        evidence.push(EvidenceItem::with_desc("sig.sqlite.geopackage", "OGC GeoPackage container"));
        ("gpkg", "database", Some("sqlite3".to_string()), "gpkg", vec!["gpkg".to_string(), "sqlite3".to_string(), "db".to_string()])
    } else {
        ("sqlite3", "database", None, "sqlite3", vec!["sqlite3".to_string(), "sqlite".to_string(), "db".to_string(), "db3".to_string()])
    };

    SqliteMatch::Confirmed(Box::new(DetectionReport {
        schema_version: 1,
        format_id: Some(fmt_id.to_string()),
        category: cat.to_string(),
        container_id: container,
        identification: Identification::Format,
        verification,
        inspection: Inspection::Complete,
        recommended_extension: Some(rec_ext.to_string()),
        accepted_extensions: accepted,
        extension_status: ExtensionStatus::Undetermined,
        evidence,
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }))
}
