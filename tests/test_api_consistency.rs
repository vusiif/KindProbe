use kindprobe::budget::DetectOptions;
use kindprobe::{detect_bytes, detect_bytes_core, detect_path, detect_reader, detect_reader_core};
use std::fs::{self, File};
use std::io::Cursor;
use std::path::Path;

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
        assert_eq!(
            rep_path.identification, rep_bytes.identification,
            "Mismatch for {}",
            fixture_name
        );
        assert_eq!(
            rep_path.format_id, rep_bytes.format_id,
            "Mismatch for {}",
            fixture_name
        );
        assert_eq!(
            rep_path.container_id, rep_bytes.container_id,
            "Mismatch for {}",
            fixture_name
        );
        assert_eq!(
            rep_path.verification, rep_bytes.verification,
            "Mismatch for {}",
            fixture_name
        );
        assert_eq!(
            rep_path.inspection, rep_bytes.inspection,
            "Mismatch for {}",
            fixture_name
        );

        assert_eq!(
            rep_bytes.identification, rep_reader.identification,
            "Mismatch for {}",
            fixture_name
        );
        assert_eq!(
            rep_bytes.format_id, rep_reader.format_id,
            "Mismatch for {}",
            fixture_name
        );
        assert_eq!(
            rep_bytes.container_id, rep_reader.container_id,
            "Mismatch for {}",
            fixture_name
        );
        assert_eq!(
            rep_bytes.verification, rep_reader.verification,
            "Mismatch for {}",
            fixture_name
        );
        assert_eq!(
            rep_bytes.inspection, rep_reader.inspection,
            "Mismatch for {}",
            fixture_name
        );

        let core_bytes = detect_bytes_core(&bytes, &options).unwrap();
        assert_eq!(core_bytes.format_id, rep_bytes.format_id.as_deref());
        assert_eq!(core_bytes.container_id, rep_bytes.container_id.as_deref());
        assert_eq!(core_bytes.identification, rep_bytes.identification);
        assert_eq!(core_bytes.verification, rep_bytes.verification);
        assert_eq!(core_bytes.inspection, rep_bytes.inspection);
    }
}

#[test]
fn test_core_vs_report_api_parity_across_domains() {
    let options = DetectOptions::default();

    let mut sqlite = vec![0u8; 72];
    sqlite[..16].copy_from_slice(b"SQLite format 3\0");
    sqlite[16..20].copy_from_slice(&[0x10, 0x00, 0x01, 0x01]);

    let mut wasm = vec![0u8; 8];
    wasm[..8].copy_from_slice(&[0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]);

    let mut mobi = vec![0u8; 68];
    mobi[60..68].copy_from_slice(b"BOOKMOBI");

    let cases: &[(&str, &[u8])] = &[
        ("gzip", &[0x1F, 0x8B, 0x08, 0x00, 0, 0, 0, 0, 0, 0xFF]),
        ("bzip2", b"BZh9\x31\x41\x59\x26\x53\x59\x00"),
        (
            "xz",
            &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00, 0, 0, 0, 0, 0, 0],
        ),
        ("zstd", &[0x28, 0xB5, 0x2F, 0xFD, 0x00, 0x00]),
        ("qcow2", &[0x51, 0x46, 0x49, 0xFB, 0, 0, 0, 3]),
        ("vhdx", b"vhdxfile\x00\x00"),
        ("vmdk", b"KDMV\x01\x00\x00\x00"),
        ("parquet", b"PAR1\x00\x00\x00\x00"),
        ("avro", b"Obj\x01\x00\x00\x00\x00"),
        (
            "pcapng",
            &[0x0A, 0x0D, 0x0D, 0x0A, 0, 0, 0, 0, 0x1A, 0x2B, 0x3C, 0x4D],
        ),
        ("sqlite3", &sqlite),
        ("wasm", &wasm),
        ("mobi", &mobi),
        ("flv", b"FLV\x01\x05\x00\x00\x00\x09"),
        ("ac3", &[0x0B, 0x77, 0x12, 0x34, 0x08, 0x40, 0x00, 0x00]),
        ("eac3", &[0x0B, 0x77, 0x01, 0x00, 0x00, 0x80, 0x00, 0x00]),
        ("sgi", &[0x01, 0xDA, 0x00, 0x01, 0x00, 0x02, 0x00, 0x01]),
        ("json", b"{\"kindprobe\": true, \"count\": 1}\n"),
        ("xml", b"<?xml version=\"1.0\"?><root><item/></root>\n"),
        ("yaml", b"---\nname: kindprobe\nversion: 1\n"),
        ("rust", b"fn main() {\n    println!(\"hello\");\n}\n"),
    ];

    for &(label, bytes) in cases {
        let rep = detect_bytes(bytes, &options).expect("detect_bytes");
        let core = detect_bytes_core(bytes, &options).expect("detect_bytes_core");
        let mut cursor = Cursor::new(bytes);
        let reader_rep = detect_reader(&mut cursor, &options).expect("detect_reader");
        let mut cursor_core = Cursor::new(bytes);
        let reader_core =
            detect_reader_core(&mut cursor_core, &options).expect("detect_reader_core");

        assert_eq!(core.format_id, rep.format_id.as_deref(), "{label}");
        assert_eq!(core.container_id, rep.container_id.as_deref(), "{label}");
        assert_eq!(core.identification, rep.identification, "{label}");
        assert_eq!(core.verification, rep.verification, "{label}");
        assert_eq!(core.inspection, rep.inspection, "{label}");
        assert_eq!(
            core.recommended_extension,
            rep.recommended_extension.as_deref(),
            "{label}"
        );
        assert_eq!(reader_rep.format_id, rep.format_id, "{label}");
        assert_eq!(reader_core.format_id, core.format_id, "{label}");
    }
}
