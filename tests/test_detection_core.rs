//! Regression and differential tests for DetectionCore zero-allocation lazy materialization engine.

#![forbid(unsafe_code)]

use kindprobe::model::{DetectionCore, ExtensionStatus, Identification, Inspection, Verification};
use kindprobe::{
    compare_extension, compare_extension_core, detect_bytes, detect_bytes_core, DetectOptions,
};
use std::ffi::OsStr;
use std::fs;

#[test]
fn test_detection_core_copy_and_compactness() {
    // Ensure DetectionCore implements Copy
    let core1 = DetectionCore::empty();
    let core2 = core1;
    assert_eq!(core1.category, core2.category);

    // Validate size of DetectionCore remains compact (fits easily in L1 cache, zero heap allocation)
    let size = std::mem::size_of::<DetectionCore>();
    assert!(
        size <= 192,
        "DetectionCore size is {} bytes, should be <= 192 bytes",
        size
    );
}

#[test]
fn test_detection_core_differential_equivalence() {
    let options = DetectOptions::default();

    // 1. Valid PNG
    let png_bytes = fs::read("fixtures/valid_png.png").expect("read valid_png.png");
    let core_png = detect_bytes_core(&png_bytes, &options).expect("detect core png");
    let rep_png = detect_bytes(&png_bytes, &options).expect("detect report png");

    assert_eq!(core_png.format_id, rep_png.format_id.as_deref());
    assert_eq!(core_png.category, rep_png.category.as_str());
    assert_eq!(core_png.container_id, rep_png.container_id.as_deref());
    assert_eq!(core_png.identification, rep_png.identification);
    assert_eq!(core_png.verification, rep_png.verification);
    assert_eq!(core_png.inspection, rep_png.inspection);
    assert!(core_png.validate_invariants().is_ok());

    // Materialize report and check equivalence
    let mat_png = core_png.to_report(Default::default(), ExtensionStatus::Undetermined);
    assert_eq!(mat_png.format_id, rep_png.format_id);
    assert_eq!(mat_png.category, rep_png.category);

    // 2. Valid PDF
    let pdf_bytes = fs::read("fixtures/valid_pdf.pdf").expect("read valid_pdf.pdf");
    let core_pdf = detect_bytes_core(&pdf_bytes, &options).expect("detect core pdf");
    let rep_pdf = detect_bytes(&pdf_bytes, &options).expect("detect report pdf");

    assert_eq!(core_pdf.format_id, rep_pdf.format_id.as_deref());
    assert_eq!(core_pdf.category, rep_pdf.category.as_str());
    assert_eq!(core_pdf.identification, rep_pdf.identification);
    assert_eq!(core_pdf.verification, rep_pdf.verification);
    assert!(core_pdf.validate_invariants().is_ok());

    // 3. Container ZIP & DOCX
    let docx_bytes = fs::read("fixtures/true_docx.docx").expect("read true_docx.docx");
    let core_docx = detect_bytes_core(&docx_bytes, &options).expect("detect core docx");
    let rep_docx = detect_bytes(&docx_bytes, &options).expect("detect report docx");

    assert_eq!(core_docx.format_id, rep_docx.format_id.as_deref());
    assert_eq!(core_docx.container_id, rep_docx.container_id.as_deref());
    assert_eq!(core_docx.identification, rep_docx.identification);
    assert!(core_docx.validate_invariants().is_ok());

    // 4. Structured JSON Text
    let json_bytes = br#"{"title": "test", "version": 1, "items": [1, 2, 3]}"#;
    let core_json = detect_bytes_core(json_bytes, &options).expect("detect core json");
    let rep_json = detect_bytes(json_bytes, &options).expect("detect report json");

    assert_eq!(core_json.format_id, rep_json.format_id.as_deref());
    assert_eq!(core_json.category, rep_json.category.as_str());
    assert_eq!(core_json.identification, rep_json.identification);
    assert!(core_json.validate_invariants().is_ok());

    // 5. Source code (Rust)
    let rust_bytes = b"fn main() {\n    println!(\"Hello, world!\");\n}\n";
    let core_rust = detect_bytes_core(rust_bytes, &options).expect("detect core rust");
    let rep_rust = detect_bytes(rust_bytes, &options).expect("detect report rust");

    assert_eq!(core_rust.format_id, rep_rust.format_id.as_deref());
    assert_eq!(core_rust.category, rep_rust.category.as_str());
    assert_eq!(core_rust.identification, rep_rust.identification);
    assert!(core_rust.validate_invariants().is_ok());

    // 6. Plain text
    let txt_bytes = b"This is just simple plain ASCII text without any code structures.";
    let core_txt = detect_bytes_core(txt_bytes, &options).expect("detect core txt");
    let rep_txt = detect_bytes(txt_bytes, &options).expect("detect report txt");

    assert_eq!(core_txt.format_id, rep_txt.format_id.as_deref());
    assert_eq!(core_txt.category, rep_txt.category.as_str());
    assert_eq!(core_txt.identification, rep_txt.identification);
    assert!(core_txt.validate_invariants().is_ok());

    // 7. Empty file
    let empty_bytes = b"";
    let core_empty = detect_bytes_core(empty_bytes, &options).expect("detect core empty");
    let rep_empty = detect_bytes(empty_bytes, &options).expect("detect report empty");

    assert_eq!(core_empty.format_id, rep_empty.format_id.as_deref());
    assert_eq!(core_empty.category, rep_empty.category.as_str());
    assert_eq!(core_empty.identification, rep_empty.identification);
    assert!(core_empty.validate_invariants().is_ok());

    // 8. Random garbage binary
    let garbage_bytes = &[0xFE, 0xFD, 0xFC, 0xFB, 0xFA, 0xF9, 0xF8, 0xF7];
    let core_garbage = detect_bytes_core(garbage_bytes, &options).expect("detect core garbage");
    let rep_garbage = detect_bytes(garbage_bytes, &options).expect("detect report garbage");

    assert_eq!(core_garbage.format_id, rep_garbage.format_id.as_deref());
    assert_eq!(core_garbage.category, rep_garbage.category.as_str());
    assert_eq!(core_garbage.identification, rep_garbage.identification);
    assert!(core_garbage.validate_invariants().is_ok());
}

