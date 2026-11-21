use std::ffi::OsStr;
use std::io::Cursor;
use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, detect_reader, ExtensionStatus, Identification};

struct FastRng(u64);

impl FastRng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0x853c49e6748fea9b } else { seed })
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) {
        for chunk in buf.chunks_mut(8) {
            let val = self.next_u64();
            let val_bytes = val.to_le_bytes();
            let len = chunk.len();
            chunk.copy_from_slice(&val_bytes[..len]);
        }
    }
}

#[test]
fn test_chaos_fuzzing_no_panic_and_invariants() {
    let mut rng = FastRng::new(0xDEADBEEF_CAFE1234);
    let opts = DetectOptions::default();

    let lengths = [0, 1, 2, 3, 4, 7, 8, 15, 16, 64, 127, 128, 512, 1024, 4096, 8192];

    for &len in &lengths {
        for _ in 0..100 {
            let mut payload = vec![0u8; len];
            rng.fill_bytes(&mut payload);

            // Invariant 1: detect_bytes must never panic
            let rep1 = detect_bytes(&payload, &opts).expect("detect_bytes never errors on mem slice");

            // Invariant 2: detect_reader must match detect_bytes idempotently
            let mut cursor = Cursor::new(&payload);
            let rep2 = detect_reader(&mut cursor, &opts).expect("detect_reader must succeed");

            assert_eq!(rep1.format_id, rep2.format_id);
            assert_eq!(rep1.identification, rep2.identification);
            assert_eq!(rep1.category, rep2.category);

            // Invariant 3: Extension diagnosis is purely orthogonal to content identification
            let ext_none = compare_extension(&rep1, None);
            assert_eq!(ext_none, ExtensionStatus::Missing);

            let ext_unrec = compare_extension(&rep1, Some(OsStr::new("nonexistent_unknown_ext_12345")));
            let expected_unrec = if rep1.identification == Identification::Format {
                ExtensionStatus::Unrecognized
            } else {
                ExtensionStatus::Undetermined
            };
            assert_eq!(ext_unrec, expected_unrec);

            // Invariant 4: Source read budget is strictly respected
            assert!(rep1.stats.source_bytes_read <= opts.max_source_bytes_read);
        }
    }
}

#[test]
fn test_prefix_truncation_fuzzing() {
    let opts = DetectOptions::default();

    let full_signatures: &[(&[u8], &str)] = &[
        (b"FLV\x01", "flv"),
        (b"#!AMR\n", "amr"),
        (b"\x89PNG\r\n\x1a\n", "png"),
        (b"%PDF-", "pdf"),
        (b"PK\x03\x04", "zip"),
        (b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1", "cfb"),
        (b"SQLite format 3\0", "sqlite3"),
        (b"fLaC", "flac"),
        (b"OggS", "ogg"),
        (b"gimp xcf ", "xcf"),
        (b"7z\xbc\xaf\x27\x1c", "7z"),
        (b"Rar!\x1a\x07\x00", "rar"),
    ];

    for &(sig, expected_cand) in full_signatures {
        for trunc_len in 1..sig.len() {
            let prefix = &sig[..trunc_len];
            let rep = detect_bytes(prefix, &opts).expect("detect prefix");

            // Must NOT falsely confirm the full format
            assert_eq!(rep.identification, Identification::Unknown, "Falsely confirmed prefix: {:?}", prefix);

            // If it identifies as truncated, must output warning and non-empty candidates
            if rep.warnings.iter().any(|w| w.code == "input.truncated") {
                let candidates = rep.candidates.as_deref().expect("truncated must provide candidates");
                assert!(!candidates.is_empty(), "Candidates list must not be empty for prefix {:?}", prefix);

                // For unambiguous multi-byte prefixes (len >= 2), candidate must match
                if prefix.len() >= 2 && !matches!(prefix, b"PK" | b"%P") {
                    assert!(
                        candidates.iter().any(|c| c == expected_cand),
                        "Expected candidate '{}' not found in {:?} for prefix {:?}",
                        expected_cand,
                        candidates,
                        prefix
                    );
                }
            }
        }
    }
}

#[test]
fn test_bit_flipping_mutation_fuzzing() {
    let mut rng = FastRng::new(0x12345678_9ABCDEF0);
    let opts = DetectOptions::default();

    let base_samples: &[&[u8]] = &[
        b"FLV\x01\x05\x00\x00\x00\x09",
        b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR",
        b"%PDF-1.7\n1 0 obj\n<<>>\nendobj",
        b"SQLite format 3\0\x10\x00\x01\x01",
        b"gimp xcf v011\x00\x00\x00",
    ];

    for &sample in base_samples {
        for _ in 0..50 {
            let mut mutated = sample.to_vec();
            // Flip 1 to 3 random bits in the header
            let flips = (rng.next_u64() % 3 + 1) as usize;
            for _ in 0..flips {
                let byte_idx = (rng.next_u64() as usize) % mutated.len();
                let bit_idx = (rng.next_u64() % 8) as u8;
                mutated[byte_idx] ^= 1 << bit_idx;
            }

            // Must never panic or crash under arbitrary bit-corruption
            let rep = detect_bytes(&mutated, &opts).expect("must survive bit-flip corruption");
            assert!(rep.stats.source_bytes_read <= opts.max_source_bytes_read);
        }
    }
}
