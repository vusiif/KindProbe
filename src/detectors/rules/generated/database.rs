use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static DATABASE_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "arrow", category: "database", container_id: None, recommended_ext: "arrow", accepted_exts: &["arrow", "feather"],
        patterns: &[MagicPattern::new(0, b"ARROW1")],
        min_header_len: 6, priority: 50, evidence_code: "rule.arrow.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "kdbx", category: "database", container_id: None, recommended_ext: "kdbx", accepted_exts: &["kdbx"],
        patterns: &[MagicPattern::new(0, b"\x03\xd9\xa2\x9ag\xfbK\xb5")],
        min_header_len: 8, priority: 50, evidence_code: "rule.kdbx.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "kdb", category: "database", container_id: None, recommended_ext: "kdb", accepted_exts: &["kdb"],
        patterns: &[MagicPattern::new(0, b"\x03\xd9\xa2\x9ae\xfbK\xb5")],
        min_header_len: 8, priority: 50, evidence_code: "rule.kdb.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "realm", category: "database", container_id: None,
        recommended_ext: "realm", accepted_exts: &["realm"],
        patterns: &[MagicPattern::new(0, b"T-D-B!")],
        min_header_len: 6, priority: 50, evidence_code: "rule.realm.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "las", category: "scientific", container_id: None,
        recommended_ext: "las", accepted_exts: &["las", "laz"],
        patterns: &[MagicPattern::new(0, b"LASF")],
        min_header_len: 4, priority: 50, evidence_code: "rule.las.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "mdb", category: "database", container_id: None, recommended_ext: "mdb", accepted_exts: &["mdb"],
        patterns: &[MagicPattern::new(0, b"\x00\x01\x00\x00Standard Jet DB")], min_header_len: 19,
        priority: 50, evidence_code: "rule.mdb.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "accdb", category: "database", container_id: None, recommended_ext: "accdb", accepted_exts: &["accdb"],
        patterns: &[MagicPattern::new(0, b"\x00\x01\x00\x00Standard ACE DB")], min_header_len: 19,
        priority: 50, evidence_code: "rule.accdb.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "snoop", category: "network", container_id: None,
        recommended_ext: "snoop", accepted_exts: &["snoop"],
        patterns: &[MagicPattern::new(0, b"snoop\x00\x00\x00")],
        min_header_len: 8, priority: 50, evidence_code: "rule.snoop.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "duckdb", category: "database", container_id: None,
        recommended_ext: "duckdb", accepted_exts: &["duckdb", "ddb"],
        patterns: &[MagicPattern::new(8, b"DUCK")],
        min_header_len: 12, priority: 50, evidence_code: "rule.duckdb.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cbor", category: "database", container_id: None,
        recommended_ext: "cbor", accepted_exts: &["cbor"],
        patterns: &[MagicPattern::new(0, b"\xd9\xd9\xf7")],
        min_header_len: 3, priority: 50, evidence_code: "rule.cbor.magic",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "sql", category: "database", container_id: None,
        recommended_ext: "sql", accepted_exts: &["sql"],
        patterns: &[MagicPattern::new(0, b"CREATE TABLE ")],
        min_header_len: 13, priority: 50, evidence_code: "rule.sql.create",
        verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "orc", category: "database", container_id: None, recommended_ext: "orc",
        accepted_exts: &["orc"], patterns: &[MagicPattern::new(0, b"ORC")],
        min_header_len: 3, priority: 50, evidence_code: "rule.orc.magic", verification: Verification::Signature, validator: None,
    },
];
