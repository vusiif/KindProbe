use std::ffi::OsStr;
use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Identification, Inspection, Verification};

#[test]
fn test_detect_sqlite3_database() {
    let opts = DetectOptions::default();

    let mut sqlite = vec![0u8; 100];
    sqlite[0..16].copy_from_slice(b"SQLite format 3\0");
    sqlite[16..18].copy_from_slice(&4096u16.to_be_bytes()); // page_size = 4096
    sqlite[18] = 1; // write version = legacy
    sqlite[19] = 1; // read version = legacy

    let rep = detect_bytes(&sqlite, &opts).expect("detect sqlite");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("sqlite3"));
    assert_eq!(rep.category, "database");
    assert_eq!(rep.verification, Verification::Structure);
    assert_eq!(rep.inspection, Inspection::Complete);
    assert_eq!(rep.recommended_extension.as_deref(), Some("sqlite3"));
    assert_eq!(rep.accepted_extensions, vec!["sqlite3", "sqlite", "db", "db3"]);
    assert!(rep.evidence.iter().any(|e| e.code == "sig.sqlite.header"));
    assert!(rep.evidence.iter().any(|e| e.code == "sig.sqlite.page_size"));

    assert_eq!(compare_extension(&rep, Some(OsStr::new("sqlite3"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("sqlite"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("db"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("pcap"))), ExtensionStatus::Mismatch);
}

#[test]
fn test_detect_pcap_and_pcapng() {
    let opts = DetectOptions::default();

    // 1. Classic PCAP (Microsecond resolution, Little-Endian: 0xD4C3B2A1)
    let mut pcap = vec![0u8; 24];
    pcap[0..4].copy_from_slice(&[0xD4, 0xC3, 0xB2, 0xA1]);
    pcap[4..6].copy_from_slice(&2u16.to_le_bytes()); // version_major = 2
    pcap[6..8].copy_from_slice(&4u16.to_le_bytes()); // version_minor = 4

    let rep_pcap = detect_bytes(&pcap, &opts).expect("detect pcap");
    assert_eq!(rep_pcap.identification, Identification::Format);
    assert_eq!(rep_pcap.format_id.as_deref(), Some("pcap"));
    assert_eq!(rep_pcap.category, "capture");
    assert_eq!(rep_pcap.verification, Verification::Structure);
    assert_eq!(rep_pcap.recommended_extension.as_deref(), Some("pcap"));
    assert_eq!(rep_pcap.accepted_extensions, vec!["pcap", "cap", "dmp"]);
    assert!(rep_pcap.evidence.iter().any(|e| e.code == "sig.pcap.magic"));
    assert!(rep_pcap.evidence.iter().any(|e| e.code == "sig.pcap.version"));

    assert_eq!(compare_extension(&rep_pcap, Some(OsStr::new("pcap"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_pcap, Some(OsStr::new("cap"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_pcap, Some(OsStr::new("pcapng"))), ExtensionStatus::Mismatch);

    // 2. PCAPNG (Section Header Block 0x0A0D0D0A + BOM 0x1A2B3C4D)
    let mut pcapng = vec![0u8; 32];
    pcapng[0..4].copy_from_slice(&[0x0A, 0x0D, 0x0D, 0x0A]);
    pcapng[4..8].copy_from_slice(&32u32.to_le_bytes()); // Block total length
    pcapng[8..12].copy_from_slice(&[0x1A, 0x2B, 0x3C, 0x4D]); // Byte-Order Magic

    let rep_pcapng = detect_bytes(&pcapng, &opts).expect("detect pcapng");
    assert_eq!(rep_pcapng.identification, Identification::Format);
    assert_eq!(rep_pcapng.format_id.as_deref(), Some("pcapng"));
    assert_eq!(rep_pcapng.category, "capture");
    assert_eq!(rep_pcapng.verification, Verification::Structure);
    assert_eq!(rep_pcapng.recommended_extension.as_deref(), Some("pcapng"));
    assert_eq!(rep_pcapng.accepted_extensions, vec!["pcapng", "ntar"]);
    assert!(rep_pcapng.evidence.iter().any(|e| e.code == "sig.pcapng.shb"));
    assert!(rep_pcapng.evidence.iter().any(|e| e.code == "sig.pcapng.bom"));

    assert_eq!(compare_extension(&rep_pcapng, Some(OsStr::new("pcapng"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_pcapng, Some(OsStr::new("pcap"))), ExtensionStatus::Mismatch);
}

#[test]
fn test_detect_parquet_and_avro() {
    let opts = DetectOptions::default();

    // 1. Apache Parquet ("PAR1")
    let parquet = b"PAR1\x00\x00\x00\x00";
    let rep_parquet = detect_bytes(parquet, &opts).expect("detect parquet");
    assert_eq!(rep_parquet.identification, Identification::Format);
    assert_eq!(rep_parquet.format_id.as_deref(), Some("parquet"));
    assert_eq!(rep_parquet.category, "database");
    assert_eq!(rep_parquet.verification, Verification::Structure);
    assert_eq!(rep_parquet.recommended_extension.as_deref(), Some("parquet"));
    assert!(rep_parquet.evidence.iter().any(|e| e.code == "sig.parquet.magic"));

    assert_eq!(compare_extension(&rep_parquet, Some(OsStr::new("parquet"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_parquet, Some(OsStr::new("avro"))), ExtensionStatus::Mismatch);

    // 2. Apache Avro ("Obj\x01")
    let avro = b"Obj\x01\x02\x04\x00\x00";
    let rep_avro = detect_bytes(avro, &opts).expect("detect avro");
    assert_eq!(rep_avro.identification, Identification::Format);
    assert_eq!(rep_avro.format_id.as_deref(), Some("avro"));
    assert_eq!(rep_avro.category, "database");
    assert_eq!(rep_avro.verification, Verification::Structure);
    assert_eq!(rep_avro.recommended_extension.as_deref(), Some("avro"));
    assert!(rep_avro.evidence.iter().any(|e| e.code == "sig.avro.magic"));

    assert_eq!(compare_extension(&rep_avro, Some(OsStr::new("avro"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_avro, Some(OsStr::new("parquet"))), ExtensionStatus::Mismatch);
}

#[test]
fn test_detect_database_truncated() {
    let opts = DetectOptions::default();

    // 1. Truncated SQLite
    let rep_sqlite = detect_bytes(b"SQLite for", &opts).expect("truncated sqlite");
    assert_eq!(rep_sqlite.identification, Identification::Unknown);
    assert_eq!(rep_sqlite.candidates.as_deref(), Some(&["sqlite3".to_string()][..]));

    // 2. Truncated PCAP
    let rep_pcap = detect_bytes(&[0xD4, 0xC3], &opts).expect("truncated pcap");
    assert_eq!(rep_pcap.identification, Identification::Unknown);
    assert_eq!(rep_pcap.candidates.as_deref(), Some(&["pcap".to_string()][..]));

    // 3. Truncated Parquet
    let rep_par = detect_bytes(b"PAR", &opts).expect("truncated parquet");
    assert_eq!(rep_par.identification, Identification::Unknown);
    assert_eq!(rep_par.candidates.as_deref(), Some(&["parquet".to_string()][..]));

    // 4. Truncated Avro
    let rep_avro = detect_bytes(b"Obj", &opts).expect("truncated avro");
    assert_eq!(rep_avro.identification, Identification::Unknown);
    assert_eq!(rep_avro.candidates.as_deref(), Some(&["avro".to_string()][..]));
}
