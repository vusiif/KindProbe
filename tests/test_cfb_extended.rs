use std::io::{Cursor, Write};
use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Identification, Inspection, Verification};
use std::ffi::OsStr;

fn create_cfb_xls(stream_name: &str, biff_version: u16) -> Vec<u8> {
    let cursor = Cursor::new(Vec::new());
    let mut comp = cfb::CompoundFile::create(cursor).expect("create cfb");

    // BIFF BOF record: opcode = 0x0809, length = 16, version = biff_version, substream = 0x0005 (Workbook)
    let mut bof = [0u8; 16];
    bof[0..2].copy_from_slice(&0x0809u16.to_le_bytes()); // BOF
    bof[2..4].copy_from_slice(&16u16.to_le_bytes()); // len
    bof[4..6].copy_from_slice(&biff_version.to_le_bytes()); // vers
    bof[6..8].copy_from_slice(&0x0005u16.to_le_bytes()); // dt = Workbook

    {
        let mut stream = comp.create_stream(format!("/{}", stream_name)).expect("create stream");
        stream.write_all(&bof).expect("write bof");
        // Write extra padding bytes
        stream.write_all(&[0u8; 48]).expect("write padding");
    }

    let inner = comp.into_inner();
    inner.into_inner()
}

fn create_cfb_ppt(has_current_user: bool, ppt_stream_len: usize) -> Vec<u8> {
    let cursor = Cursor::new(Vec::new());
    let mut comp = cfb::CompoundFile::create(cursor).expect("create cfb");

    let ppt_data = vec![0xAAu8; ppt_stream_len];
    {
        let mut stream = comp.create_stream("/PowerPoint Document").expect("create ppt stream");
        stream.write_all(&ppt_data).expect("write ppt");
    }

    if has_current_user {
        let mut user_stream = comp.create_stream("/Current User").expect("create user stream");
        user_stream.write_all(&[0u8; 20]).expect("write user");
    }

    let inner = comp.into_inner();
    inner.into_inner()
}

#[test]
fn test_detect_xls_biff8_success() {
    let data = create_cfb_xls("Workbook", 0x0600); // Excel 97-2003 BIFF8
    let rep = detect_bytes(&data, &DetectOptions::default()).expect("detect xls");

    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("xls"));
    assert_eq!(rep.category, "document");
    assert_eq!(rep.container_id.as_deref(), Some("cfb"));
    assert_eq!(rep.verification, Verification::Structure);
    assert_eq!(rep.inspection, Inspection::Complete);
    assert_eq!(rep.recommended_extension.as_deref(), Some("xls"));
    assert_eq!(rep.accepted_extensions, vec!["xls", "xlt"]);
    assert!(rep.evidence.iter().any(|e| e.code == "cfb.workbook_stream"));
    assert!(rep.evidence.iter().any(|e| e.code == "cfb.biff_record"));

    // Extension matching
    assert_eq!(compare_extension(&rep, Some(OsStr::new("xls"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("XLT"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("docx"))), ExtensionStatus::Mismatch);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("ppt"))), ExtensionStatus::Mismatch);
}

#[test]
fn test_detect_xls_biff5_book_stream() {
    let data = create_cfb_xls("Book", 0x0500); // Excel 5.0/95 BIFF5
    let rep = detect_bytes(&data, &DetectOptions::default()).expect("detect biff5");

    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("xls"));
    assert_eq!(rep.container_id.as_deref(), Some("cfb"));
    assert_eq!(rep.verification, Verification::Structure);
}

#[test]
fn test_detect_ppt_success() {
    let data = create_cfb_ppt(true, 64);
    let rep = detect_bytes(&data, &DetectOptions::default()).expect("detect ppt");

    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("ppt"));
    assert_eq!(rep.category, "document");
    assert_eq!(rep.container_id.as_deref(), Some("cfb"));
    assert_eq!(rep.verification, Verification::Structure);
    assert_eq!(rep.inspection, Inspection::Complete);
    assert_eq!(rep.recommended_extension.as_deref(), Some("ppt"));
    assert_eq!(rep.accepted_extensions, vec!["ppt", "pot", "pps"]);
    assert!(rep.evidence.iter().any(|e| e.code == "cfb.powerpoint_stream"));
    assert!(rep.evidence.iter().any(|e| e.code == "cfb.current_user_stream"));

    // Extension matching
    assert_eq!(compare_extension(&rep, Some(OsStr::new("ppt"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("pot"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("pps"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("pdf"))), ExtensionStatus::Mismatch);
}

#[test]
fn test_detect_ppt_without_current_user() {
    let data = create_cfb_ppt(false, 40);
    let rep = detect_bytes(&data, &DetectOptions::default()).expect("detect ppt");

    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("ppt"));
    assert!(rep.evidence.iter().any(|e| e.code == "cfb.powerpoint_stream"));
    assert!(!rep.evidence.iter().any(|e| e.code == "cfb.current_user_stream"));
}

#[test]
fn test_detect_xls_invalid_bof() {
    let cursor = Cursor::new(Vec::new());
    let mut comp = cfb::CompoundFile::create(cursor).expect("create cfb");

    // Invalid opcode 0x1234
    let mut bof = [0u8; 16];
    bof[0..2].copy_from_slice(&0x1234u16.to_le_bytes());
    {
        let mut stream = comp.create_stream("/Workbook").expect("create stream");
        stream.write_all(&bof).expect("write bof");
    }

    let inner = comp.into_inner();
    let data = inner.into_inner();

    let rep = detect_bytes(&data, &DetectOptions::default()).expect("detect invalid bof");
    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
    assert_eq!(rep.container_id.as_deref(), Some("cfb"));
    assert!(rep.warnings.iter().any(|w| w.code == "cfb.invalid_biff_bof"));
}

#[test]
fn test_detect_xls_unsupported_biff_version() {
    let data = create_cfb_xls("Workbook", 0x0200); // BIFF2 (not supported in CFB)
    let rep = detect_bytes(&data, &DetectOptions::default()).expect("detect biff2");

    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
    assert!(rep.warnings.iter().any(|w| w.code == "cfb.unsupported_biff_version"));
}

#[test]
fn test_detect_ppt_stream_too_short() {
    let data = create_cfb_ppt(false, 16); // < 32 bytes
    let rep = detect_bytes(&data, &DetectOptions::default()).expect("detect short ppt");

    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
    assert!(rep.warnings.iter().any(|w| w.code == "cfb.invalid_powerpoint_stream"));
}
