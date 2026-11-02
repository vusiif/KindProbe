use std::fs::{self, File};
use std::path::Path;
use kindprobe::budget::DetectOptions;
use kindprobe::{detect_bytes, detect_path, detect_reader};

#[test]
fn test_api_consistency() {
    let options = DetectOptions::default();

    for fixture_name in &[
        "valid_png.png",
        "valid_pdf.pdf",
        "plain_zip.zip",
        "true_docx.docx",
        "true_doc.doc",
        "fake_docx_zip.zip",
        "fake_doc_cfb.doc",
    ] {
        let p = Path::new("fixtures").join(fixture_name);
        let bytes = fs::read(&p).unwrap();

        // 1. detect_path
        let rep_path = detect_path(&p, &options).unwrap();

        // 2. detect_bytes
        let rep_bytes = detect_bytes(&bytes, &options).unwrap();

        // 3. detect_reader
        let mut file = File::open(&p).unwrap();
        let rep_reader = detect_reader(&mut file, &options).unwrap();

        // Ensure identification, format, container, inspection, verification match
        assert_eq!(rep_path.identification, rep_bytes.identification, "Mismatch for {}", fixture_name);
        assert_eq!(rep_path.format_id, rep_bytes.format_id, "Mismatch for {}", fixture_name);
        assert_eq!(rep_path.container_id, rep_bytes.container_id, "Mismatch for {}", fixture_name);
        assert_eq!(rep_path.verification, rep_bytes.verification, "Mismatch for {}", fixture_name);
        assert_eq!(rep_path.inspection, rep_bytes.inspection, "Mismatch for {}", fixture_name);

        assert_eq!(rep_bytes.identification, rep_reader.identification, "Mismatch for {}", fixture_name);
        assert_eq!(rep_bytes.format_id, rep_reader.format_id, "Mismatch for {}", fixture_name);
        assert_eq!(rep_bytes.container_id, rep_reader.container_id, "Mismatch for {}", fixture_name);
        assert_eq!(rep_bytes.verification, rep_reader.verification, "Mismatch for {}", fixture_name);
        assert_eq!(rep_bytes.inspection, rep_reader.inspection, "Mismatch for {}", fixture_name);
    }
}
