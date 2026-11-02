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
