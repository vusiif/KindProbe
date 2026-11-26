#![no_main]

use kindprobe::budget::DetectOptions;
use kindprobe::detect_bytes;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let opts = DetectOptions::default();
    let mut buf = Vec::with_capacity(data.len() + 8);
    if data[0] & 1 == 0 {
        buf.extend_from_slice(b"PK\x03\x04");
    } else {
        buf.extend_from_slice(b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1");
    }
    buf.extend_from_slice(&data[1..]);
    let _ = detect_bytes(&buf, &opts);
});
