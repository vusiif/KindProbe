use std::path::Path;
use kindprobe::budget::DetectOptions;
use kindprobe::{detect_path, Identification, Inspection};

#[test]
fn test_budget_xml_depth() {
    let options = DetectOptions::default();

    // Deep XML fixture has 80 nested levels (budget is 64)
    let rep = detect_path(Path::new("fixtures/budget_xml_deep.docx"), &options).unwrap();
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("docx"));
    assert_eq!(rep.inspection, Inspection::Limited);
    assert!(rep.warnings.iter().any(|w| w.code == "budget.xml_depth_exceeded"));
}

#[test]
fn test_empty_and_truncated() {
    let options = DetectOptions::default();

    // Empty file
    let rep = detect_path(Path::new("fixtures/empty_file.bin"), &options).unwrap();
    assert_eq!(rep.identification, Identification::Unknown);
    assert!(rep.warnings.iter().any(|w| w.code == "input.empty"));

    // Truncated PNG
    let rep = detect_path(Path::new("fixtures/truncated_png.png"), &options).unwrap();
    assert_eq!(rep.identification, Identification::Unknown);
    assert!(rep.warnings.iter().any(|w| w.code == "input.truncated"));

    // Truncated PDF
    let rep = detect_path(Path::new("fixtures/truncated_pdf.pdf"), &options).unwrap();
    assert_eq!(rep.identification, Identification::Unknown);
    assert!(rep.warnings.iter().any(|w| w.code == "input.truncated"));

    // Truncated ZIP
    let rep = detect_path(Path::new("fixtures/truncated_zip.zip"), &options).unwrap();
    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.container_id.as_deref(), Some("zip"));
    assert_eq!(rep.inspection, Inspection::Limited);

    // Truncated CFB
    let rep = detect_path(Path::new("fixtures/truncated_cfb.doc"), &options).unwrap();
    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.container_id.as_deref(), Some("cfb"));
    assert_eq!(rep.inspection, Inspection::Limited);

    // Random garbage
    let rep = detect_path(Path::new("fixtures/random_garbage.bin"), &options).unwrap();
    assert_eq!(rep.identification, Identification::Unknown);
    assert!(rep.warnings.iter().any(|w| w.code == "format.unknown"));
}

#[test]
fn test_real_io_errors_return_err() {
    use std::io::{self, Read, Seek, SeekFrom};
    use kindprobe::detect_reader;
    use kindprobe::model::DetectError;

    struct FailingReader {
        fail_read: bool,
        fail_seek: bool,
    }

    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            if self.fail_read {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "simulated broken pipe"))
            } else {
                Ok(0)
            }
        }
    }

    impl Seek for FailingReader {
        fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
            if self.fail_seek {
                Err(io::Error::new(io::ErrorKind::Other, "simulated seek error"))
            } else {
                Ok(0)
            }
        }
    }

    let options = DetectOptions::default();

    // 1. Read failure
    let mut r1 = FailingReader { fail_read: true, fail_seek: false };
    match detect_reader(&mut r1, &options) {
        Err(DetectError::Io(e)) => assert_eq!(e.kind(), io::ErrorKind::BrokenPipe),
        other => panic!("Expected DetectError::Io, got {:?}", other),
    }

    // 2. Seek failure
    let mut r2 = FailingReader { fail_read: false, fail_seek: true };
    match detect_reader(&mut r2, &options) {
        Err(DetectError::Io(e)) => assert_eq!(e.kind(), io::ErrorKind::Other),
        other => panic!("Expected DetectError::Io, got {:?}", other),
    }
}

#[test]
fn test_budget_zip_directory_window() {
    let mut options = DetectOptions::default();
    // Set a very tiny directory window budget (10 bytes)
    options.max_zip_directory_window = 10;

    let rep = detect_path(Path::new("fixtures/plain_zip.zip"), &options).unwrap();
    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.inspection, Inspection::Limited);
    assert!(rep.warnings.iter().any(|w| w.code == "budget.zip_directory_window_exceeded"));
}

#[test]
fn test_budget_total_decompressed_bytes() {
    let mut options = DetectOptions::default();
    // Restrict total decompression budget to 50 bytes (DOCX parts exceed this)
    options.max_total_decompressed_bytes = 50;

    let rep = detect_path(Path::new("fixtures/true_docx.docx"), &options).unwrap();
    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.inspection, Inspection::Limited);
    assert!(rep.warnings.iter().any(|w| w.code == "budget.total_decompressed_exceeded"));
}

