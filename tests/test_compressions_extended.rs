use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Identification, Inspection, Verification};
use std::ffi::OsStr;

#[test]
fn test_detect_tar() {
    let opts = DetectOptions::default();

    // 512-byte POSIX ustar tar header block
    let mut tar_block = vec![0u8; 512];
    // File name: "test.txt"
    tar_block[0..8].copy_from_slice(b"test.txt");
    // File mode: "0000644\0"
    tar_block[100..108].copy_from_slice(b"0000644\0");
    // Magic: "ustar\0" at offset 257
    tar_block[257..263].copy_from_slice(b"ustar\0");
    // Version: "00" at offset 263
    tar_block[263..265].copy_from_slice(b"00");

    // Calculate tar checksum: treating offset 148..156 as 8 spaces (0x20)
    let mut sum: u32 = 0;
    for (i, &b) in tar_block.iter().enumerate() {
        if (148..156).contains(&i) {
            sum += 0x20;
        } else {
            sum += b as u32;
        }
    }
    let chk_str = format!("{:06o}\0 ", sum);
    tar_block[148..156].copy_from_slice(chk_str.as_bytes());

    let rep = detect_bytes(&tar_block, &opts).expect("detect tar");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("tar"));
    assert_eq!(rep.category, "archive");
    assert_eq!(rep.verification, Verification::Structure);
    assert_eq!(rep.inspection, Inspection::Complete);
    assert_eq!(rep.recommended_extension.as_deref(), Some("tar"));
    assert_eq!(rep.accepted_extensions, vec!["tar"]);
    assert!(rep.evidence.iter().any(|e| e.code == "sig.tar.ustar"));
    assert!(rep.evidence.iter().any(|e| e.code == "sig.tar.checksum"));

    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("tar"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("gz"))),
        ExtensionStatus::Mismatch
    );
}

#[test]
fn test_detect_gzip() {
    let opts = DetectOptions::default();

    // RFC 1952 Gzip header: \x1F\x8B + CM=8 (deflate) + FLG=0
    let mut gz = vec![0u8; 10];
    gz[0] = 0x1F;
    gz[1] = 0x8B;
    gz[2] = 0x08; // CM = deflate
    gz[3] = 0x00; // FLG = 0

    let rep = detect_bytes(&gz, &opts).expect("detect gzip");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("gzip"));
    assert_eq!(rep.category, "archive");
    assert_eq!(rep.verification, Verification::Structure);
    assert_eq!(rep.recommended_extension.as_deref(), Some("gz"));
    assert_eq!(rep.accepted_extensions, vec!["gz", "gzip", "tgz"]);
    assert!(rep.evidence.iter().any(|e| e.code == "sig.gzip.magic"));
    assert!(rep.evidence.iter().any(|e| e.code == "sig.gzip.header"));

    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("gz"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("gzip"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("tgz"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("bz2"))),
        ExtensionStatus::Mismatch
    );
}

#[test]
fn test_detect_bzip2() {
    let opts = DetectOptions::default();

    // Bzip2 header: BZh9 + Pi constant block 0x31 0x41 0x59 0x26 0x53 0x59
    let mut bz2 = vec![0u8; 10];
    bz2[0..3].copy_from_slice(b"BZh");
    bz2[3] = b'9'; // 900k block size
    bz2[4..10].copy_from_slice(&[0x31, 0x41, 0x59, 0x26, 0x53, 0x59]);

    let rep = detect_bytes(&bz2, &opts).expect("detect bzip2");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("bzip2"));
    assert_eq!(rep.category, "archive");
    assert_eq!(rep.verification, Verification::Structure);
    assert_eq!(rep.recommended_extension.as_deref(), Some("bz2"));
    assert_eq!(rep.accepted_extensions, vec!["bz2", "bzip2", "tbz2", "tbz"]);
    assert!(rep.evidence.iter().any(|e| e.code == "sig.bzip2.magic"));
    assert!(rep.evidence.iter().any(|e| e.code == "sig.bzip2.header"));

    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("bz2"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("tbz2"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("xz"))),
        ExtensionStatus::Mismatch
    );
}

#[test]
fn test_detect_xz_and_zstd() {
    let opts = DetectOptions::default();

    // 1. XZ format
    // Magic: [0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]
    // Stream Flags: [0x00, 0x00] -> CRC32 = 0x00000000 (IEEE crc32 of two zero bytes is 0x41d912ff inverted, let's calculate)
    let mut xz = vec![0u8; 12];
    xz[0..6].copy_from_slice(&[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]);
    xz[6..8].copy_from_slice(&[0x00, 0x00]);

    // calculate IEEE CRC32 of xz[6..8]
    let mut crc = 0xFFFF_FFFFu32;
    for &b in &xz[6..8] {
        crc ^= b as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    let flags_crc = !crc;
    xz[8..12].copy_from_slice(&flags_crc.to_le_bytes());

    let rep_xz = detect_bytes(&xz, &opts).expect("detect xz");
    assert_eq!(rep_xz.identification, Identification::Format);
    assert_eq!(rep_xz.format_id.as_deref(), Some("xz"));
    assert_eq!(rep_xz.category, "archive");
    assert_eq!(rep_xz.verification, Verification::Structure);
    assert_eq!(rep_xz.recommended_extension.as_deref(), Some("xz"));
    assert_eq!(rep_xz.accepted_extensions, vec!["xz", "txz"]);
    assert!(rep_xz.evidence.iter().any(|e| e.code == "sig.xz.magic"));
    assert!(rep_xz.evidence.iter().any(|e| e.code == "sig.xz.crc"));

    assert_eq!(
        compare_extension(&rep_xz, Some(OsStr::new("xz"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_xz, Some(OsStr::new("txz"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_xz, Some(OsStr::new("zst"))),
        ExtensionStatus::Mismatch
    );

    // 2. Zstandard format: [0x28, 0xB5, 0x2F, 0xFD]
    let zstd = [0x28, 0xB5, 0x2F, 0xFD, 0x00, 0x00];
    let rep_zstd = detect_bytes(&zstd, &opts).expect("detect zstd");
    assert_eq!(rep_zstd.identification, Identification::Format);
    assert_eq!(rep_zstd.format_id.as_deref(), Some("zstd"));
    assert_eq!(rep_zstd.category, "archive");
    assert_eq!(rep_zstd.verification, Verification::Structure);
    assert_eq!(rep_zstd.recommended_extension.as_deref(), Some("zst"));
    assert_eq!(rep_zstd.accepted_extensions, vec!["zst", "zstd", "tzst"]);
    assert!(rep_zstd.evidence.iter().any(|e| e.code == "sig.zstd.magic"));

    assert_eq!(
        compare_extension(&rep_zstd, Some(OsStr::new("zst"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_zstd, Some(OsStr::new("zstd"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_zstd, Some(OsStr::new("gz"))),
        ExtensionStatus::Mismatch
    );
}

#[test]
fn test_detect_truncated_compressions() {
    let opts = DetectOptions::default();

    let trunc_gz = detect_bytes(&[0x1F], &opts).expect("detect truncated gzip");
    assert_eq!(trunc_gz.identification, Identification::Unknown);
    assert!(trunc_gz
        .warnings
        .iter()
        .any(|w| w.code == "input.truncated"));

    let trunc_bz = detect_bytes(b"BZ", &opts).expect("detect truncated bzip2");
    assert_eq!(trunc_bz.identification, Identification::Unknown);
    assert!(trunc_bz
        .warnings
        .iter()
        .any(|w| w.code == "input.truncated"));
}
