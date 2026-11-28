use kindprobe::{detect_bytes, DetectOptions, Identification, Inspection, Verification};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn create_ooxml_package(
    rel_target: &str,
    content_type: &str,
    main_part_path: &str,
    root_tag: &str,
    namespace: &str,
) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

        // 1. _rels/.rels
        zip.start_file("_rels/.rels", options).unwrap();
        let rels_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="{}"/>
</Relationships>"#,
            rel_target
        );
        zip.write_all(rels_content.as_bytes()).unwrap();

        // 2. [Content_Types].xml
        zip.start_file("[Content_Types].xml", options).unwrap();
        let ct_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Override PartName="/{}" ContentType="{}"/>
</Types>"#,
            main_part_path, content_type
        );
        zip.write_all(ct_content.as_bytes()).unwrap();

        // 3. Main part XML
        zip.start_file(main_part_path, options).unwrap();
        let main_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<{} xmlns="{}">
</{}>"#,
            root_tag, namespace, root_tag
        );
        zip.write_all(main_content.as_bytes()).unwrap();

        zip.finish().unwrap();
    }
    buf
}

#[test]
fn test_detect_xlsx_success() {
    let bytes = create_ooxml_package(
        "xl/workbook.xml",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml",
        "xl/workbook.xml",
        "workbook",
        "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
    );
    let options = DetectOptions::default();
    let report = detect_bytes(&bytes, &options).expect("detect xlsx");

    assert_eq!(report.identification, Identification::Format);
    assert_eq!(report.format_id.as_deref(), Some("xlsx"));
    assert_eq!(report.recommended_extension.as_deref(), Some("xlsx"));
    assert_eq!(report.category, "document");
    assert_eq!(report.container_id.as_deref(), Some("zip"));
    assert_eq!(report.verification, Verification::Structure);
    assert_eq!(report.inspection, Inspection::Complete);
    assert!(report
        .evidence
        .iter()
        .any(|e| e.code == "ooxml.spreadsheet_main_part"));
}

#[test]
fn test_detect_pptx_success() {
    let bytes = create_ooxml_package(
        "ppt/presentation.xml",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml",
        "ppt/presentation.xml",
        "presentation",
        "http://schemas.openxmlformats.org/presentationml/2006/main",
    );
    let options = DetectOptions::default();
    let report = detect_bytes(&bytes, &options).expect("detect pptx");

    assert_eq!(report.identification, Identification::Format);
    assert_eq!(report.format_id.as_deref(), Some("pptx"));
    assert_eq!(report.recommended_extension.as_deref(), Some("pptx"));
    assert_eq!(report.category, "document");
    assert_eq!(report.container_id.as_deref(), Some("zip"));
    assert_eq!(report.verification, Verification::Structure);
    assert_eq!(report.inspection, Inspection::Complete);
    assert!(report
        .evidence
        .iter()
        .any(|e| e.code == "ooxml.presentation_main_part"));
}

#[test]
fn test_detect_macro_variants() {
    let options = DetectOptions::default();

    // DOCM
    let docm_bytes = create_ooxml_package(
        "word/document.xml",
        "application/vnd.ms-word.document.macroEnabled.main+xml",
        "word/document.xml",
        "document",
        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
    );
    let rep_docm = detect_bytes(&docm_bytes, &options).expect("detect docm");
    assert_eq!(rep_docm.identification, Identification::Format);
    assert_eq!(rep_docm.format_id.as_deref(), Some("docm"));
    assert_eq!(rep_docm.recommended_extension.as_deref(), Some("docm"));

    // XLSM
    let xlsm_bytes = create_ooxml_package(
        "xl/workbook.xml",
        "application/vnd.ms-excel.sheet.macroEnabled.main+xml",
        "xl/workbook.xml",
        "workbook",
        "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
    );
    let rep_xlsm = detect_bytes(&xlsm_bytes, &options).expect("detect xlsm");
    assert_eq!(rep_xlsm.identification, Identification::Format);
    assert_eq!(rep_xlsm.format_id.as_deref(), Some("xlsm"));
    assert_eq!(rep_xlsm.recommended_extension.as_deref(), Some("xlsm"));

    // PPTM
    let pptm_bytes = create_ooxml_package(
        "ppt/presentation.xml",
        "application/vnd.ms-powerpoint.presentation.macroEnabled.main+xml",
        "ppt/presentation.xml",
        "presentation",
        "http://schemas.openxmlformats.org/presentationml/2006/main",
    );
    let rep_pptm = detect_bytes(&pptm_bytes, &options).expect("detect pptm");
    assert_eq!(rep_pptm.identification, Identification::Format);
    assert_eq!(rep_pptm.format_id.as_deref(), Some("pptm"));
    assert_eq!(rep_pptm.recommended_extension.as_deref(), Some("pptm"));
}

#[test]
fn test_detect_ooxml_adversarial_mismatched_root_and_namespace() {
    let options = DetectOptions::default();

    // Mismatched root element: XLSX content type but with <presentation> root tag
    let bad_root = create_ooxml_package(
        "xl/workbook.xml",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml",
        "xl/workbook.xml",
        "presentation",
        "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
    );
    let rep = detect_bytes(&bad_root, &options).expect("detect bad root");
    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
    assert!(rep
        .warnings
        .iter()
        .any(|w| w.code == "ooxml.invalid_root_element"));

    // Mismatched namespace: PPTX content type but with unknown fake namespace
    let bad_ns = create_ooxml_package(
        "ppt/presentation.xml",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml",
        "ppt/presentation.xml",
        "presentation",
        "http://fake.example.com/fake_ns",
    );
    let rep_ns = detect_bytes(&bad_ns, &options).expect("detect bad ns");
    assert_eq!(rep_ns.identification, Identification::ContainerOnly);
    assert_eq!(rep_ns.format_id, None);
    assert!(rep_ns
        .warnings
        .iter()
        .any(|w| w.code == "ooxml.invalid_root_element"));
}
