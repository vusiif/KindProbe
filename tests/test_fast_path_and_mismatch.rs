use std::fs;
use std::path::Path;
use kindprobe::budget::DetectOptions;
use kindprobe::{detect_bytes, detect_path, ExtensionStatus, Identification};

#[test]
fn test_fast_path_optimistic_hits_on_real_fixtures() {
    let opts = DetectOptions::default();

    let cases = [
        ("fixtures/valid_png.png", "png"),
        ("fixtures/valid_pdf.pdf", "pdf"),
        ("fixtures/plain_zip.zip", "zip"),
        ("fixtures/true_docx.docx", "docx"),
        ("fixtures/true_doc.doc", "doc"),
    ];

    for (file_path, expected_fid) in cases {
        let p = Path::new(file_path);
        let rep = detect_path(p, &opts).expect("detect_path should succeed");
        assert_eq!(rep.identification, Identification::Format);
        assert_eq!(rep.format_id.as_deref(), Some(expected_fid));
        assert_eq!(rep.extension_status, ExtensionStatus::Match);

        // Also test detect_bytes with explicit extension_hint
        let bytes = fs::read(p).expect("read fixture bytes");
        let hint_opts = DetectOptions {
            extension_hint: Some(expected_fid.to_string()),
            ..DetectOptions::default()
        };
        let rep_hint = detect_bytes(&bytes, &hint_opts).expect("detect_bytes with hint should succeed");
        assert_eq!(rep_hint.identification, Identification::Format);
        assert_eq!(rep_hint.format_id.as_deref(), Some(expected_fid));
    }
}

#[test]
fn test_fast_path_declarative_rules_and_formats() {
    // 1. DuckDB (magic "DUCK" at offset 8, min_len 12)
    let mut duckdb_buf = vec![0u8; 1024];
    duckdb_buf[8..12].copy_from_slice(b"DUCK");
    let opts_duck = DetectOptions {
        extension_hint: Some("duckdb".to_string()),
        ..DetectOptions::default()
    };
    let rep_duck = detect_bytes(&duckdb_buf, &opts_duck).unwrap();
    assert_eq!(rep_duck.format_id.as_deref(), Some("duckdb"));

    // 2. FLAC
    let mut flac_buf = vec![0u8; 128];
    flac_buf[..4].copy_from_slice(b"fLaC");
    flac_buf[4] = 0x00;
    let opts_flac = DetectOptions {
        extension_hint: Some("flac".to_string()),
        ..DetectOptions::default()
    };
    let rep_flac = detect_bytes(&flac_buf, &opts_flac).unwrap();
    assert_eq!(rep_flac.format_id.as_deref(), Some("flac"));

    // 3. JSON
    let json_bytes = b"{\"kind\": \"probe\", \"version\": 1}";
    let opts_json = DetectOptions {
        extension_hint: Some("json".to_string()),
        ..DetectOptions::default()
    };
    let rep_json = detect_bytes(json_bytes, &opts_json).unwrap();
    assert_eq!(rep_json.format_id.as_deref(), Some("json"));

    // 4. ELF
    let mut elf_buf = vec![0u8; 128];
    elf_buf[..4].copy_from_slice(b"\x7fELF");
    elf_buf[4] = 2; // 64-bit
    elf_buf[5] = 1; // Little endian
    elf_buf[6] = 1; // Version 1
    let opts_elf = DetectOptions {
        extension_hint: Some("so".to_string()),
        ..DetectOptions::default()
    };
    let rep_elf = detect_bytes(&elf_buf, &opts_elf).unwrap();
    assert_eq!(rep_elf.format_id.as_deref(), Some("elf"));
}

#[test]
fn test_fast_path_adversarial_mismatch_fallback() {
    let opts = DetectOptions::default();

    // 1. fixtures/mismatched_png.pdf (PNG content with .pdf extension)
    let p_png_as_pdf = Path::new("fixtures/mismatched_png.pdf");
    let rep1 = detect_path(p_png_as_pdf, &opts).expect("detect_path should succeed");
    assert_eq!(rep1.format_id.as_deref(), Some("png"), "Must detect real format as png");
    assert_eq!(rep1.extension_status, ExtensionStatus::Mismatch, "Must report extension mismatch");

    // 2. fixtures/mismatched_doc.docx (DOC CFB content with .docx extension)
    let p_doc_as_docx = Path::new("fixtures/mismatched_doc.docx");
    let rep2 = detect_path(p_doc_as_docx, &opts).expect("detect_path should succeed");
    assert_eq!(rep2.format_id.as_deref(), Some("doc"), "Must detect real format as doc");
    assert_eq!(rep2.extension_status, ExtensionStatus::Mismatch, "Must report extension mismatch");

    // 3. PE executable disguised with .pdf hint
    let mut pe_buf = vec![0u8; 512];
    pe_buf[..2].copy_from_slice(b"MZ");
    pe_buf[0x3c..0x40].copy_from_slice(&0x80u32.to_le_bytes());
    pe_buf[0x80..0x84].copy_from_slice(b"PE\0\0");
    let opts_fake_pdf = DetectOptions {
        extension_hint: Some("pdf".to_string()),
        ..DetectOptions::default()
    };
    let rep3 = detect_bytes(&pe_buf, &opts_fake_pdf).unwrap();
    assert_eq!(rep3.format_id.as_deref(), Some("pe"), "Must fall back and detect true PE format");

    // 4. Plain ZIP disguised with .docx hint
    let plain_zip = fs::read("fixtures/plain_zip.zip").unwrap();
    let opts_fake_docx = DetectOptions {
        extension_hint: Some("docx".to_string()),
        ..DetectOptions::default()
    };
    let rep4 = detect_bytes(&plain_zip, &opts_fake_docx).unwrap();
    assert_eq!(rep4.format_id.as_deref(), Some("zip"), "Must fall back to container zip");

    // 5. Fake DOCX (missing [Content_Types].xml) disguised with .docx hint
    let fake_docx = fs::read("fixtures/fake_docx_zip.zip").unwrap();
    let rep5 = detect_bytes(&fake_docx, &opts_fake_docx).unwrap();
    assert_eq!(rep5.identification, Identification::ContainerOnly);
    assert_eq!(rep5.container_id.as_deref(), Some("zip"));
}

#[test]
fn test_fast_path_truncated_and_unrecognized_fallback() {
    let opts = DetectOptions::default();

    // 1. Truncated PNG
    let p_trunc = Path::new("fixtures/truncated_png.png");
    let rep_trunc = detect_path(p_trunc, &opts).unwrap();
    assert!(rep_trunc.warnings.iter().any(|w| w.code == "input.truncated"));

    // 2. Unrecognized extension with authentic content
    let png_bytes = fs::read("fixtures/valid_png.png").unwrap();
    let opts_unrec = DetectOptions {
        extension_hint: Some("unknown_custom_ext_xyz".to_string()),
        ..DetectOptions::default()
    };
    let rep_unrec = detect_bytes(&png_bytes, &opts_unrec).unwrap();
    assert_eq!(rep_unrec.format_id.as_deref(), Some("png"));

    // 3. None extension hint
    let opts_none = DetectOptions {
        extension_hint: None,
        ..DetectOptions::default()
    };
    let rep_none = detect_bytes(&png_bytes, &opts_none).unwrap();
    assert_eq!(rep_none.format_id.as_deref(), Some("png"));
}
