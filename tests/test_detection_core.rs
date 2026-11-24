//! Regression and differential tests for DetectionCore zero-allocation lazy materialization engine.

#![forbid(unsafe_code)]

use kindprobe::model::{DetectionCore, ExtensionStatus, Identification, Inspection, Verification};
use kindprobe::{compare_extension, compare_extension_core, detect_bytes, detect_bytes_core, DetectOptions};
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
    assert!(size <= 192, "DetectionCore size is {} bytes, should be <= 192 bytes", size);
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
    assert_eq!(core.warning.map(|(c, _)| c), Some("budget.source_read_exceeded"));
    assert!(rep.warnings.iter().any(|w| w.code == "budget.source_read_exceeded"));
    assert!(core.validate_invariants().is_ok());
}
