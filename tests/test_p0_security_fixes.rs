use kindprobe::budget::DetectOptions;
use kindprobe::{
    detect_bytes, detect_path, detect_reader, ExtensionStatus, Identification, Inspection,
};
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use std::path::Path;

/// P0.1 Test: Verify that a multi-byte UTF-8 character crossing byte 4096 never causes a panic.
#[test]
fn test_p0_1_utf8_char_boundary_slicing_no_panic() {
    let options = DetectOptions::default();

    // 1. Rust syntax at the top of the file
    let mut text =
        String::from("use std::collections::HashMap;\npub fn verify_boundary() -> bool { true }\n");

    // 2. Pad comments up to exactly byte 4095
    while text.len() < 4095 {
        text.push_str("// Padding comment line to reach exact boundary offset\n");
    }
    text.truncate(4095);
    assert_eq!(text.len(), 4095);

    // 3. Append 3-byte Chinese character "测" (bytes: [0xe6, 0xb5, 0x8b]) crossing boundary 4096
    text.push_str("测试中文字符边界安全\n");

    // Must not panic, and must safely detect as "rs"
    let rep =
        detect_bytes(text.as_bytes(), &options).expect("Detection must succeed without panic");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("rs"));
}

/// A reader wrapper that returns at most `chunk_size` bytes per `read()` call,
/// simulating fragmented TCP sockets or pipe reads.
struct ChunkedReader<R> {
    inner: R,
    chunk_size: usize,
}

impl<R: Read> Read for ChunkedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let max_to_read = buf.len().min(self.chunk_size);
        self.inner.read(&mut buf[..max_to_read])
    }
}

impl<R: Seek> Seek for ChunkedReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

/// P0.2 Test: Verify that short reads (e.g. 3 bytes per chunk) correctly fill the header window
/// without falsely reporting truncated signatures or incomplete inputs.
#[test]
fn test_p0_2_short_read_stream_handling() {
    let options = DetectOptions::default();

    // Real PNG fixture
    let raw = std::fs::read("fixtures/valid_png.png").expect("Read fixture");
    let mut chunked = ChunkedReader {
        inner: Cursor::new(raw),
        chunk_size: 3, // Very small chunk, returning 3 bytes per read
    };

    let rep = detect_reader(&mut chunked, &options).expect("Detection must succeed");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("png"));
    assert_eq!(rep.verification, kindprobe::Verification::Structure);
    assert!(!rep.warnings.iter().any(|w| w.code == "input.truncated"));
}

/// P0.3 Test: Verify budget consistency between detect_bytes and detect_reader.
#[test]
fn test_p0_3_slice_budget_consistency() {
    let raw = std::fs::read("fixtures/valid_png.png").expect("Read fixture");

    // Case A: max_source_bytes_read = 0 on non-empty slice must trigger budget.source_read_exceeded
    let zero_budget_opts = DetectOptions {
        max_source_bytes_read: 0,
        ..Default::default()
    };

    let rep_bytes_zero = detect_bytes(&raw, &zero_budget_opts).expect("Must return report");
    assert_eq!(rep_bytes_zero.identification, Identification::Unknown);
    assert_eq!(rep_bytes_zero.inspection, Inspection::Limited);
    assert!(rep_bytes_zero
        .warnings
        .iter()
        .any(|w| w.code == "budget.source_read_exceeded"));

    let mut cursor = Cursor::new(&raw);
    let rep_reader_zero =
        detect_reader(&mut cursor, &zero_budget_opts).expect("Must return report");
    assert_eq!(rep_reader_zero.identification, Identification::Unknown);
    assert_eq!(rep_reader_zero.inspection, Inspection::Limited);
    assert!(rep_reader_zero
        .warnings
        .iter()
        .any(|w| w.code == "budget.source_read_exceeded"));

    // Case B: max_source_bytes_read = 4 on a file whose window size is 4096
    let small_budget_opts = DetectOptions {
        max_source_bytes_read: 4,
        ..Default::default()
    };

    let rep_bytes_small = detect_bytes(&raw, &small_budget_opts).expect("Must return report");
    assert_eq!(rep_bytes_small.inspection, Inspection::Limited);
    assert!(rep_bytes_small
        .warnings
        .iter()
        .any(|w| w.code == "budget.source_read_exceeded"));

    let mut cursor2 = Cursor::new(&raw);
    let rep_reader_small =
        detect_reader(&mut cursor2, &small_budget_opts).expect("Must return report");
    assert_eq!(rep_reader_small.inspection, Inspection::Limited);
    assert!(rep_reader_small
        .warnings
        .iter()
        .any(|w| w.code == "budget.source_read_exceeded"));
}

/// P0.4 Test: Verify that an incorrect extension hint on a container format (e.g. .xlsx hint on .docx file)
/// successfully detects the true container format without duplicate parsing or double I/O.
#[test]
fn test_p0_4_mismatched_container_hint_no_double_parse() {
    let mismatched_options = DetectOptions {
        extension_hint: Some("xlsx".to_string()),
        ..Default::default()
    };

    // Fixture is actually DOCX
    let docx_path = Path::new("fixtures/true_docx.docx");
    let rep_with_mismatched_hint =
        detect_path(docx_path, &mismatched_options).expect("Detection must succeed");

    assert_eq!(
        rep_with_mismatched_hint.identification,
        Identification::Format
    );
    assert_eq!(rep_with_mismatched_hint.format_id.as_deref(), Some("docx"));
    assert_eq!(
        rep_with_mismatched_hint.extension_status,
        ExtensionStatus::Match
    );

    // Test with detect_reader where path is not available
    let file_bytes = std::fs::read(docx_path).expect("Read fixture");
    let mut cursor = Cursor::new(&file_bytes);
    let rep_reader = detect_reader(&mut cursor, &mismatched_options).expect("Reader detect");

    assert_eq!(rep_reader.identification, Identification::Format);
    assert_eq!(rep_reader.format_id.as_deref(), Some("docx"));
}
