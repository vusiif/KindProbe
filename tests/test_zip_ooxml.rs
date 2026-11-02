use std::path::Path;
use kindprobe::budget::DetectOptions;
use kindprobe::{detect_path, Identification, Inspection};

#[test]
fn test_plain_zip_vs_docx() {
    let options = DetectOptions::default();

    // Plain ZIP
    let rep = detect_path(Path::new("fixtures/plain_zip.zip"), &options).unwrap();
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("zip"));
    assert_eq!(rep.container_id.as_deref(), Some("zip"));
    assert_eq!(rep.inspection, Inspection::Complete);
    assert_eq!(rep.recommended_extension.as_deref(), Some("zip"));

    // True DOCX
    let rep = detect_path(Path::new("fixtures/true_docx.docx"), &options).unwrap();
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("docx"));
    assert_eq!(rep.container_id.as_deref(), Some("zip"));
    assert_eq!(rep.inspection, Inspection::Complete);
    assert_eq!(rep.recommended_extension.as_deref(), Some("docx"));

    // True DOCX Strict
    let rep = detect_path(Path::new("fixtures/true_docx_strict.docx"), &options).unwrap();
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("docx"));
    assert_eq!(rep.inspection, Inspection::Complete);

    // True DOCX Relative Path
    let rep = detect_path(Path::new("fixtures/true_docx_rel_path.docx"), &options).unwrap();
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("docx"));
    assert_eq!(rep.inspection, Inspection::Complete);
}

#[test]
fn test_fake_docx_and_partial_ooxml() {
    let options = DetectOptions::default();

    // Fake DOCX ZIP (contains only word/document.xml, missing Content_Types)
    let rep = detect_path(Path::new("fixtures/fake_docx_zip.zip"), &options).unwrap();
    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
    assert_eq!(rep.container_id.as_deref(), Some("zip"));
    assert_eq!(rep.recommended_extension, None);
    assert!(rep.warnings.iter().any(|w| w.code == "ooxml.content_types_missing"));

    // Partial OOXML: missing rels
    let rep = detect_path(Path::new("fixtures/partial_ooxml_missing_rels.zip"), &options).unwrap();
    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
    assert!(rep.warnings.iter().any(|w| w.code == "ooxml.required_relationship_missing"));

    // Partial OOXML: missing main part
    let rep = detect_path(Path::new("fixtures/partial_ooxml_missing_main_part.zip"), &options).unwrap();
    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
    assert!(rep.warnings.iter().any(|w| w.code == "ooxml.main_part_missing"));
}
