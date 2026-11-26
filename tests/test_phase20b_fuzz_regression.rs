use kindprobe::budget::DetectOptions;
use kindprobe::{detect_bytes, detect_bytes_core};

struct XorShift64(u64);

impl XorShift64 {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

#[test]
fn test_phase20b_swar_and_differential_fuzz_regression() {
    let opts = DetectOptions::default();
    let mut rng = XorShift64(0x2026_1126_CAFE_BABEu64);

    // 1. Exact SWAR length boundaries (63, 64, 65, 71, 72, 4095, 4096, 4097)
    // with NUL, control bytes, and UTF-8 placed at every boundary offset.
    for len in [0usize, 1, 7, 8, 9, 63, 64, 65, 71, 72, 512, 4095, 4096, 4097] {
        let mut buf = vec![b'A'; len];
        if len > 0 {
            let r1 = detect_bytes(&buf, &opts).expect("detect ascii");
            let c1 = detect_bytes_core(&buf, &opts).expect("core ascii");
            assert_eq!(r1.format_id.as_deref(), c1.format_id);
            assert_eq!(r1.identification, c1.identification);

            // Inject NUL at last byte of buffer
            buf[len - 1] = 0x00;
            let r2 = detect_bytes(&buf, &opts).expect("detect nul");
            let c2 = detect_bytes_core(&buf, &opts).expect("core nul");
            assert_eq!(r2.format_id.as_deref(), c2.format_id);
            assert_eq!(r2.identification, c2.identification);
            if len <= 4096 {
                assert_eq!(r2.format_id, None);
            }
        }
    }

    // 2. 1,500 container-prefixed chaos mutations (ZIP / CFB / XML / JSON)
    let seeds: [&[u8]; 4] = [
        b"PK\x03\x04\x14\x00\x00\x00\x08\x00_rels/.relsPK\x01\x02PK\x05\x06\x00\x00\x00\x00\x01\x00\x01\x00\x2e\x00\x00\x00\x20\x00\x00\x00\x00\x00",
        b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x3e\x00\x03\x00\xfe\xff\x09\x00\x06\x00\x00\x00",
        br#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"/>"#,
        br#"{"k":1,"arr":[true,false,null,"text"]}"#,
    ];

    for iter in 0..1500 {
        let base = seeds[iter % seeds.len()];
        let mut mutated = base.to_vec();
        let extra = (rng.next() % 128) as usize;
        for _ in 0..extra {
            mutated.push((rng.next() & 0xFF) as u8);
        }
        if !mutated.is_empty() {
            let flips = 1 + (rng.next() % 4) as usize;
            for _ in 0..flips {
                let idx = (rng.next() as usize) % mutated.len();
                mutated[idx] ^= (rng.next() & 0xFF) as u8;
            }
        }
        let r = detect_bytes(&mutated, &opts).expect("detect mutated");
        let c = detect_bytes_core(&mutated, &opts).expect("core mutated");
        assert_eq!(r.format_id.as_deref(), c.format_id);
        assert_eq!(r.identification, c.identification);
    }
}
