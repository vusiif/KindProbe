use kindprobe::budget::DetectOptions;
use kindprobe::{detect_path, Identification, Inspection};
use std::path::Path;

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
    assert!(rep
        .warnings
        .iter()
        .any(|w| w.code == "ooxml.content_types_missing"));

    // Partial OOXML: missing rels
    let rep = detect_path(
        Path::new("fixtures/partial_ooxml_missing_rels.zip"),
        &options,
    )
    .unwrap();
    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
    assert!(rep
        .warnings
        .iter()
        .any(|w| w.code == "ooxml.required_relationship_missing"));

    // Partial OOXML: missing main part
    let rep = detect_path(
        Path::new("fixtures/partial_ooxml_missing_main_part.zip"),
        &options,
    )
    .unwrap();
    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
    assert!(rep
        .warnings
        .iter()
        .any(|w| w.code == "ooxml.main_part_missing"));
}

#[test]
fn test_ooxml_case_insensitive_content_types() {
    use kindprobe::detect_bytes;
    use std::io::Write;

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

#[test]
fn test_opc_path_traversal_and_conflicting_rels_rejected() {
    use kindprobe::detect_bytes;
    use std::io::Write;

    let options = DetectOptions::default();

    // 1. Official verifier fixture: ../../word/document.xml must be rejected
    let rep_trav = detect_path(
        Path::new("fixtures/verifier/adv_fake_docx_path_traversal.docx"),
        &options,
    )
    .unwrap();
    assert_eq!(rep_trav.identification, Identification::ContainerOnly);
    assert_eq!(rep_trav.format_id, None);
    assert_eq!(rep_trav.container_id.as_deref(), Some("zip"));
    assert!(rep_trav
        .warnings
        .iter()
        .any(|w| w.code == "ooxml.invalid_target_path"));

    // 2. Official verifier round1 fixture: conflicting multiple officeDocument relationships
    let rep_conf = detect_path(
        Path::new("fixtures/verifier_round1/adv_conflicting_rels.docx"),
        &options,
    )
    .unwrap();
    assert_eq!(rep_conf.identification, Identification::ContainerOnly);
    assert_eq!(rep_conf.format_id, None);
    assert_eq!(rep_conf.container_id.as_deref(), Some("zip"));
    assert!(rep_conf
        .warnings
        .iter()
        .any(|w| w.code == "ooxml.conflicting_relationships"));

    // 3. Single leading "../" target and legal internal "word/../word/document.xml"
    let build_docx_with_target = |target: &str| -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            let zo = zip::write::SimpleFileOptions::default();
            let ct = r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
            let rels = format!(
                r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="{}"/></Relationships>"#,
                target
            );
            zip.start_file("[Content_Types].xml", zo).unwrap();
            zip.write_all(ct.as_bytes()).unwrap();
            zip.start_file("_rels/.rels", zo).unwrap();
            zip.write_all(rels.as_bytes()).unwrap();
            zip.start_file("word/document.xml", zo).unwrap();
            zip.write_all(br#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body/></w:document>"#)
                .unwrap();
            zip.finish().unwrap();
        }
        buf
    };

    let rep_single_escape =
        detect_bytes(&build_docx_with_target("../word/document.xml"), &options).unwrap();
    assert_eq!(
        rep_single_escape.identification,
        Identification::ContainerOnly
    );
    assert_eq!(rep_single_escape.format_id, None);

    let rep_internal_ok = detect_bytes(
        &build_docx_with_target("word/../word/document.xml"),
        &options,
    )
    .unwrap();
    assert_eq!(rep_internal_ok.identification, Identification::Format);
    assert_eq!(rep_internal_ok.format_id.as_deref(), Some("docx"));
}

#[test]
fn test_real_opc_vsix_and_nupkg_not_swallowed_by_ooxml() {
    use kindprobe::detect_bytes;
    use std::io::Write;

    let options = DetectOptions::default();
    let make_opc_pkg = |manifest_name: &str, manifest_body: &[u8]| -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            let zo = zip::write::SimpleFileOptions::default();
            zip.start_file("[Content_Types].xml", zo).unwrap();
            zip.write_all(
                br#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"/>"#,
            )
            .unwrap();
            zip.start_file("_rels/.rels", zo).unwrap();
            zip.write_all(br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#)
                .unwrap();
            zip.start_file(manifest_name, zo).unwrap();
            zip.write_all(manifest_body).unwrap();
            zip.finish().unwrap();
        }
        buf
    };

    let rep_vsix = detect_bytes(
        &make_opc_pkg("extension.vsixmanifest", b"<PackageManifest/>"),
        &options,
    )
    .unwrap();
    assert_eq!(rep_vsix.identification, Identification::Format);
    assert_eq!(rep_vsix.format_id.as_deref(), Some("vsix"));

    let rep_nupkg = detect_bytes(
        &make_opc_pkg("KindProbe.nuspec", b"<package><metadata/></package>"),
        &options,
    )
    .unwrap();
    assert_eq!(rep_nupkg.identification, Identification::Format);
    assert_eq!(rep_nupkg.format_id.as_deref(), Some("nupkg"));
}