#[test]
fn test_compare_extension_core_parity() {
    let options = DetectOptions::default();
    let png_bytes = fs::read("fixtures/valid_png.png").expect("read valid_png.png");
    let core_png = detect_bytes_core(&png_bytes, &options).expect("detect core png");
    let rep_png = detect_bytes(&png_bytes, &options).expect("detect report png");

    // Match
    let st_core_match = compare_extension_core(&core_png, Some(OsStr::new("png")));
    let st_rep_match = compare_extension(&rep_png, Some(OsStr::new("png")));
    assert_eq!(st_core_match, ExtensionStatus::Match);
    assert_eq!(st_core_match, st_rep_match);

    // Mismatch (registered extension "jpg" on png file)
    let st_core_mismatch = compare_extension_core(&core_png, Some(OsStr::new("jpg")));
    let st_rep_mismatch = compare_extension(&rep_png, Some(OsStr::new("jpg")));
    assert_eq!(st_core_mismatch, ExtensionStatus::Mismatch);
    assert_eq!(st_core_mismatch, st_rep_mismatch);

    // Missing (no extension provided)
    let st_core_none = compare_extension_core(&core_png, None);
    let st_rep_none = compare_extension(&rep_png, None);
    assert_eq!(st_core_none, ExtensionStatus::Missing);
    assert_eq!(st_core_none, st_rep_none);

    // Undetermined (format is unknown, but extension is provided)
    let garbage_bytes = &[0xFE, 0xFD, 0xFC, 0xFB, 0xFA, 0xF9, 0xF8, 0xF7];
    let core_garbage = detect_bytes_core(garbage_bytes, &options).expect("detect core garbage");
    let rep_garbage = detect_bytes(garbage_bytes, &options).expect("detect report garbage");
    let st_core_undet = compare_extension_core(&core_garbage, Some(OsStr::new("bin")));
    let st_rep_undet = compare_extension(&rep_garbage, Some(OsStr::new("bin")));
    assert_eq!(st_core_undet, ExtensionStatus::Undetermined);
    assert_eq!(st_core_undet, st_rep_undet);

    // Unrecognized (unregistered extension)
    let st_core_unrec = compare_extension_core(&core_png, Some(OsStr::new("xyznotexist999")));
    let st_rep_unrec = compare_extension(&rep_png, Some(OsStr::new("xyznotexist999")));
    assert_eq!(st_core_unrec, ExtensionStatus::Unrecognized);
    assert_eq!(st_core_unrec, st_rep_unrec);
}

#[test]
fn test_budget_exceeded_core_parity() {
    let options = DetectOptions {
        initial_window_size: 4096,
        max_source_bytes_read: 16,
        ..Default::default()
    };

    let png_bytes = fs::read("fixtures/valid_png.png").expect("read valid_png.png");
    let core = detect_bytes_core(&png_bytes, &options).expect("detect core with budget");
    let rep = detect_bytes(&png_bytes, &options).expect("detect report with budget");

    assert_eq!(core.identification, Identification::Unknown);
    assert_eq!(core.inspection, Inspection::Limited);
    assert_eq!(core.verification, Verification::None);
    assert_eq!(rep.inspection, Inspection::Limited);
    assert_eq!(
        core.warning.map(|(c, _)| c),
        Some("budget.source_read_exceeded")
    );
    assert!(rep
        .warnings
        .iter()
        .any(|w| w.code == "budget.source_read_exceeded"));
    assert!(core.validate_invariants().is_ok());
}

