use std::ffi::OsStr;
use std::io::Cursor;
use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{
    detect_bytes, detect_reader, ExtensionStatus, Identification, Inspection, Verification,
};

#[test]
fn test_detect_qcow2() {
    let opts = DetectOptions::default();

    let mut qcow = vec![0u8; 32];
    qcow[0..4].copy_from_slice(&[0x51, 0x46, 0x49, 0xFB]); // "QFI\xFB"
    qcow[4..8].copy_from_slice(&3u32.to_be_bytes()); // version = 3 (QCOW2 v3)
    qcow[20..24].copy_from_slice(&16u32.to_be_bytes()); // cluster_bits = 16 (64KB clusters)

    let rep = detect_bytes(&qcow, &opts).expect("detect qcow2");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("qcow2"));
    assert_eq!(rep.category, "disk");
    assert_eq!(rep.verification, Verification::Structure);
    assert_eq!(rep.inspection, Inspection::Complete);
    assert_eq!(rep.recommended_extension.as_deref(), Some("qcow2"));
    assert_eq!(rep.accepted_extensions, vec!["qcow2", "qcow", "img"]);
    assert!(rep.evidence.iter().any(|e| e.code == "sig.qcow.magic"));
    assert!(rep.evidence.iter().any(|e| e.code == "sig.qcow.header"));

    assert_eq!(compare_extension(&rep, Some(OsStr::new("qcow2"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("qcow"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("img"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("vmdk"))), ExtensionStatus::Mismatch);
}

#[test]
fn test_detect_vmdk() {
    let opts = DetectOptions::default();

    // 1. Sparse VMDK binary
    let mut sparse_vmdk = vec![0u8; 16];
    sparse_vmdk[0..4].copy_from_slice(b"KDMV");
    sparse_vmdk[4..8].copy_from_slice(&1u32.to_le_bytes()); // version = 1

    let rep_sparse = detect_bytes(&sparse_vmdk, &opts).expect("detect sparse vmdk");
    assert_eq!(rep_sparse.identification, Identification::Format);
    assert_eq!(rep_sparse.format_id.as_deref(), Some("vmdk"));
    assert_eq!(rep_sparse.category, "disk");
    assert_eq!(rep_sparse.verification, Verification::Structure);
    assert!(rep_sparse.evidence.iter().any(|e| e.code == "sig.vmdk.sparse_magic"));
    assert!(rep_sparse.evidence.iter().any(|e| e.code == "sig.vmdk.version"));
    assert_eq!(compare_extension(&rep_sparse, Some(OsStr::new("vmdk"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_sparse, Some(OsStr::new("vhdx"))), ExtensionStatus::Mismatch);

    // 2. Monolithic Flat descriptor VMDK
    let desc_vmdk = b"# Disk DescriptorFile\nversion=1\nCID=fffffffe\nparentCID=ffffffff\n";
    let rep_desc = detect_bytes(desc_vmdk, &opts).expect("detect desc vmdk");
    assert_eq!(rep_desc.identification, Identification::Format);
    assert_eq!(rep_desc.format_id.as_deref(), Some("vmdk"));
    assert_eq!(rep_desc.category, "disk");
    assert_eq!(rep_desc.verification, Verification::Structure);
    assert!(rep_desc.evidence.iter().any(|e| e.code == "sig.vmdk.desc_magic"));
    assert_eq!(compare_extension(&rep_desc, Some(OsStr::new("vmdk"))), ExtensionStatus::Match);
}

#[test]
fn test_detect_vhdx_and_vhd() {
    let opts = DetectOptions::default();

    // 1. VHDX format ("vhdxfile")
    let vhdx = b"vhdxfile\x00\x00\x00\x00";
    let rep_vhdx = detect_bytes(vhdx, &opts).expect("detect vhdx");
    assert_eq!(rep_vhdx.identification, Identification::Format);
    assert_eq!(rep_vhdx.format_id.as_deref(), Some("vhdx"));
    assert_eq!(rep_vhdx.category, "disk");
    assert_eq!(rep_vhdx.verification, Verification::Structure);
    assert!(rep_vhdx.evidence.iter().any(|e| e.code == "sig.vhdx.magic"));
    assert_eq!(compare_extension(&rep_vhdx, Some(OsStr::new("vhdx"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_vhdx, Some(OsStr::new("vhd"))), ExtensionStatus::Mismatch);

    // 2. VHD format ("conectix")
    let vhd = b"conectix\x00\x00\x00\x02";
    let rep_vhd = detect_bytes(vhd, &opts).expect("detect vhd");
    assert_eq!(rep_vhd.identification, Identification::Format);
    assert_eq!(rep_vhd.format_id.as_deref(), Some("vhd"));
    assert_eq!(rep_vhd.category, "disk");
    assert_eq!(rep_vhd.verification, Verification::Structure);
    assert!(rep_vhd.evidence.iter().any(|e| e.code == "sig.vhd.magic"));
    assert_eq!(compare_extension(&rep_vhd, Some(OsStr::new("vhd"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_vhd, Some(OsStr::new("vhdx"))), ExtensionStatus::Mismatch);
}

#[test]
fn test_detect_iso_stream() {
    let opts = DetectOptions::default();

    // ISO 9660 with Primary Volume Descriptor at offset 0x8000 (32768)
    let mut iso_data = vec![0u8; 0x8000 + 2048];
    iso_data[0x8000..0x8000 + 6].copy_from_slice(b"\x01CD001");

    let mut cursor = Cursor::new(iso_data);
    let rep_iso = detect_reader(&mut cursor, &opts).expect("detect iso");
    assert_eq!(rep_iso.identification, Identification::Format);
    assert_eq!(rep_iso.format_id.as_deref(), Some("iso"));
    assert_eq!(rep_iso.category, "disk");
    assert_eq!(rep_iso.verification, Verification::Structure);
    assert_eq!(rep_iso.recommended_extension.as_deref(), Some("iso"));
    assert_eq!(rep_iso.accepted_extensions, vec!["iso", "img"]);
    assert!(rep_iso.evidence.iter().any(|e| e.code == "sig.iso.volume_descriptor"));

    assert_eq!(compare_extension(&rep_iso, Some(OsStr::new("iso"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_iso, Some(OsStr::new("img"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_iso, Some(OsStr::new("qcow2"))), ExtensionStatus::Mismatch);
}

#[test]
fn test_detect_truncated_disks() {
    let opts = DetectOptions::default();

    let trunc_qcow = detect_bytes(b"QF", &opts).expect("detect truncated qcow");
    assert_eq!(trunc_qcow.identification, Identification::Unknown);
    assert!(trunc_qcow.warnings.iter().any(|w| w.code == "input.truncated"));

    let trunc_vmdk = detect_bytes(b"KD", &opts).expect("detect truncated vmdk");
    assert_eq!(trunc_vmdk.identification, Identification::Unknown);
    assert!(trunc_vmdk.warnings.iter().any(|w| w.code == "input.truncated"));
}