#[test]
fn test_cfb_entries_budget_and_stats() {
    let options = DetectOptions::default();

    // 1. Verify entries_inspected is populated (> 0)
    let rep = detect_path(Path::new("fixtures/plain_cfb.cfb"), &options).unwrap();
    assert!(rep.stats.entries_inspected > 0, "entries_inspected must be > 0 for CFB files");

    // 2. Verify max_entries triggers budget.entries_exceeded
    let mut restricted_options = DetectOptions::default();
    restricted_options.max_entries = 1;

    let rep_limited = detect_path(Path::new("fixtures/plain_cfb.cfb"), &restricted_options).unwrap();
    assert_eq!(rep_limited.inspection, Inspection::Limited);
    assert!(rep_limited.warnings.iter().any(|w| w.code == "budget.entries_exceeded"));
}

#[test]
fn test_zip_conflicting_duplicate_entries() {
    use kindprobe::detect_bytes;

    let mut zip = Vec::new();
    // 1. Local file header: "test.txt"
    zip.extend_from_slice(b"PK\x03\x04");
    zip.extend_from_slice(&[20, 0]); // version
    zip.extend_from_slice(&[0, 0]); // flags
    zip.extend_from_slice(&[0, 0]); // compression: none
    zip.extend_from_slice(&[0, 0, 0, 0]); // mtime/mdate
    zip.extend_from_slice(&[0, 0, 0, 0]); // crc32
    zip.extend_from_slice(&[0, 0, 0, 0]); // compressed size
    zip.extend_from_slice(&[0, 0, 0, 0]); // uncompressed size
    zip.extend_from_slice(&8u16.to_le_bytes()); // name length
    zip.extend_from_slice(&0u16.to_le_bytes()); // extra length
    zip.extend_from_slice(b"test.txt"); // file name

    let cd_offset = zip.len() as u32;

    // 2. Central directory entry 1: "test.txt"
    let cd_entry_start1 = zip.len();
    zip.extend_from_slice(b"PK\x01\x02");
    zip.extend_from_slice(&[20, 0, 20, 0]); // version made by / version needed
    zip.extend_from_slice(&[0, 0, 0, 0]); // flags, compression
    zip.extend_from_slice(&[0, 0, 0, 0]); // mtime/mdate
    zip.extend_from_slice(&[0, 0, 0, 0]); // crc32
    zip.extend_from_slice(&[0, 0, 0, 0]); // comp size
    zip.extend_from_slice(&[0, 0, 0, 0]); // uncomp size
    zip.extend_from_slice(&8u16.to_le_bytes()); // name len
    zip.extend_from_slice(&0u16.to_le_bytes()); // extra len
    zip.extend_from_slice(&0u16.to_le_bytes()); // comment len
    zip.extend_from_slice(&[0, 0, 0, 0]); // disk, internal attr
    zip.extend_from_slice(&[0, 0, 0, 0]); // external attr
    zip.extend_from_slice(&0u32.to_le_bytes()); // local header offset
    zip.extend_from_slice(b"test.txt");

    // 3. Central directory entry 2 (DUPLICATE): "test.txt"
    zip.extend_from_slice(b"PK\x01\x02");
    zip.extend_from_slice(&[20, 0, 20, 0]);
    zip.extend_from_slice(&[0, 0, 0, 0]);
    zip.extend_from_slice(&[0, 0, 0, 0]);
    zip.extend_from_slice(&[0, 0, 0, 0]);
    zip.extend_from_slice(&[0, 0, 0, 0]);
    zip.extend_from_slice(&[0, 0, 0, 0]);
    zip.extend_from_slice(&8u16.to_le_bytes());
    zip.extend_from_slice(&0u16.to_le_bytes());
    zip.extend_from_slice(&0u16.to_le_bytes());
    zip.extend_from_slice(&[0, 0, 0, 0]);
    zip.extend_from_slice(&[0, 0, 0, 0]);
    zip.extend_from_slice(&0u32.to_le_bytes());
    zip.extend_from_slice(b"test.txt");

    let cd_size = (zip.len() - cd_entry_start1) as u32;

    // 4. End of Central Directory
    zip.extend_from_slice(b"PK\x05\x06");
    zip.extend_from_slice(&[0, 0]); // disk num
    zip.extend_from_slice(&[0, 0]); // cd disk
    zip.extend_from_slice(&2u16.to_le_bytes()); // disk entries = 2
    zip.extend_from_slice(&2u16.to_le_bytes()); // total entries = 2
    zip.extend_from_slice(&cd_size.to_le_bytes()); // cd size
    zip.extend_from_slice(&cd_offset.to_le_bytes()); // cd offset
    zip.extend_from_slice(&[0, 0]); // comment len

    let options = DetectOptions::default();
    let rep = detect_bytes(&zip, &options).unwrap();
    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.inspection, Inspection::Ambiguous);
    assert!(rep.warnings.iter().any(|w| w.code == "zip.conflicting_entries"));
}
