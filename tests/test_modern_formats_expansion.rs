use std::io::{Cursor, Write};
use kindprobe::budget::DetectOptions;
use kindprobe::extension::find_registered_format;
use kindprobe::{detect_bytes, detect_bytes_core, Identification};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

fn assert_both_paths(data: &[u8], expected_fmt: &str) {
    let opts = DetectOptions::default();
    let rep = detect_bytes(data, &opts).expect("detect_bytes");
    let core = detect_bytes_core(data, &opts).expect("detect_bytes_core");
    assert_eq!(rep.identification, Identification::Format, "fmt={}", expected_fmt);
    assert_eq!(rep.format_id.as_deref(), Some(expected_fmt));
    assert_eq!(core.format_id, Some(expected_fmt));
    assert_eq!(core.identification, Identification::Format);
}

#[test]
fn test_modern_isobmff_images_avif_heic_heif() {
    let avif = b"\x00\x00\x00\x18ftypavif\x00\x00\x00\x00avifmif1miaf";
    let heic = b"\x00\x00\x00\x18ftypheic\x00\x00\x00\x00mif1heic";
    let heif = b"\x00\x00\x00\x18ftypmif1\x00\x00\x00\x00mif1heic";

    assert_both_paths(avif, "avif");
    assert_both_paths(heic, "heic");
    assert_both_paths(heif, "heif");
}

#[test]
fn test_modern_ai_models_and_pst() {
    // GGUF v3
    let mut gguf = Vec::new();
    gguf.extend_from_slice(b"GGUF");
    gguf.extend_from_slice(&3u32.to_le_bytes());
    gguf.extend_from_slice(&[0u8; 16]);
    assert_both_paths(&gguf, "gguf");

    // Safetensors
    let header_json = br#"{"__metadata__":{"format":"pt"},"weight":{"dtype":"F16","shape":[4,4],"data_offsets":[0,32]}}"#;
    let mut safetensors = Vec::new();
    safetensors.extend_from_slice(&(header_json.len() as u64).to_le_bytes());
    safetensors.extend_from_slice(header_json);
    safetensors.extend_from_slice(&[0u8; 32]);
    assert_both_paths(&safetensors, "safetensors");

    // TFLite (FlatBuffer file_identifier "TFL3" at offset 4)
    let tflite = b"\x1c\x00\x00\x00TFL3\x00\x00\x00\x00\x00\x00";
    assert_both_paths(tflite, "tflite");

    // Outlook PST / OST
    let pst = b"!BDN\x00\x00\x00\x00SM\x13\x00\x01\x01\x00\x00";
    assert_both_paths(pst, "pst");
}

#[test]
fn test_modern_aab_zip_package() {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut zw = ZipWriter::new(&mut buf);
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        zw.start_file("BundleConfig.pb", opts).expect("start");
        zw.write_all(b"\x0a\x051.0.0").expect("write");
        zw.start_file("base/manifest/AndroidManifest.xml", opts)
            .expect("start2");
        zw.write_all(b"\x03\x00\x08\x00").expect("write2");
        zw.finish().expect("finish");
    }
    let bytes = buf.into_inner();
    assert_both_paths(&bytes, "aab");
}

#[test]
fn test_modern_text_eml_ini_dockerfile_css() {
    let eml = b"From: alice@example.com\r\nTo: bob@example.com\r\nSubject: Hello KindProbe\r\nMIME-Version: 1.0\r\n\r\nBody text.\r\n";
    assert_both_paths(eml, "eml");

    let ini = b"; Application configuration\n[database]\nhost = 127.0.0.1\nport = 5432\n";
    assert_both_paths(ini, "ini");

    let dockerfile = b"# Build stage\nFROM rust:1.85-alpine\nWORKDIR /app\nCOPY . .\nRUN cargo build --release\nCMD [\"./kindprobe\"]\n";
    assert_both_paths(dockerfile, "dockerfile");

    let css = b"body {\n  margin: 0;\n  padding: 16px;\n  color: #222;\n  display: flex;\n}\n";
    assert_both_paths(css, "css");
}

#[test]
fn test_new_extensions_registered() {
    for (ext, fid) in [
        ("aab", "aab"),
        ("avif", "avif"),
        ("css", "css"),
        ("dockerfile", "dockerfile"),
        ("eml", "eml"),
        ("gguf", "gguf"),
        ("heic", "heic"),
        ("heif", "heif"),
        ("ini", "ini"),
        ("ost", "pst"),
        ("pst", "pst"),
        ("safetensors", "safetensors"),
        ("tflite", "tflite"),
    ] {
        assert_eq!(find_registered_format(ext), Some(fid));
    }
}
