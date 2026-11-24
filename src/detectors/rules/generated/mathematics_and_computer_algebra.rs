use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static MATHEMATICS_AND_COMPUTER_ALGEBRA_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "mathematica_notebook", category: "scientific", container_id: None, recommended_ext: "nb_wolfram", accepted_exts: &["nb_wolfram"],
        patterns: &[MagicPattern::new(0, b"(* Wolfram Notebook File *)")], min_header_len: 27, priority: 50,
        evidence_code: "rule.mathematica_notebook.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "maple_worksheet_mw", category: "scientific", container_id: None, recommended_ext: "mw_maple", accepted_exts: &["mw_maple"],
        patterns: &[MagicPattern::new(0, b"<Worksheet><Version major=")], min_header_len: 26, priority: 50,
        evidence_code: "rule.maple_worksheet_mw.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "maxima_wxmx_manifest", category: "scientific", container_id: None, recommended_ext: "wxmx_hdr", accepted_exts: &["wxmx_hdr"],
        patterns: &[MagicPattern::new(0, b"wxMaxima Document Header\x00")], min_header_len: 25, priority: 50,
        evidence_code: "rule.maxima_wxmx_manifest.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "mathcad_xmcd", category: "scientific", container_id: None, recommended_ext: "xmcd", accepted_exts: &["xmcd"],
        patterns: &[MagicPattern::new(0, b"<worksheet xmlns=\"http://schemas.mathsoft.com/worksheet30\"")], min_header_len: 59, priority: 50,
        evidence_code: "rule.mathcad_xmcd.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "r_workspace_rdata", category: "scientific", container_id: None, recommended_ext: "rdata7", accepted_exts: &["rdata7"],
        patterns: &[MagicPattern::new(0, b"RDA7\n")], min_header_len: 5, priority: 50,
        evidence_code: "rule.r_workspace_rdata.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "stata_dta_118", category: "scientific", container_id: None, recommended_ext: "dta118", accepted_exts: &["dta118"],
        patterns: &[MagicPattern::new(0, b"<stata_dta><header><release>118")], min_header_len: 32, priority: 50,
        evidence_code: "rule.stata_dta_118.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "spss_sav_system", category: "scientific", container_id: None, recommended_ext: "sav_spss", accepted_exts: &["sav_spss"],
        patterns: &[MagicPattern::new(0, b"$FL2@(#) SPSS DATA FILE")], min_header_len: 23, priority: 50,
        evidence_code: "rule.spss_sav_system.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "minitab_mtw", category: "scientific", container_id: None, recommended_ext: "mtw", accepted_exts: &["mtw"],
        patterns: &[MagicPattern::new(0, b"MINITAB_WORKSHEET_V1")], min_header_len: 20, priority: 50,
        evidence_code: "rule.minitab_mtw.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "geogebra_ggb_xml", category: "scientific", container_id: None, recommended_ext: "ggb_xml", accepted_exts: &["ggb_xml"],
        patterns: &[MagicPattern::new(0, b"<geogebra format=\"5.0\"")], min_header_len: 22, priority: 50,
        evidence_code: "rule.geogebra_ggb_xml.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "sage_math_sobj", category: "scientific", container_id: None, recommended_ext: "sobj", accepted_exts: &["sobj"],
        patterns: &[MagicPattern::new(0, b"SAGE_MATH_SERIALIZED_OBJ_V1")], min_header_len: 27, priority: 50,
        evidence_code: "rule.sage_math_sobj.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "pari_gp_script", category: "scientific", container_id: None, recommended_ext: "gp_script", accepted_exts: &["gp_script"],
        patterns: &[MagicPattern::new(0, b"/* PARI/GP Script File */\n")], min_header_len: 26, priority: 50,
        evidence_code: "rule.pari_gp_script.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "gap_groups_script", category: "scientific", container_id: None, recommended_ext: "gap_sys", accepted_exts: &["gap_sys"],
        patterns: &[MagicPattern::new(0, b"# GAP System File\n")], min_header_len: 18, priority: 50,
        evidence_code: "rule.gap_groups_script.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "coq_proof_v", category: "scientific", container_id: None, recommended_ext: "coq_v", accepted_exts: &["coq_v"],
        patterns: &[MagicPattern::new(0, b"(* Coq Proof Assistant Document *)\n")], min_header_len: 36, priority: 50,
        evidence_code: "rule.coq_proof_v.magic", verification: Verification::Signature, validator: None,
    },
];
