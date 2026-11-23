use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static GEOSPATIAL_VECTOR_CAD_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "gml", category: "gis", container_id: None, recommended_ext: "gml", accepted_exts: &["gml"],
        patterns: &[MagicPattern::new(0, b"<gml:FeatureCollection ")], min_header_len: 24, priority: 50,
        evidence_code: "rule.gml.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "pbf", category: "gis", container_id: None, recommended_ext: "pbf", accepted_exts: &["pbf"],
        patterns: &[MagicPattern::new(0, b"\x00\x00\x00\x0dOSMHeader\x08")], min_header_len: 18, priority: 50,
        evidence_code: "rule.pbf.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "wor", category: "gis", container_id: None, recommended_ext: "wor", accepted_exts: &["wor"],
        patterns: &[MagicPattern::new(0, b"!workspace")], min_header_len: 10, priority: 50,
        evidence_code: "rule.wor.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "apr", category: "gis", container_id: None, recommended_ext: "apr", accepted_exts: &["apr"],
        patterns: &[MagicPattern::new(0, b"/3.0\n(ODB.1")], min_header_len: 11, priority: 50,
        evidence_code: "rule.apr.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "qgs", category: "gis", container_id: None, recommended_ext: "qgs", accepted_exts: &["qgs"],
        patterns: &[MagicPattern::new(0, b"<qgis ")], min_header_len: 6, priority: 50,
        evidence_code: "rule.qgs.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dgn", category: "cad", container_id: None, recommended_ext: "dgn", accepted_exts: &["dgn"],
        patterns: &[MagicPattern::new(0, b"\x08\x05\x19\x00")], min_header_len: 4, priority: 50,
        evidence_code: "rule.dgn.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "landxml", category: "cad", container_id: None, recommended_ext: "landxml", accepted_exts: &["landxml"],
        patterns: &[MagicPattern::new(0, b"<LandXML ")], min_header_len: 9, priority: 50,
        evidence_code: "rule.landxml.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "citygml", category: "gis", container_id: None, recommended_ext: "citygml", accepted_exts: &["citygml"],
        patterns: &[MagicPattern::new(0, b"<CityModel ")], min_header_len: 11, priority: 50,
        evidence_code: "rule.citygml.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cpg", category: "gis", container_id: None, recommended_ext: "cpg", accepted_exts: &["cpg"],
        patterns: &[MagicPattern::new(0, b"UTF-8")], min_header_len: 5, priority: 45,
        evidence_code: "rule.cpg.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "prj", category: "gis", container_id: None, recommended_ext: "prj", accepted_exts: &["prj"],
        patterns: &[MagicPattern::new(0, b"GEOGCS[\"")], min_header_len: 8, priority: 50,
        evidence_code: "rule.prj.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "e00_cov", category: "gis", container_id: None, recommended_ext: "cov", accepted_exts: &["cov"],
        patterns: &[MagicPattern::new(0, b"ARC  2")], min_header_len: 6, priority: 50,
        evidence_code: "rule.e00_cov.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "gxt", category: "gis", container_id: None, recommended_ext: "gxt", accepted_exts: &["gxt"],
        patterns: &[MagicPattern::new(0, b"GXT")], min_header_len: 3, priority: 50,
        evidence_code: "rule.gxt.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "gpx_trk", category: "gis", container_id: None, recommended_ext: "trk", accepted_exts: &["trk"],
        patterns: &[MagicPattern::new(0, b"<gpx ")], min_header_len: 5, priority: 45,
        evidence_code: "rule.gpx_trk.magic", verification: Verification::Signature, validator: None,
    },
];
