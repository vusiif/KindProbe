use std::path::Path;
use kindprobe::budget::DetectOptions;
use kindprobe::{detect_path, Identification, Inspection, Verification};

#[test]
fn test_cfb_and_word_doc() {
    let options = DetectOptions::default();

    // Plain CFB (not Word)
    let rep = detect_path(Path::new("fixtures/plain_cfb.cfb"), &options).unwrap();
    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
    assert_eq!(rep.container_id.as_deref(), Some("cfb"));
    assert_eq!(rep.inspection, Inspection::Complete);
    assert_eq!(rep.recommended_extension, None);

    // Fake DOC in CFB (WordDocument stream name with invalid FIB)
    let rep = detect_path(Path::new("fixtures/fake_doc_cfb.doc"), &options).unwrap();
    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
    assert_eq!(rep.container_id.as_deref(), Some("cfb"));
    assert_eq!(rep.recommended_extension, None);
    assert!(rep.warnings.iter().any(|w| w.code == "cfb.invalid_word_fib"));

    // True Word 97+ DOC
    let rep = detect_path(Path::new("fixtures/true_doc.doc"), &options).unwrap();
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("doc"));
    assert_eq!(rep.container_id.as_deref(), Some("cfb"));
    assert_eq!(rep.verification, Verification::Structure);
    assert_eq!(rep.inspection, Inspection::Complete);
    assert_eq!(rep.recommended_extension.as_deref(), Some("doc"));
    assert!(rep.evidence.iter().any(|e| e.code == "cfb.fib_word97"));

    // Cyclic CFB chain
    let rep = detect_path(Path::new("fixtures/cfb_cyclic_chain.doc"), &options).unwrap();
    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
}

#[test]
fn test_cfb_case_insensitive_table_stream_open() {
    use std::io::Write;
    use kindprobe::detect_bytes;

    let cursor = std::io::Cursor::new(Vec::new());
    let mut comp = cfb::CompoundFile::create(cursor).unwrap();

    let mut fib = vec![0u8; 64];
    fib[0..2].copy_from_slice(&0xA5ECu16.to_le_bytes()); // wIdent
    fib[2..4].copy_from_slice(&0x00C1u16.to_le_bytes()); // nFib
    fib[10..12].copy_from_slice(&0x1000u16.to_le_bytes()); // flags: bit 9 (0x0200) is 0 => 0Table
    fib[12..14].copy_from_slice(&0x00BFu16.to_le_bytes());

    comp.create_stream("/WordDocument").unwrap().write_all(&fib).unwrap();
    // Create table stream with ALL UPPERCASE name: /0TABLE
    comp.create_stream("/0TABLE").unwrap().write_all(&[0u8; 64]).unwrap();

    let inner = comp.into_inner();
    let data = inner.into_inner();

    let options = DetectOptions::default();
    let rep = detect_bytes(&data, &options).unwrap();
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("doc"));
    assert_eq!(rep.inspection, Inspection::Complete);
}
