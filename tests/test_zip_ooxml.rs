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

#[test]
fn test_ooxml_case_insensitive_content_types() {
    use std::io::Write;
    use kindprobe::detect_bytes;

    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = zip::write::SimpleFileOptions::default();

        // 1. [Content_Types].xml with uppercase Default Extension and uppercase Override PartName
        let ct_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="XML" ContentType="application/xml"/>
    <Override PartName="/WORD/DOCUMENT.XML" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;
        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(ct_xml.as_bytes()).unwrap();

        // 2. _rels/.rels pointing to word/document.xml
        let rels_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;
        zip.start_file("_rels/.rels", options).unwrap();
        zip.write_all(rels_xml.as_bytes()).unwrap();

        // 3. word/document.xml with valid WordProcessingML
        let doc_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:body/>
</w:document>"#;
        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(doc_xml.as_bytes()).unwrap();

        zip.finish().unwrap();
    }

    let options = DetectOptions::default();
    let rep = detect_bytes(&buf, &options).unwrap();
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("docx"));
    assert_eq!(rep.inspection, Inspection::Complete);
}
