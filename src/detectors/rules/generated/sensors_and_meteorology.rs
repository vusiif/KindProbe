use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static SENSORS_AND_METEOROLOGY_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "bufr", category: "scientific", container_id: None, recommended_ext: "bufr", accepted_exts: &["bufr"],
        patterns: &[MagicPattern::new(0, b"BUFR")], min_header_len: 4, priority: 50,
        evidence_code: "rule.bufr.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "mcl", category: "scientific", container_id: None, recommended_ext: "mcl", accepted_exts: &["mcl"],
        patterns: &[MagicPattern::new(0, b"MCL1")], min_header_len: 4, priority: 50,
        evidence_code: "rule.mcl.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "nexrad", category: "scientific", container_id: None, recommended_ext: "ar2v", accepted_exts: &["ar2v"],
        patterns: &[MagicPattern::new(0, b"AR2V00")], min_header_len: 6, priority: 50,
        evidence_code: "rule.nexrad.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "iris", category: "scientific", container_id: None, recommended_ext: "iris", accepted_exts: &["iris"],
        patterns: &[MagicPattern::new(0, b"\x1a\x04\x00\x00\x00\x18")], min_header_len: 6, priority: 50,
        evidence_code: "rule.iris.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cinrad", category: "scientific", container_id: None, recommended_ext: "cinrad", accepted_exts: &["cinrad"],
        patterns: &[MagicPattern::new(0, b"CINRAD\x00")], min_header_len: 7, priority: 50,
        evidence_code: "rule.cinrad.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "e57", category: "scientific", container_id: None, recommended_ext: "e57", accepted_exts: &["e57"],
        patterns: &[MagicPattern::new(0, b"ASTM-E57")], min_header_len: 8, priority: 50,
        evidence_code: "rule.e57.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cwls_las", category: "scientific", container_id: None, recommended_ext: "las", accepted_exts: &["las"],
        patterns: &[MagicPattern::new(0, b"~VERSION")], min_header_len: 8, priority: 50,
        evidence_code: "rule.cwls_las.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "mseed", category: "scientific", container_id: None, recommended_ext: "mseed", accepted_exts: &["mseed"],
        patterns: &[MagicPattern::new(0, b"000001D ")], min_header_len: 8, priority: 50,
        evidence_code: "rule.mseed.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "ubx", category: "location", container_id: None, recommended_ext: "ubx", accepted_exts: &["ubx"],
        patterns: &[MagicPattern::new(0, b"\xb5\x62")], min_header_len: 2, priority: 50,
        evidence_code: "rule.ubx.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "nmea", category: "location", container_id: None, recommended_ext: "nmea", accepted_exts: &["nmea", "log"],
        patterns: &[MagicPattern::new(0, b"$GPRMC,")], min_header_len: 7, priority: 50,
        evidence_code: "rule.nmea.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "novatel_gps", category: "location", container_id: None, recommended_ext: "gps", accepted_exts: &["gps"],
        patterns: &[MagicPattern::new(0, b"\xaa\x44\x12")], min_header_len: 3, priority: 50,
        evidence_code: "rule.novatel_gps.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dorade", category: "scientific", container_id: None, recommended_ext: "dorade", accepted_exts: &["dorade"],
        patterns: &[MagicPattern::new(0, b"SSWB")], min_header_len: 4, priority: 50,
        evidence_code: "rule.dorade.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "fpa", category: "scientific", container_id: None, recommended_ext: "fpa", accepted_exts: &["fpa"],
        patterns: &[MagicPattern::new(0, b"FPA1\x00")], min_header_len: 5, priority: 50,
        evidence_code: "rule.fpa.magic", verification: Verification::Signature, validator: None,
    },
];
