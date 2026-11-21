use kindprobe::{detect_bytes, DetectOptions, Identification, Inspection, Verification};

#[test]
fn test_detect_jpeg_with_segment() {
    // Standard JPEG: SOI (\xFF\xD8\xFF), APP0 marker (\xE0), length (16), "JFIF\0" ...
    let mut jpeg_bytes = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
    jpeg_bytes.extend_from_slice(b"JFIF\0\x01\x01\x00\x00\x01\x00\x01\x00\x00");
    jpeg_bytes.extend_from_slice(&[0xFF, 0xD9]); // EOI

    let options = DetectOptions::default();
    let report = detect_bytes(&jpeg_bytes, &options).expect("detect jpeg");

    assert_eq!(report.identification, Identification::Format);
    assert_eq!(report.format_id.as_deref(), Some("jpeg"));
    assert_eq!(report.recommended_extension.as_deref(), Some("jpg"));
    assert_eq!(report.category, "image");
    assert_eq!(report.container_id, None);
    assert_eq!(report.verification, Verification::Structure);
    assert_eq!(report.inspection, Inspection::Complete);
    assert!(report.evidence.iter().any(|e| e.code == "sig.jpeg.soi"));
    assert!(report.evidence.iter().any(|e| e.code == "sig.jpeg.segment"));
}

#[test]
fn test_detect_jpeg_signature_only() {
    // Just SOI + 0xFF, without recognized marker segment length
    let jpeg_bytes = [0xFF, 0xD8, 0xFF, 0x00];
    let options = DetectOptions::default();
    let report = detect_bytes(&jpeg_bytes, &options).expect("detect jpeg sig only");

    assert_eq!(report.identification, Identification::Format);
    assert_eq!(report.format_id.as_deref(), Some("jpeg"));
    assert_eq!(report.verification, Verification::Signature);
    assert_eq!(report.inspection, Inspection::Complete);
    assert!(report.evidence.iter().any(|e| e.code == "sig.jpeg.soi"));
    assert!(!report.evidence.iter().any(|e| e.code == "sig.jpeg.segment"));
}

#[test]
fn test_detect_jpeg_truncated() {
    let truncated_bytes = [0xFF, 0xD8];
    let options = DetectOptions::default();
    let report = detect_bytes(&truncated_bytes, &options).expect("detect truncated jpeg");

    assert_eq!(report.identification, Identification::Unknown);
    assert_eq!(report.format_id, None);
    assert!(report.warnings.iter().any(|w| w.code == "input.truncated"));
    assert_eq!(report.candidates.as_deref(), Some(&["jpeg".to_string()][..]));
}

#[test]
fn test_detect_gif_with_screen_descriptor() {
    // Valid GIF89a: 6 bytes header + 7 bytes screen descriptor (width: 100, height: 200)
    let mut gif_bytes = Vec::new();
    gif_bytes.extend_from_slice(b"GIF89a");
    gif_bytes.extend_from_slice(&100u16.to_le_bytes()); // width
    gif_bytes.extend_from_slice(&200u16.to_le_bytes()); // height
    gif_bytes.extend_from_slice(&[0x80, 0x00, 0x00]);   // packed, bg, aspect
    gif_bytes.push(0x3B);                               // GIF trailer

    let options = DetectOptions::default();
    let report = detect_bytes(&gif_bytes, &options).expect("detect gif");

    assert_eq!(report.identification, Identification::Format);
    assert_eq!(report.format_id.as_deref(), Some("gif"));
    assert_eq!(report.recommended_extension.as_deref(), Some("gif"));
    assert_eq!(report.category, "image");
    assert_eq!(report.container_id, None);
    assert_eq!(report.verification, Verification::Structure);
    assert_eq!(report.inspection, Inspection::Complete);
    assert!(report.evidence.iter().any(|e| e.code == "sig.gif.header"));
    assert!(report.evidence.iter().any(|e| e.code == "sig.gif.logical_screen"));
}

#[test]
fn test_detect_gif_zero_dimensions_downgrades_to_signature() {
    // GIF87a with 0 width/height -> valid header signature, but invalid screen descriptor
    let mut gif_bytes = Vec::new();
    gif_bytes.extend_from_slice(b"GIF87a");
    gif_bytes.extend_from_slice(&0u16.to_le_bytes()); // width = 0
    gif_bytes.extend_from_slice(&0u16.to_le_bytes()); // height = 0

    let options = DetectOptions::default();
    let report = detect_bytes(&gif_bytes, &options).expect("detect gif zero dimensions");

    assert_eq!(report.identification, Identification::Format);
    assert_eq!(report.format_id.as_deref(), Some("gif"));
    assert_eq!(report.verification, Verification::Signature);
    assert!(report.evidence.iter().any(|e| e.code == "sig.gif.header"));
    assert!(!report.evidence.iter().any(|e| e.code == "sig.gif.logical_screen"));
}

#[test]
fn test_detect_gif_truncated() {
    let truncated_bytes = b"GIF8";
    let options = DetectOptions::default();
    let report = detect_bytes(truncated_bytes, &options).expect("detect truncated gif");

    assert_eq!(report.identification, Identification::Unknown);
    assert_eq!(report.format_id, None);
    assert!(report.warnings.iter().any(|w| w.code == "input.truncated"));
    assert_eq!(report.candidates.as_deref(), Some(&["gif".to_string()][..]));
}
