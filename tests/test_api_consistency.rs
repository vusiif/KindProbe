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

#[test]
fn test_static_rule_index_invariants() {
    use kindprobe::detectors::rules::static_index::{BUCKETS, BY_FORMAT, OFFSET_GROUPS};
    use kindprobe::detectors::rules::{DECLARATIVE_RULES, GENERATED_RULES};
    use std::collections::HashSet;

    // 1. BY_FORMAT must stay strictly sorted so binary_search_by_key never misses an entry
    for pair in BY_FORMAT.windows(2) {
        assert!(
            pair[0].0 <= pair[1].0,
            "BY_FORMAT out of order: {:?} > {:?}",
            pair[0].0,
            pair[1].0
        );
    }
    for &(fid, rule) in BY_FORMAT {
        assert_eq!(fid, rule.format_id);
    }

    // 2. OFFSET_GROUPS must stay sorted by ascending offset
    for pair in OFFSET_GROUPS.windows(2) {
        assert!(
            pair[0].offset < pair[1].offset,
            "OFFSET_GROUPS out of order: {} >= {}",
            pair[0].offset,
            pair[1].offset
        );
    }

    // 3. Bucket + offset group entry counts must match total rules (DECLARATIVE + GENERATED)
    let bucket_count: usize = BUCKETS.iter().map(|b| b.len()).sum();
    let offset_count: usize = OFFSET_GROUPS.iter().map(|g| g.rules.len()).sum();
    let total_rules =
        DECLARATIVE_RULES.len() + GENERATED_RULES.iter().map(|g| g.len()).sum::<usize>();
    assert_eq!(bucket_count + offset_count, total_rules);
    assert_eq!(BY_FORMAT.len(), total_rules);

    // 4. Every rule in DECLARATIVE_RULES + GENERATED_RULES must be lookup-able via BY_FORMAT,
    //    and no exact duplicate (format_id, patterns) rule pairs should exist.
    type RuleKey<'a> = (&'a str, Vec<(usize, &'a [u8])>);
    let mut seen_rule_keys: HashSet<RuleKey<'_>> = HashSet::with_capacity(total_rules);
    for rule in DECLARATIVE_RULES
        .iter()
        .chain(GENERATED_RULES.iter().flat_map(|g| g.iter()))
    {
        assert!(
            BY_FORMAT
                .binary_search_by_key(&rule.format_id, |&(fid, _)| fid)
                .is_ok(),
            "missing BY_FORMAT entry for rule {}",
            rule.format_id
        );
        let pat_key: Vec<(usize, &[u8])> =
            rule.patterns.iter().map(|p| (p.offset, p.bytes)).collect();
        assert!(
            seen_rule_keys.insert((rule.format_id, pat_key)),
            "duplicate (format_id, patterns) rule found for {}",
            rule.format_id
        );
    }
}

#[test]
fn test_slice_vs_reader_budget_boundary_and_window_parity() {
    use kindprobe::Inspection;

    // 1. Plain text longer than 4 KiB with initial_window_size == max_source_bytes_read == 4096:
    //    both slice and reader must return txt + Complete without out-of-window ISO probe overflow.
    let plain_text = "Plain UTF-8 diagnostic log line for budget consistency checks.\n".repeat(700); // > 40 KiB
    let text_bytes = plain_text.as_bytes();
    assert!(text_bytes.len() > 40 * 1024);

    let tight_opts = DetectOptions {
        initial_window_size: 4096,
        max_source_bytes_read: 4096,
        ..Default::default()
    };
    let slice_tight = detect_bytes(text_bytes, &tight_opts).expect("slice tight");
    let slice_tight_core = detect_bytes_core(text_bytes, &tight_opts).expect("slice tight core");
    let mut cur_tight = Cursor::new(text_bytes);
    let reader_tight = detect_reader(&mut cur_tight, &tight_opts).expect("reader tight");
    let mut cur_tight_core = Cursor::new(text_bytes);
    let reader_tight_core =
        detect_reader_core(&mut cur_tight_core, &tight_opts).expect("reader tight core");

    for (rep, label) in [(&slice_tight, "slice"), (&reader_tight, "reader")] {
        assert_eq!(rep.format_id.as_deref(), Some("txt"), "{label}");
        assert_eq!(rep.inspection, Inspection::Complete, "{label}");
        assert_eq!(rep.stats.source_bytes_read, 4096, "{label}");
        assert!(rep.warnings.is_empty(), "{label}: {:?}", rep.warnings);
    }
    assert_eq!(slice_tight_core.format_id, Some("txt"));
    assert_eq!(slice_tight_core.inspection, Inspection::Complete);
    assert_eq!(reader_tight_core.format_id, Some("txt"));
    assert_eq!(reader_tight_core.inspection, Inspection::Complete);

    // 2. Relaxed budget on the same plain text must also avoid out-of-window ISO seeks/reads.
    let relaxed_opts = DetectOptions::default();
    let slice_relaxed = detect_bytes(text_bytes, &relaxed_opts).expect("slice relaxed");
    let mut cur_relaxed = Cursor::new(text_bytes);
    let reader_relaxed = detect_reader(&mut cur_relaxed, &relaxed_opts).expect("reader relaxed");
    assert_eq!(slice_relaxed.format_id.as_deref(), Some("txt"));
    assert_eq!(reader_relaxed.format_id.as_deref(), Some("txt"));
    assert_eq!(slice_relaxed.stats.source_bytes_read, 4096);
    assert_eq!(reader_relaxed.stats.source_bytes_read, 4096);

    // 3. Unknown binary > 32 KiB at budget boundary (4096, 4101, 4102) keeps slice/reader parity.
    let mut unknown_bin = vec![0x00u8; 40 * 1024];
    unknown_bin[..4].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
    for budget in [4096usize, 4101, 4102] {
        let opts = DetectOptions {
            initial_window_size: 4096,
            max_source_bytes_read: budget,
            ..Default::default()
        };
        let sb = detect_bytes(&unknown_bin, &opts).unwrap();
        let mut c = Cursor::new(&unknown_bin);
        let sr = detect_reader(&mut c, &opts).unwrap();
        assert_eq!(sb.identification, sr.identification, "budget={budget}");
        assert_eq!(sb.inspection, sr.inspection, "budget={budget}");
        assert_eq!(sb.inspection, Inspection::Complete, "budget={budget}");
        assert_eq!(
            sb.stats.source_bytes_read, sr.stats.source_bytes_read,
            "budget={budget}"
        );
    }

    // 4. Huge initial_window_size paired with tiny max_source_bytes_read must clamp allocation
    //    and still detect inputs smaller than max_source_bytes_read as Complete without OOM.
    let clamped_opts = DetectOptions {
        initial_window_size: usize::MAX / 2,
        max_source_bytes_read: 4096,
        ..Default::default()
    };
    let small_png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15c4";
    let sb_small = detect_bytes(small_png, &clamped_opts).expect("clamped slice");
    let mut cur_small = Cursor::new(small_png.as_slice());
    let sr_small = detect_reader(&mut cur_small, &clamped_opts).expect("clamped reader");
    assert_eq!(sb_small.format_id.as_deref(), Some("png"));
    assert_eq!(sr_small.format_id.as_deref(), Some("png"));
    assert_eq!(sb_small.inspection, Inspection::Complete);
    assert_eq!(sr_small.inspection, Inspection::Complete);

    // 5. Large initial_window_size covering ISO PVD reuses header buffer with 0 extra bytes read.
    let mut iso_data = vec![0u8; 0x8000 + 2048];
    iso_data[0x8000..0x8000 + 6].copy_from_slice(b"\x01CD001");
    let iso_window_opts = DetectOptions {
        initial_window_size: iso_data.len(),
        max_source_bytes_read: iso_data.len(),
        ..Default::default()
    };
    let iso_slice = detect_bytes(&iso_data, &iso_window_opts).expect("iso window slice");
    let mut iso_cur = Cursor::new(&iso_data);
    let iso_reader = detect_reader(&mut iso_cur, &iso_window_opts).expect("iso window reader");
    assert_eq!(iso_slice.format_id.as_deref(), Some("iso"));
    assert_eq!(iso_reader.format_id.as_deref(), Some("iso"));
    assert_eq!(iso_slice.stats.source_bytes_read, iso_data.len());
    assert_eq!(iso_reader.stats.source_bytes_read, iso_data.len());

    // 6. 4,096-byte DMG at exact initial_window_size == max_source_bytes_read == 4096
    //    must detect `dmg` / `Complete` on both slice and reader.
    let mut dmg_4096 = vec![0u8; 4096];
    let koly_off = 4096 - 512;
    dmg_4096[koly_off..koly_off + 4].copy_from_slice(b"koly");
    dmg_4096[koly_off + 4..koly_off + 8].copy_from_slice(&4u32.to_be_bytes());
    let dmg_slice = detect_bytes(&dmg_4096, &tight_opts).expect("dmg 4096 slice");
    let dmg_slice_core = detect_bytes_core(&dmg_4096, &tight_opts).expect("dmg 4096 slice core");
    let mut dmg_cur = Cursor::new(&dmg_4096);
    let dmg_reader = detect_reader(&mut dmg_cur, &tight_opts).expect("dmg 4096 reader");
    let mut dmg_cur_core = Cursor::new(&dmg_4096);
    let dmg_reader_core =
        detect_reader_core(&mut dmg_cur_core, &tight_opts).expect("dmg 4096 reader core");
    assert_eq!(dmg_slice.format_id.as_deref(), Some("dmg"));
    assert_eq!(dmg_slice.inspection, Inspection::Complete);
    assert_eq!(dmg_reader.format_id.as_deref(), Some("dmg"));
    assert_eq!(dmg_reader.inspection, Inspection::Complete);
    assert_eq!(dmg_slice_core.format_id, Some("dmg"));
    assert_eq!(dmg_reader_core.format_id, Some("dmg"));
}
