use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use kindprobe::budget::DetectOptions;
use kindprobe::{detect_bytes, detect_reader, Identification, Inspection};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

struct InterruptedReader<R> {
    inner: R,
    toggle: bool,
}

impl<R: Read> Read for InterruptedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.toggle = !self.toggle;
        if self.toggle {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "simulated EINTR"));
        }
        self.inner.read(buf)
    }
}

impl<R: Seek> Seek for InterruptedReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

fn build_valid_docx_bytes(extra_padding_xml: &str) -> Vec<u8> {
    let rels = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;
    let ct = r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{}</w:body></w:document>"#,
        extra_padding_xml
    );

    let mut buf = Cursor::new(Vec::new());
    {
        let mut zw = ZipWriter::new(&mut buf);
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        zw.start_file("_rels/.rels", opts).expect("rels");
        zw.write_all(rels.as_bytes()).expect("rels write");
        zw.start_file("[Content_Types].xml", opts).expect("ct");
        zw.write_all(ct.as_bytes()).expect("ct write");
        zw.start_file("word/document.xml", opts).expect("doc");
        zw.write_all(doc.as_bytes()).expect("doc write");
        zw.finish().expect("finish");
    }
    buf.into_inner()
}

#[test]
fn test_phase17_budget_reader_interrupted_retry() {
    let docx = build_valid_docx_bytes("<w:p><w:r><w:t>EINTR resilience</w:t></w:r></w:p>");
    let mut reader = InterruptedReader {
        inner: Cursor::new(docx),
        toggle: false,
    };
    let rep = detect_reader(&mut reader, &DetectOptions::default()).expect("detect_reader");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("docx"));
    assert_eq!(rep.inspection, Inspection::Complete);
}

#[test]
fn test_phase17_streaming_xml_budget_exceeded() {
    let big_para = "<w:p><w:r><w:t>AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA</w:t></w:r></w:p>".repeat(400);
    let docx = build_valid_docx_bytes(&big_para);
    let opts = DetectOptions {
        max_single_decompressed_bytes: 2048,
        ..DetectOptions::default()
    };
    let rep = detect_bytes(&docx, &opts).expect("detect_bytes");
    assert_eq!(rep.inspection, Inspection::Limited);
    assert!(
        rep.warnings.iter().any(|w| w.code == "budget.xml_output_exceeded"),
        "Expected budget.xml_output_exceeded, got: {:?}",
        rep.warnings
    );
}

#[test]
fn test_phase17_zip_overlap_cd_rejected() {
    let mut zip_bytes = build_valid_docx_bytes("<w:p/>");
    // Locate last PK\x01\x02 in Central Directory and corrupt its compressed_size to overlap cd_offset
    let cd_pos = zip_bytes
        .windows(4)
        .rposition(|w| w == b"PK\x01\x02")
        .expect("find CD header");
    // compressed_size at offset +20..+24: set to 0x0FFF_FFFF (overlaps cd_offset)
    zip_bytes[cd_pos + 20..cd_pos + 24].copy_from_slice(&0x0FFF_FFFFu32.to_le_bytes());

    let rep = detect_bytes(&zip_bytes, &DetectOptions::default()).expect("detect overlap zip");
    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
    assert_eq!(rep.inspection, Inspection::Limited);
    assert!(rep
        .warnings
        .iter()
        .any(|w| w.code == "zip.truncated_or_corrupted"));
}

