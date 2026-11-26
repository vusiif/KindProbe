#![no_main]

use kindprobe::budget::DetectOptions;
use kindprobe::{detect_bytes, detect_bytes_core};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let opts = DetectOptions::default();
    if let (Ok(report), Ok(core)) = (detect_bytes(data, &opts), detect_bytes_core(data, &opts)) {
        assert_eq!(report.format_id.as_deref(), core.format_id);
        assert_eq!(report.identification, core.identification);
    }
});
