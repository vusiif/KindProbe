use std::fs;
use std::path::Path;
use kindprobe::budget::DetectOptions;
use kindprobe::{detect_bytes, detect_path, compare_extension, ExtensionStatus, Identification};

#[test]
fn test_extension_independence() {
    let options = DetectOptions::default();

    // 1. PNG renamed as .pdf
    let png_path = Path::new("fixtures/mismatched_png.pdf");
    let rep_png = detect_path(png_path, &options).expect("detect png");
    assert_eq!(rep_png.identification, Identification::Format);
    assert_eq!(rep_png.format_id.as_deref(), Some("png"));
    assert_eq!(rep_png.extension_status, ExtensionStatus::Mismatch);

    // 2. DOCX renamed as .zip
    let docx_path = Path::new("fixtures/mismatched_docx.zip");
    let rep_docx = detect_path(docx_path, &options).expect("detect docx");
    assert_eq!(rep_docx.identification, Identification::Format);
    assert_eq!(rep_docx.format_id.as_deref(), Some("docx"));
    assert_eq!(rep_docx.extension_status, ExtensionStatus::Mismatch);

    // 3. DOC renamed as .docx
    let doc_path = Path::new("fixtures/mismatched_doc.docx");
    let rep_doc = detect_path(doc_path, &options).expect("detect doc");
    assert_eq!(rep_doc.identification, Identification::Format);
    assert_eq!(rep_doc.format_id.as_deref(), Some("doc"));
    assert_eq!(rep_doc.extension_status, ExtensionStatus::Mismatch);
}

#[test]
fn test_compare_extension_pure_function() {
    let options = DetectOptions::default();
    let bytes = fs::read("fixtures/valid_png.png").expect("read png");
    let rep = detect_bytes(&bytes, &options).expect("detect");

    use std::ffi::OsStr;
    // Matching
    assert_eq!(compare_extension(&rep, Some(OsStr::new("png"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("PNG"))), ExtensionStatus::Match);

    // Mismatched known
    assert_eq!(compare_extension(&rep, Some(OsStr::new("pdf"))), ExtensionStatus::Mismatch);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("docx"))), ExtensionStatus::Mismatch);

    // Missing
    assert_eq!(compare_extension(&rep, None), ExtensionStatus::Missing);
    assert_eq!(compare_extension(&rep, Some(OsStr::new(""))), ExtensionStatus::Missing);

    // Unrecognized
    assert_eq!(compare_extension(&rep, Some(OsStr::new("unknownextxyz"))), ExtensionStatus::Unrecognized);

    // Non-UTF-8 extension on confirmed format (Priority 5)
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let non_utf8_ext = OsStr::from_bytes(b"\xff\xfe");
        assert_eq!(compare_extension(&rep, Some(non_utf8_ext)), ExtensionStatus::Unrecognized);

        // Non-UTF-8 extension on container-only report (Priority 2 before non-UTF-8 check -> Undetermined)
        let mut container_rep = rep.clone();
        container_rep.identification = Identification::ContainerOnly;
        container_rep.format_id = None;
        container_rep.accepted_extensions.clear();
        assert_eq!(compare_extension(&container_rep, Some(non_utf8_ext)), ExtensionStatus::Undetermined);

        // Non-UTF-8 extension on unknown report (Priority 2 -> Undetermined)
        let mut unknown_rep = rep.clone();
        unknown_rep.identification = Identification::Unknown;
        unknown_rep.format_id = None;
        unknown_rep.accepted_extensions.clear();
        assert_eq!(compare_extension(&unknown_rep, Some(non_utf8_ext)), ExtensionStatus::Undetermined);
    }
}
