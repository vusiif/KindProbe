use std::io::{Cursor, Write};
use std::path::PathBuf;
use kindprobe::budget::DetectOptions;
use kindprobe::{
    detect_bytes, detect_path, detect_reader, ExtensionStatus, Identification, Inspection,
    Verification,
};

fn create_docx_package(path: &PathBuf, document_xml: &str) {
    let file = std::fs::File::create(path).expect("create docx file");
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let ct = r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    let rels = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="r1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;

    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(ct.as_bytes()).unwrap();

    zip.start_file("_rels/.rels", options).unwrap();
    zip.write_all(rels.as_bytes()).unwrap();

    zip.start_file("word/document.xml", options).unwrap();
    zip.write_all(document_xml.as_bytes()).unwrap();

    zip.finish().unwrap();
}

fn create_cfb_case(
    path: &PathBuf,
    nested: bool,
    version: u16,
    flags: u16,
    create_table: bool,
) {
    let mut comp = cfb::create(path).expect("create cfb");
    let mut fib = vec![0u8; 64];
    fib[0..2].copy_from_slice(&0xA5ECu16.to_le_bytes());
    fib[2..4].copy_from_slice(&version.to_le_bytes());
    fib[10..12].copy_from_slice(&flags.to_le_bytes());
    fib[12..14].copy_from_slice(&0x00BFu16.to_le_bytes());

    if nested {
        comp.create_storage("/EmbeddedWord").unwrap();
        comp.create_stream("/Workbook")
            .unwrap()
            .write_all(b"OUTER WORKBOOK CONTAINER")
            .unwrap();
    }

    let prefix = if nested { "/EmbeddedWord/" } else { "/" };
    comp.create_stream(format!("{}WordDocument", prefix))
        .unwrap()
        .write_all(&fib)
        .unwrap();

    if create_table {
        let is_1table = (flags & 0x0200) != 0;
        let table_name = if is_1table { "1Table" } else { "0Table" };
        comp.create_stream(format!("{}{}", prefix, table_name))
            .unwrap()
            .write_all(&[0u8; 64])
            .unwrap();
    }
}

// 1. invalid_root_deep
#[test]
fn test_regression_invalid_root_deep() {
    let tmp = std::env::temp_dir().join("test_regression_invalid_root_deep.docx");
    let xml = format!("<evil>{}</evil>", "<x>".repeat(70) + &"</x>".repeat(70));
    create_docx_package(&tmp, &xml);

    let rep = detect_path(&tmp, &DetectOptions::default()).unwrap();
    let _ = std::fs::remove_file(&tmp);

    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
    assert_eq!(rep.container_id.as_deref(), Some("zip"));
    assert_eq!(rep.inspection, Inspection::Limited);
    assert!(rep.warnings.iter().any(|w| w.code == "budget.xml_depth_exceeded"));
}

// 2. wrong_namespace_prefixed
#[test]
fn test_regression_wrong_namespace_prefixed() {
    let tmp = std::env::temp_dir().join("test_regression_wrong_ns.docx");
    let xml = r#"<w:document xmlns:w="urn:evil"><w:body/></w:document>"#;
    create_docx_package(&tmp, xml);

    let rep = detect_path(&tmp, &DetectOptions::default()).unwrap();
    let _ = std::fs::remove_file(&tmp);

    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
    assert_eq!(rep.container_id.as_deref(), Some("zip"));
}

// 3. no_namespace_empty_root
#[test]
fn test_regression_no_namespace_empty_root() {
    let tmp = std::env::temp_dir().join("test_regression_empty_root.docx");
    let xml = r#"<document/>"#;
    create_docx_package(&tmp, xml);

    let rep = detect_path(&tmp, &DetectOptions::default()).unwrap();
    let _ = std::fs::remove_file(&tmp);

    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
}

// 4. invalid_multiple_roots
#[test]
fn test_regression_invalid_multiple_roots() {
    let tmp = std::env::temp_dir().join("test_regression_multi_roots.docx");
    let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body/></w:document><evil/>"#;
    create_docx_package(&tmp, xml);

    let rep = detect_path(&tmp, &DetectOptions::default()).unwrap();
    let _ = std::fs::remove_file(&tmp);

    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
}

// 5. cfb_forged_fib_missing_table
#[test]
fn test_regression_cfb_forged_fib_missing_table() {
    let tmp = std::env::temp_dir().join("test_regression_missing_table.cfb");
    // flags = 0x1000 (bit 9 = 0 -> 0Table required), table = false
    create_cfb_case(&tmp, false, 0x00C1, 0x1000, false);

    let rep = detect_path(&tmp, &DetectOptions::default()).unwrap();
    let _ = std::fs::remove_file(&tmp);

    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
    assert_eq!(rep.container_id.as_deref(), Some("cfb"));
    assert!(rep.warnings.iter().any(|w| w.code == "cfb.table_stream_missing"));
}

// 6. cfb_embedded_word_in_workbook
#[test]
fn test_regression_cfb_embedded_word_in_workbook() {
    let tmp = std::env::temp_dir().join("test_regression_embedded_word.cfb");
    // nested = true
    create_cfb_case(&tmp, true, 0x00C1, 0x1000, true);

    let rep = detect_path(&tmp, &DetectOptions::default()).unwrap();
    let _ = std::fs::remove_file(&tmp);

    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
    assert_eq!(rep.container_id.as_deref(), Some("cfb"));
}

// 7. cfb_unsupported_fib_version
#[test]
fn test_regression_cfb_unsupported_fib_version() {
    let tmp = std::env::temp_dir().join("test_regression_unsupported_fib.cfb");
    create_cfb_case(&tmp, false, 0xFFFF, 0x1000, true);

    let rep = detect_path(&tmp, &DetectOptions::default()).unwrap();
    let _ = std::fs::remove_file(&tmp);

    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
    assert_eq!(rep.container_id.as_deref(), Some("cfb"));
    assert!(rep.warnings.iter().any(|w| w.code == "cfb.unsupported_word_version"));
}

// 8. api_missing_extension
#[test]
fn test_regression_api_missing_extension() {
    let o = DetectOptions::default();
    let png = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR\0\0\0\x01\0\0\0\x01\x08\x02\0\0\0";
    let tmp = std::env::temp_dir().join("png_without_suffix");
    std::fs::write(&tmp, png).unwrap();

    let a = detect_path(&tmp, &o).unwrap();
    let b = detect_bytes(png, &o).unwrap();
    let c = detect_reader(&mut Cursor::new(png), &o).unwrap();
    let _ = std::fs::remove_file(&tmp);

    assert_eq!(a.extension_status, ExtensionStatus::Missing);
    assert_eq!(b.extension_status, ExtensionStatus::Missing);
    assert_eq!(c.extension_status, ExtensionStatus::Missing);
}

// 9. png_signature_only_evidence
#[test]
fn test_regression_png_signature_only_evidence() {
    let o = DetectOptions::default();
    let sig_only = b"\x89PNG\r\n\x1a\n";
    let r = detect_bytes(sig_only, &o).unwrap();

    assert_eq!(r.format_id.as_deref(), Some("png"));
    assert_eq!(r.verification, Verification::Signature);
    assert!(r.evidence.iter().any(|e| e.code == "sig.png.header"));
    assert!(!r.evidence.iter().any(|e| e.code == "sig.png.ihdr"));
}
