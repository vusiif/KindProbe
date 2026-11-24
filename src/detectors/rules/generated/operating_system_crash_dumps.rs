use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static OPERATING_SYSTEM_CRASH_DUMPS_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "macos_panic_report", category: "forensics", container_id: None, recommended_ext: "panic", accepted_exts: &["panic"],
        patterns: &[MagicPattern::new(0, b"{\"macOS_panic\":{\"version\":\"1.0\"")], min_header_len: 32, priority: 50,
        evidence_code: "rule.macos_panic_report.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "linux_vmcore_elf_note", category: "forensics", container_id: None, recommended_ext: "vmcoreinfo", accepted_exts: &["vmcoreinfo"],
        patterns: &[MagicPattern::new(0, b"VMCOREINFO\n")], min_header_len: 11, priority: 50,
        evidence_code: "rule.linux_vmcore_elf_note.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "solaris_savecore_dump", category: "forensics", container_id: None, recommended_ext: "savecore", accepted_exts: &["savecore"],
        patterns: &[MagicPattern::new(0, b"SolarisSavecoreDumpHeaderV1\x00")], min_header_len: 28, priority: 50,
        evidence_code: "rule.solaris_savecore_dump.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "aix_core_dump", category: "forensics", container_id: None, recommended_ext: "aixcore", accepted_exts: &["aixcore"],
        patterns: &[MagicPattern::new(0, b"\x01\xdf\x01\x02AIX_CORE")], min_header_len: 12, priority: 50,
        evidence_code: "rule.aix_core_dump.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "freebsd_minidump", category: "forensics", container_id: None, recommended_ext: "fbsddump", accepted_exts: &["fbsddump"],
        patterns: &[MagicPattern::new(0, b"FreeBSDMinidumpHeader\x00")], min_header_len: 22, priority: 50,
        evidence_code: "rule.freebsd_minidump.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "netbsd_core_dump", category: "forensics", container_id: None, recommended_ext: "nbsdcore", accepted_exts: &["nbsdcore"],
        patterns: &[MagicPattern::new(0, b"NetBSD-CORE\x00")], min_header_len: 12, priority: 50,
        evidence_code: "rule.netbsd_core_dump.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "hpux_crash_dump", category: "forensics", container_id: None, recommended_ext: "hpuxdump", accepted_exts: &["hpuxdump"],
        patterns: &[MagicPattern::new(0, b"HPUX_CRASH_DUMP_V1\x00")], min_header_len: 19, priority: 50,
        evidence_code: "rule.hpux_crash_dump.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "vxworks_core_dump", category: "forensics", container_id: None, recommended_ext: "vxcore", accepted_exts: &["vxcore"],
        patterns: &[MagicPattern::new(0, b"WRS_VXWORKS_CORE\x00")], min_header_len: 17, priority: 50,
        evidence_code: "rule.vxworks_core_dump.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "qnx_core_dump", category: "forensics", container_id: None, recommended_ext: "qnxcore", accepted_exts: &["qnxcore"],
        patterns: &[MagicPattern::new(0, b"QNX6_CORE_DUMP\x00")], min_header_len: 15, priority: 50,
        evidence_code: "rule.qnx_core_dump.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "openvms_crash_dump", category: "forensics", container_id: None, recommended_ext: "vmsdump", accepted_exts: &["vmsdump"],
        patterns: &[MagicPattern::new(0, b"OPENVMS_DUMP_V1\x00")], min_header_len: 16, priority: 50,
        evidence_code: "rule.openvms_crash_dump.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "apple_spin_dump", category: "forensics", container_id: None, recommended_ext: "spindump", accepted_exts: &["spindump"],
        patterns: &[MagicPattern::new(0, b"Date/Time:       20\nOS Version:      macOS")], min_header_len: 43, priority: 50,
        evidence_code: "rule.apple_spin_dump.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "android_tombstone", category: "forensics", container_id: None, recommended_ext: "tombstone", accepted_exts: &["tombstone"],
        patterns: &[MagicPattern::new(0, b"*** *** *** *** *** *** *** ***\nBuild fingerprint:")], min_header_len: 50, priority: 50,
        evidence_code: "rule.android_tombstone.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "systemd_coredump", category: "forensics", container_id: None, recommended_ext: "core_zst", accepted_exts: &["core_zst"],
        patterns: &[MagicPattern::new(0, b"SYSTEMD_COREDUMP_METADATA\n")], min_header_len: 26, priority: 50,
        evidence_code: "rule.systemd_coredump.magic", verification: Verification::Signature, validator: None,
    },
];
