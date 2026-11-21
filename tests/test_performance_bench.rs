use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;
use kindprobe::budget::DetectOptions;
use kindprobe::extension::find_registered_format;
use kindprobe::{detect_bytes, detect_path, detect_reader};

#[test]
fn test_micro_benchmark_performance() {
    let opts = DetectOptions::default();
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    let real_dir = fixtures_dir.join("real");

    println!("\n==================================================================");
    println!("             KindProbe Real Performance Benchmark                ");
    println!("==================================================================");

    // -------------------------------------------------------------------------
    // 1. Extension Registry Binary Search (203 entries)
    // -------------------------------------------------------------------------
    let ext_test_cases = [
        "png", "jpg", "docx", "blend", "h5", "pdf", "flv", "zip", "exe", "xyz123"
    ];
    let ext_iterations = 500_000;
    let start_ext = Instant::now();
    for i in 0..ext_iterations {
        let ext = ext_test_cases[i % ext_test_cases.len()];
        let _ = find_registered_format(ext);
    }
    let dur_ext = start_ext.elapsed();
    let ns_per_lookup = dur_ext.as_nanos() as f64 / ext_iterations as f64;
    let qps_ext = ext_iterations as f64 / dur_ext.as_secs_f64();
    println!("[1] Extension Binary Search (282 entries):");
    println!("    Total Lookups: {}", ext_iterations);
    println!("    Latency:       {:.2} ns / op", ns_per_lookup);
    println!("    Throughput:    {:.2} million ops / sec", qps_ext / 1_000_000.0);

    // -------------------------------------------------------------------------
    // 2. Memory Slice Content Detection (detect_bytes)
    // -------------------------------------------------------------------------
    let samples: &[(&str, &[u8])] = &[
        ("PNG Image", b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15\xc4\x89"),
        ("JPEG Image", b"\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01\x01\x01\x00\x60\x00\x60\x00\x00"),
        ("PDF Document", b"%PDF-1.7\n1 0 obj\n<<>>\nendobj"),
        ("Blender 3D", b"BLENDER_v293\x00\x00\x00\x00"),
        ("HDF5 Scientific", b"\x89HDF\r\n\x1a\n\x00\x00\x00\x00"),
        ("FLV Video", b"FLV\x01\x05\x00\x00\x00\x09\x00\x00\x00\x00"),
        ("Java KeyStore", b"\xfe\xed\xfe\xed\x00\x00\x00\x02"),
        ("LIDAR LAS Point Cloud", b"LASF\x00\x00\x00\x00\x01\x04"),
        ("Python Shebang", b"#!/usr/bin/env python3\nimport sys\nprint('hello')\n"),
        ("C Source Code", b"#include <stdio.h>\nint main() { printf(\"hello\\n\"); return 0; }\n"),
        ("Verilog HDL Source", b"module counter(input clk, output reg [3:0] q);\nalways @(posedge clk) q <= q + 1;\nendmodule\n"),
        ("Unknown Binary", b"\x12\x34\x56\x78\x9a\xbc\xde\xf0\x11\x22\x33\x44"),
    ];

    let bytes_iterations = 200_000;
    for &(name, data) in samples {
        let start = Instant::now();
        for _ in 0..bytes_iterations {
            let _ = detect_bytes(data, &opts);
        }
        let dur = start.elapsed();
        let ns_per_op = dur.as_nanos() as f64 / bytes_iterations as f64;
        let qps = bytes_iterations as f64 / dur.as_secs_f64();
        println!("[2] detect_bytes - {}:", name);
        println!("    Latency:    {:.2} ns ({:.3} μs) / detection", ns_per_op, ns_per_op / 1000.0);
        println!("    Throughput: {:.2} thousand files / sec", qps / 1000.0);
    }

    // -------------------------------------------------------------------------
    // 3. Real Disk File Detection (detect_path - includes OS open + fstat)
    // -------------------------------------------------------------------------
    let disk_files = [
        "sample.flv", "sample.blend", "sample.fits", "sample.h5", "sample.pbm"
    ];
    let path_iterations = 2_000;
    println!("[3] detect_path (Full OS open + fstat + budget read):");
    for &file_name in &disk_files {
        let path = real_dir.join(file_name);
        if !path.exists() { continue; }
        let start = Instant::now();
        for _ in 0..path_iterations {
            let _ = detect_path(&path, &opts);
        }
        let dur = start.elapsed();
        let us_per_op = dur.as_micros() as f64 / path_iterations as f64;
        let qps = path_iterations as f64 / dur.as_secs_f64();
        println!("    File {}: {:.2} μs / detection ({:.0} files/sec)", file_name, us_per_op, qps);
    }

    // -------------------------------------------------------------------------
    // 4. Deep Structural Container Detection (ZIP/OOXML DOCX & CFB DOC)
    // -------------------------------------------------------------------------
    let docx_path = fixtures_dir.join("true_docx.docx");
    let doc_path = fixtures_dir.join("true_doc.doc");
    let container_iterations = 5_000;

    println!("[4] Deep Container Structural Parsing (in-memory reader):");
    if docx_path.exists() {
        let mut f = File::open(&docx_path).unwrap();
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut f, &mut buf).unwrap();

        let start = Instant::now();
        for _ in 0..container_iterations {
            let mut cur = std::io::Cursor::new(&buf);
            let _ = detect_reader(&mut cur, &opts);
        }
        let dur = start.elapsed();
        let us_per_op = dur.as_micros() as f64 / container_iterations as f64;
        let qps = container_iterations as f64 / dur.as_secs_f64();
        println!("    DOCX (ZIP EOCD + Central Directory + XML Pull Parser):");
        println!("        Latency:    {:.2} μs / document", us_per_op);
        println!("        Throughput: {:.0} documents / sec", qps);
    }

    if doc_path.exists() {
        let mut f = File::open(&doc_path).unwrap();
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut f, &mut buf).unwrap();

        let start = Instant::now();
        for _ in 0..container_iterations {
            let mut cur = std::io::Cursor::new(&buf);
            let _ = detect_reader(&mut cur, &opts);
        }
        let dur = start.elapsed();
        let us_per_op = dur.as_micros() as f64 / container_iterations as f64;
        let qps = container_iterations as f64 / dur.as_secs_f64();
        println!("    DOC (CFB Sector Navigation + WordDocument Stream + FIB Verification):");
        println!("        Latency:    {:.2} μs / document", us_per_op);
        println!("        Throughput: {:.0} documents / sec", qps);
    }
    println!("==================================================================\n");
}