#[test]
fn test_detect_path_and_reader_core_container_and_extension_parity() {
    use kindprobe::{detect_path_core, detect_reader_core};
    use std::io::Cursor;
    use std::path::Path;

    let options = DetectOptions::default();

    // 1. DOCX via detect_path_core + compare_extension_core
    let docx_core =
        detect_path_core(Path::new("fixtures/true_docx.docx"), &options).expect("docx path core");
    assert_eq!(docx_core.format_id, Some("docx"));
    assert_eq!(docx_core.container_id, Some("zip"));
    assert_eq!(
        compare_extension_core(&docx_core, Some(OsStr::new("docx"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension_core(&docx_core, Some(OsStr::new("pdf"))),
        ExtensionStatus::Mismatch
    );

    // 2. PNG via detect_path_core + detect_reader_core
    let png_path_core =
        detect_path_core(Path::new("fixtures/valid_png.png"), &options).expect("png path core");
    let png_bytes = fs::read("fixtures/valid_png.png").unwrap();
    let png_reader_core =
        detect_reader_core(&mut Cursor::new(&png_bytes), &options).expect("png reader core");
    assert_eq!(png_path_core, png_reader_core);
    assert_eq!(
        compare_extension_core(&png_path_core, Some(OsStr::new("png"))),
        ExtensionStatus::Match
    );

    // 3. Large (> 4 KiB) non-container text buffer fast slice evaluation
    let mut big_rust = String::from("use std::io;\nfn main() {\n    println!(\"hi\");\n}\n");
    big_rust.push_str(&"// comment padding\n".repeat(300));
    assert!(big_rust.len() > 4096);
    let big_core = detect_bytes_core(big_rust.as_bytes(), &options).expect("big rust core");
    let big_rep = detect_bytes(big_rust.as_bytes(), &options).expect("big rust rep");
    assert_eq!(big_core.format_id, Some("rs"));
    assert_eq!(big_rep.format_id.as_deref(), Some("rs"));
}

#[test]
fn test_dual_path_edge_cases_and_overflow_safety() {
    let options = DetectOptions::default();

    // 1. Exact 4-byte Zstd magic parity (must NOT falsely report Truncated in detect_bytes)
    let zstd_4b = [0x28, 0xB5, 0x2F, 0xFD];
    let zstd_core = detect_bytes_core(&zstd_4b, &options).expect("zstd 4b core");
    let zstd_rep = detect_bytes(&zstd_4b, &options).expect("zstd 4b rep");
    assert_eq!(zstd_core.format_id, Some("zstd"));
    assert_eq!(zstd_rep.format_id.as_deref(), Some("zstd"));
    assert_eq!(zstd_core.verification, zstd_rep.verification);

    // 2. RAR5 parity (Structure + sig.rar.header + sig.rar5.signature)
    let rar5 = b"Rar!\x1A\x07\x01\x00\x00\x00\x00\x00";
    let rar5_core = detect_bytes_core(rar5, &options).expect("rar5 core");
    let rar5_rep = detect_bytes(rar5, &options).expect("rar5 rep");
    assert_eq!(rar5_core.verification, Verification::Structure);
    assert_eq!(rar5_core.verification, rar5_rep.verification);
    assert_eq!(rar5_core.primary_evidence, Some("sig.rar.header"));
    assert_eq!(rar5_core.secondary_evidence, Some("sig.rar5.signature"));

    // 3. SQLite3 accepted_extensions includes db3 across both paths
    let sqlite = b"SQLite format 3\x00\x10\x00\x01\x01\x00\x40\x20\x20";
    let sql_core = detect_bytes_core(sqlite, &options).expect("sqlite core");
    assert_eq!(
        compare_extension_core(&sql_core, Some(OsStr::new("db3"))),
        ExtensionStatus::Match
    );

    // 4. MP4 with >= 16MB box_size (b0 != 0x00) dual-path parity
    let mp4_big_box = b"\x01\x00\x00\x00ftypisom\x00\x00\x02\x00isom";
    let mp4_core = detect_bytes_core(mp4_big_box, &options).expect("mp4 core");
    let mp4_rep = detect_bytes(mp4_big_box, &options).expect("mp4 rep");
    assert_eq!(mp4_core.format_id, Some("mp4"));
    assert_eq!(mp4_rep.format_id.as_deref(), Some("mp4"));

    // 5. Malicious WAV with chunk_size = u32::MAX must not overflow or panic
    let mut bad_wav = Vec::from(&b"RIFF\x24\x00\x00\x00WAVE"[..]);
    bad_wav.extend_from_slice(b"JUNK");
    bad_wav.extend_from_slice(&u32::MAX.to_le_bytes());
    bad_wav.extend_from_slice(&[0u8; 16]);
    let wav_core = detect_bytes_core(&bad_wav, &options).expect("malformed wav core");
    let wav_rep = detect_bytes(&bad_wav, &options).expect("malformed wav rep");
    assert_eq!(wav_core.format_id, Some("wav"));
    assert_eq!(wav_rep.format_id.as_deref(), Some("wav"));
    assert_eq!(wav_core.verification, Verification::Signature);
    assert_eq!(wav_rep.verification, Verification::Signature);
}
