use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static COMPILER_IR_AND_ASSEMBLIES_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "llvm_mir_yaml", category: "bytecode", container_id: None, recommended_ext: "mir_yaml", accepted_exts: &["mir_yaml"],
        patterns: &[MagicPattern::new(0, b"---\nname:            ")], min_header_len: 21, priority: 50,
        evidence_code: "rule.llvm_mir_yaml.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "swift_module_sib", category: "bytecode", container_id: None, recommended_ext: "sib", accepted_exts: &["sib"],
        patterns: &[MagicPattern::new(0, b"SIBF\x00\x00\x00\x01")], min_header_len: 8, priority: 50,
        evidence_code: "rule.swift_module_sib.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "graal_truffle_cache", category: "bytecode", container_id: None, recommended_ext: "tcache", accepted_exts: &["tcache"],
        patterns: &[MagicPattern::new(0, b"TRUFFLE_CODE_CACHE\x00")], min_header_len: 19, priority: 50,
        evidence_code: "rule.graal_truffle_cache.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dalvik_odex", category: "bytecode", container_id: None, recommended_ext: "odex", accepted_exts: &["odex"],
        patterns: &[MagicPattern::new(0, b"dey\n036\x00")], min_header_len: 8, priority: 50,
        evidence_code: "rule.dalvik_odex.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dalvik_vdex", category: "bytecode", container_id: None, recommended_ext: "vdex", accepted_exts: &["vdex"],
        patterns: &[MagicPattern::new(0, b"vdex021\x00")], min_header_len: 8, priority: 50,
        evidence_code: "rule.dalvik_vdex.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dalvik_oat", category: "bytecode", container_id: None, recommended_ext: "oat", accepted_exts: &["oat"],
        patterns: &[MagicPattern::new(0, b"oat\n079\x00")], min_header_len: 8, priority: 50,
        evidence_code: "rule.dalvik_oat.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cranelift_clif", category: "bytecode", container_id: None, recommended_ext: "clif", accepted_exts: &["clif"],
        patterns: &[MagicPattern::new(0, b"test compile\ntarget ")], min_header_len: 20, priority: 50,
        evidence_code: "rule.cranelift_clif.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "gimple_dump", category: "bytecode", container_id: None, recommended_ext: "gimple", accepted_exts: &["gimple"],
        patterns: &[MagicPattern::new(0, b";; Function ")], min_header_len: 12, priority: 50,
        evidence_code: "rule.gimple_dump.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "spirv_assembly", category: "bytecode", container_id: None, recommended_ext: "spvasm", accepted_exts: &["spvasm"],
        patterns: &[MagicPattern::new(0, b"; SPIR-V\n; Version: 1.")], min_header_len: 21, priority: 50,
        evidence_code: "rule.spirv_assembly.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "dotnet_cil_text", category: "bytecode", container_id: None, recommended_ext: "cil", accepted_exts: &["cil"],
        patterns: &[MagicPattern::new(0, b".assembly extern mscorlib\n")], min_header_len: 27, priority: 50,
        evidence_code: "rule.dotnet_cil_text.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "elixir_beam_chunk", category: "bytecode", container_id: None, recommended_ext: "exir", accepted_exts: &["exir"],
        patterns: &[MagicPattern::new(0, b"EXIR\x00\x00\x00\x01")], min_header_len: 8, priority: 50,
        evidence_code: "rule.elixir_beam_chunk.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "v8_code_cache", category: "bytecode", container_id: None, recommended_ext: "v8cache", accepted_exts: &["v8cache"],
        patterns: &[MagicPattern::new(0, b"V8_CACHE_V1\x00")], min_header_len: 12, priority: 50,
        evidence_code: "rule.v8_code_cache.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "mir_rust_dump", category: "bytecode", container_id: None, recommended_ext: "mir", accepted_exts: &["mir"],
        patterns: &[MagicPattern::new(0, b"// WARNING: This output format is intended for human consumers only\nfn ")], min_header_len: 70, priority: 50,
        evidence_code: "rule.mir_rust_dump.magic", verification: Verification::Signature, validator: None,
    },
];
