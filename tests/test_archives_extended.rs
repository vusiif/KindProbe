use kindprobe::{detect_bytes, DetectOptions, Identification, Inspection, Verification};

fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

#[test]
fn test_detect_7z_with_start_header() {
    let mut sevenz = Vec::new();
    // 0..6: Signature
    sevenz.extend_from_slice(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]);
    // 6..8: Version (0.4)
    sevenz.extend_from_slice(&[0x00, 0x04]);

    // 12..32: 20 bytes of dummy StartHeader fields
    let next_header_data = [0x01u8; 20];
    let crc = crc32_ieee(&next_header_data);

    // 8..12: StartHeaderCRC
    sevenz.extend_from_slice(&crc.to_le_bytes());
    sevenz.extend_from_slice(&next_header_data);

    let options = DetectOptions::default();
    let report = detect_bytes(&sevenz, &options).expect("detect 7z");

    assert_eq!(report.identification, Identification::Format);
    assert_eq!(report.format_id.as_deref(), Some("7z"));
    assert_eq!(report.recommended_extension.as_deref(), Some("7z"));
    assert_eq!(report.category, "archive");
    assert_eq!(report.container_id.as_deref(), Some("7z"));
    assert_eq!(report.verification, Verification::Structure);
    assert_eq!(report.inspection, Inspection::Complete);
    assert!(report.evidence.iter().any(|e| e.code == "sig.7z.header"));
    assert!(report
        .evidence
        .iter()
        .any(|e| e.code == "sig.7z.start_header"));
}

#[test]
fn test_detect_7z_signature_only() {
    // 6 bytes signature without valid 32-byte header
    let sevenz = [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C, 0x00, 0x04];
    let options = DetectOptions::default();
    let report = detect_bytes(&sevenz, &options).expect("detect 7z sig only");

    assert_eq!(report.identification, Identification::Format);
    assert_eq!(report.format_id.as_deref(), Some("7z"));
    assert_eq!(report.verification, Verification::Signature);
    assert!(report.evidence.iter().any(|e| e.code == "sig.7z.header"));
    assert!(!report
        .evidence
        .iter()
        .any(|e| e.code == "sig.7z.start_header"));
}

#[test]
fn test_detect_7z_truncated() {
    let truncated = [0x37, 0x7A, 0xBC];
    let options = DetectOptions::default();
    let report = detect_bytes(&truncated, &options).expect("detect truncated 7z");

    assert_eq!(report.identification, Identification::Unknown);
    assert_eq!(report.format_id, None);
    assert!(report.warnings.iter().any(|w| w.code == "input.truncated"));
    assert_eq!(report.candidates.as_deref(), Some(&["7z".to_string()][..]));
}

#[test]
fn test_detect_rar4_with_main_header() {
    // 0..7: Rar!\x1A\x07\x00, 7..9: CRC, 9: 0x73 (HEAD_TYPE = MAIN_HEAD)
    let mut rar4 = Vec::new();
    rar4.extend_from_slice(b"Rar!\x1A\x07\x00");
    rar4.extend_from_slice(&[0x12, 0x34]); // head_crc
    rar4.push(0x73); // head_type = 0x73 (Main Archive Header)
    rar4.extend_from_slice(&[0x00; 10]);

    let options = DetectOptions::default();
    let report = detect_bytes(&rar4, &options).expect("detect rar4");

    assert_eq!(report.identification, Identification::Format);
    assert_eq!(report.format_id.as_deref(), Some("rar"));
    assert_eq!(report.recommended_extension.as_deref(), Some("rar"));
    assert_eq!(report.category, "archive");
    assert_eq!(report.container_id.as_deref(), Some("rar"));
    assert_eq!(report.verification, Verification::Structure);
    assert_eq!(report.inspection, Inspection::Complete);
    assert!(report.evidence.iter().any(|e| e.code == "sig.rar.header"));
    assert!(report
        .evidence
        .iter()
        .any(|e| e.code == "sig.rar.main_header"));
}

#[test]
fn test_detect_rar5_signature() {
    // Rar!\x1A\x07\x01\x00 (8 bytes)
    let mut rar5 = Vec::new();
    rar5.extend_from_slice(b"Rar!\x1A\x07\x01\x00");
    rar5.extend_from_slice(&[0x00; 8]);

    let options = DetectOptions::default();
    let report = detect_bytes(&rar5, &options).expect("detect rar5");

    assert_eq!(report.identification, Identification::Format);
    assert_eq!(report.format_id.as_deref(), Some("rar"));
    assert_eq!(report.verification, Verification::Structure);
    assert!(report.evidence.iter().any(|e| e.code == "sig.rar.header"));
    assert!(report
        .evidence
        .iter()
        .any(|e| e.code == "sig.rar5.signature"));
}

#[test]
fn test_detect_rar_truncated() {
    let truncated = b"Rar!";
    let options = DetectOptions::default();
    let report = detect_bytes(truncated, &options).expect("detect truncated rar");

    assert_eq!(report.identification, Identification::Unknown);
    assert_eq!(report.format_id, None);
    assert!(report.warnings.iter().any(|w| w.code == "input.truncated"));
    assert_eq!(report.candidates.as_deref(), Some(&["rar".to_string()][..]));
}
