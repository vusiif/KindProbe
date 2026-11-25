use std::io::{Cursor, Write};
use std::time::Instant;

use kindprobe::{detect_bytes, DetectOptions, Identification};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

fn build_zip_bytes(entries: &[(&str, &[u8], CompressionMethod)]) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut zw = ZipWriter::new(&mut buf);
        for &(name, data, method) in entries {
            let opts = SimpleFileOptions::default().compression_method(method);
            zw.start_file(name, opts).expect("start_file");
            zw.write_all(data).expect("write_all");
        }
        zw.finish().expect("finish");
    }
    buf.into_inner()
}

fn build_ooxml_sample(
    rel_target: &str,
    content_type: &str,
    main_xml: &[u8],
    extra_entries: usize,
    method: CompressionMethod,
) -> Vec<u8> {
    let rels = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="{}"/>
</Relationships>"#,
        rel_target
    );
    let ct = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/{}" ContentType="{}"/>
</Types>"#,
        rel_target, content_type
    );

    let mut buf = Cursor::new(Vec::new());
    {
        let mut zw = ZipWriter::new(&mut buf);
        let opts = SimpleFileOptions::default().compression_method(method);
        zw.start_file("_rels/.rels", opts).expect("rels");
        zw.write_all(rels.as_bytes()).expect("rels write");
        zw.start_file("[Content_Types].xml", opts).expect("ct");
        zw.write_all(ct.as_bytes()).expect("ct write");
        zw.start_file(rel_target, opts).expect("main");
        zw.write_all(main_xml).expect("main write");
        for i in 0..extra_entries {
            let name = format!("word/media/part_{:03}.bin", i);
            zw.start_file(name, opts).expect("extra");
            zw.write_all(b"dummy-payload-0123456789abcdef").expect("extra write");
        }
        zw.finish().expect("finish");
    }
    buf.into_inner()
}

fn bench_group(name: &str, samples: &[(&str, Vec<u8>, Option<&str>)], iters: usize) -> f64 {
    let opts = DetectOptions::default();
    let start = Instant::now();
    for _ in 0..iters {
        for &(label, ref bytes, expected_fmt) in samples {
            let report = detect_bytes(bytes, &opts).expect("detect_bytes");
            assert_eq!(
                report.format_id.as_deref(),
                expected_fmt,
                "Mismatch on sample '{}': got {:?}, warnings={:?}",
                label,
                report.format_id,
                report.warnings
            );
            if expected_fmt.is_some() {
                assert_eq!(report.identification, Identification::Format);
            }
        }
    }
    let elapsed = start.elapsed().as_secs_f64();
    let total_ops = (iters * samples.len()) as f64;
    let ns_per_op = (elapsed * 1e9) / total_ops;
    let ops_per_sec = total_ops / elapsed;
    println!(
        "[Phase 18 Bench] {:<28} | {:>7} ops | {:>7.3} s | {:>9.1} ns/op | {:>10.0} ops/s",
        name, total_ops as usize, elapsed, ns_per_op, ops_per_sec
    );
    elapsed
}

#[test]
fn test_phase18_ooxml_and_text_anti_overfitting_bench() {
    let docx_small = build_ooxml_sample(
        "word/document.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
        br#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hello Phase 18</w:t></w:r></w:p></w:body></w:document>"#,
        2,
        CompressionMethod::Deflated,
    );
    let docm_stored = build_ooxml_sample(
        "word/document2.xml",
        "application/vnd.ms-word.document.macroEnabled.main+xml",
        br#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body/></w:document>"#,
        12,
        CompressionMethod::Stored,
    );
    let xlsx_medium = build_ooxml_sample(
        "xl/workbook.xml",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml",
        br#"<?xml version="1.0"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheets><sheet name="S1" sheetId="1" r:id="rId1"/></sheets></workbook>"#,
        22,
        CompressionMethod::Deflated,
    );
    let pptx_large_cd = build_ooxml_sample(
        "ppt/presentation.xml",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml",
        br#"<?xml version="1.0"?><p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"><p:sldIdLst/></p:presentation>"#,
        55, // > 32 entries to exercise FastNameSet::Large promotion
        CompressionMethod::Deflated,
    );

    let ooxml_samples = vec![
        ("docx_small_deflated", docx_small, Some("docx")),
        ("docm_stored_custom_path", docm_stored, Some("docm")),
        ("xlsx_medium_25_entries", xlsx_medium, Some("xlsx")),
        ("pptx_large_58_entries", pptx_large_cd, Some("pptx")),
    ];

    let plain_zip = build_zip_bytes(&[
        ("readme.txt", b"just a plain zip archive", CompressionMethod::Deflated),
        ("data/bin.dat", &[1, 2, 3, 4, 5], CompressionMethod::Stored),
    ]);
    let jar_zip = build_zip_bytes(&[
        ("META-INF/MANIFEST.MF", b"Manifest-Version: 1.0\r\n", CompressionMethod::Deflated),
        ("Main.class", b"\xCA\xFE\xBA\xBE\x00\x00\x00\x34", CompressionMethod::Stored),
    ]);
    let non_ooxml_zip_samples = vec![
        ("plain_zip", plain_zip, Some("zip")),
        ("jar_zip", jar_zip, Some("jar")),
    ];

    let ascii_prose_16k = "The quick brown fox jumps over the lazy dog while scanning large ASCII documents cleanly.\n"
        .repeat(200);
    let utf8_mixed_16k = format!(
        "{}\n2026-11-24 INFO 用户请求处理完成 latency=1.2ms status=ok\n{}",
        "server.worker.heartbeat ok=true\n".repeat(150),
        "日志条目结束：一切正常\n".repeat(120)
    );
    let json_payload_16k = format!(
        "{{\"items\":[{}],\"ok\":true}}",
        (0..220)
            .map(|i| format!("{{\"id\":{},\"name\":\"item_{}\"}}", i, i))
            .collect::<Vec<_>>()
            .join(",")
    );
    let rust_source_8k = "pub fn compute(v: &[u64]) -> u64 { v.iter().copied().sum() }\n".repeat(140);
    let mut near_miss_binary_8k = vec![b'A'; 8192];
    near_miss_binary_8k[3900] = 0x00; // NUL near end of 4KB text probe window

    let text_samples = vec![
        ("ascii_prose_16k", ascii_prose_16k.into_bytes(), Some("txt")),
        ("utf8_mixed_16k", utf8_mixed_16k.into_bytes(), Some("txt")),
        ("json_payload_16k", json_payload_16k.into_bytes(), Some("json")),
        ("rust_source_8k", rust_source_8k.into_bytes(), Some("rs")),
        ("near_miss_binary_8k", near_miss_binary_8k, None),
    ];

    let mut total = 0.0;
    total += bench_group("OOXML Diverse (Deflate/Stored)", &ooxml_samples, 600);
    total += bench_group("Non-OOXML ZIP Short-Circuit", &non_ooxml_zip_samples, 1500);
    total += bench_group("Text & Near-Miss 8K-16K", &text_samples, 1500);
    println!("[Phase 18 Bench] TOTAL elapsed: {:.3} s", total);
}