#[test]
fn test_phase17_forged_ppt_current_user_rejected() {
    let cursor = Cursor::new(Vec::new());
    let mut comp = cfb::CompoundFile::create(cursor).expect("create cfb");
    {
        let mut ppt_data = vec![0u8; 64];
        ppt_data[0..2].copy_from_slice(&0x000Fu16.to_le_bytes());
        ppt_data[2..4].copy_from_slice(&0x03E8u16.to_le_bytes());
        ppt_data[4..8].copy_from_slice(&56u32.to_le_bytes());
        let mut s = comp.create_stream("/PowerPoint Document").expect("ppt");
        s.write_all(&ppt_data).expect("write ppt");
    }
    {
        let mut cu = [0u8; 20];
        cu[2..4].copy_from_slice(&0x0FF6u16.to_le_bytes());
        cu[12..16].copy_from_slice(&0xDEAD_BEEFu32.to_le_bytes()); // bad headerToken
        let mut s = comp.create_stream("/Current User").expect("cu");
        s.write_all(&cu).expect("write cu");
    }
    let data = comp.into_inner().into_inner();
    let rep = detect_bytes(&data, &DetectOptions::default()).expect("detect forged cu");
    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.format_id, None);
    assert!(rep
        .warnings
        .iter()
        .any(|w| w.code == "cfb.invalid_powerpoint_stream"));
}

#[test]
fn test_step3_eocd_forged_entry_count_and_xml_empty_depth() {
    use kindprobe::budget::BudgetReader;
    use std::io::Read as _;

    // 1. Forged ZIP EOCD total_entries=1 while actual CD contains 5 PK\x01\x02 entries
    //    (tested with max_entries=3): actual parsed count must trigger budget.entries_exceeded.
    let mut raw = Vec::new();
    {
        let mut zw = ZipWriter::new(Cursor::new(&mut raw));
        let zo = SimpleFileOptions::default();
        for idx in 0..5 {
            zw.start_file(format!("f{}.txt", idx), zo).unwrap();
            zw.write_all(b"hi").unwrap();
        }
        zw.finish().unwrap();
    }
    let eocd_pos = raw.windows(4).rposition(|w| w == b"PK\x05\x06").unwrap();
    raw[eocd_pos + 8..eocd_pos + 10].copy_from_slice(&1u16.to_le_bytes());
    raw[eocd_pos + 10..eocd_pos + 12].copy_from_slice(&1u16.to_le_bytes());

    let opts = DetectOptions {
        max_entries: 3,
        ..DetectOptions::default()
    };
    let rep = detect_bytes(&raw, &opts).unwrap();
    assert_eq!(rep.identification, Identification::ContainerOnly);
    assert_eq!(rep.inspection, Inspection::Limited);
    assert!(rep
        .warnings
        .iter()
        .any(|w| w.code == "budget.entries_exceeded"));

    // 2. Empty XML element (<leaf/>) at depth > max_xml_depth must trigger budget.xml_depth_exceeded
    let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r/></w:p></w:body></w:document>"#;
    let rels = r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let ct = r#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    let mut empty_elem_docx = Vec::new();
    {
        let mut zw = ZipWriter::new(Cursor::new(&mut empty_elem_docx));
        let zo = SimpleFileOptions::default();
        zw.start_file("_rels/.rels", zo).unwrap();
        zw.write_all(rels.as_bytes()).unwrap();
        zw.start_file("[Content_Types].xml", zo).unwrap();
        zw.write_all(ct.as_bytes()).unwrap();
        zw.start_file("word/document.xml", zo).unwrap();
        zw.write_all(xml.as_bytes()).unwrap();
        zw.finish().unwrap();
    }
    let depth_opts = DetectOptions {
        max_xml_depth: 3, // root(1) -> body(2) -> p(3) -> <w:r/>(4)
        ..DetectOptions::default()
    };
    let rep_depth = detect_bytes(&empty_elem_docx, &depth_opts).unwrap();
    assert_eq!(rep_depth.inspection, Inspection::Limited);
    assert!(rep_depth
        .warnings
        .iter()
        .any(|w| w.code == "budget.xml_depth_exceeded"));

    // 3. BudgetReader exact boundary EOF must not trigger budget_exceeded
    let mut br = BudgetReader::new(Cursor::new(b"abcd".to_vec()), 4);
    let mut buf = [0u8; 4];
    assert_eq!(br.read(&mut buf).unwrap(), 4);
    assert_eq!(br.read(&mut buf).unwrap(), 0);
    assert!(!br.is_budget_exceeded());
}
