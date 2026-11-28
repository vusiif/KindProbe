use kindprobe::budget::DetectOptions;
use kindprobe::detect_bytes;
use kindprobe::extension::compare_extension;
use kindprobe::model::{ExtensionStatus, Identification, Verification};
use std::ffi::OsStr;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[test]
fn test_detect_audio_synth_and_db() {
    let opts = DetectOptions::default();

    // 1. SoundFont 2.0 (SF2)
    let mut sf2 = vec![0u8; 32];
    sf2[0..4].copy_from_slice(b"RIFF");
    sf2[8..12].copy_from_slice(b"sfbk");
    let rep_sf2 = detect_bytes(&sf2, &opts).expect("detect sf2");
    assert_eq!(rep_sf2.format_id.as_deref(), Some("sf2"));
    assert_eq!(rep_sf2.category, "audio");
    assert_eq!(
        compare_extension(&rep_sf2, Some(OsStr::new("sf2"))),
        ExtensionStatus::Match
    );

    // 2. Downloadable Sounds (DLS)
    let mut dls = vec![0u8; 32];
    dls[0..4].copy_from_slice(b"RIFF");
    dls[8..12].copy_from_slice(b"DLS ");
    let rep_dls = detect_bytes(&dls, &opts).expect("detect dls");
    assert_eq!(rep_dls.format_id.as_deref(), Some("dls"));
    assert_eq!(rep_dls.category, "audio");
    assert_eq!(
        compare_extension(&rep_dls, Some(OsStr::new("dls"))),
        ExtensionStatus::Match
    );

    // 3. True Audio (TTA)
    let mut tta = vec![0u8; 32];
    tta[0..4].copy_from_slice(b"TTA1");
    let rep_tta = detect_bytes(&tta, &opts).expect("detect tta");
    assert_eq!(rep_tta.format_id.as_deref(), Some("tta"));
    assert_eq!(rep_tta.category, "audio");
    assert_eq!(
        compare_extension(&rep_tta, Some(OsStr::new("tta"))),
        ExtensionStatus::Match
    );

    // 4. DuckDB
    let mut duckdb = vec![0u8; 32];
    duckdb[8..12].copy_from_slice(b"DUCK");
    let rep_duckdb = detect_bytes(&duckdb, &opts).expect("detect duckdb");
    assert_eq!(rep_duckdb.format_id.as_deref(), Some("duckdb"));
    assert_eq!(rep_duckdb.category, "database");
    assert_eq!(
        compare_extension(&rep_duckdb, Some(OsStr::new("duckdb"))),
        ExtensionStatus::Match
    );

    // 5. Protein Data Bank (PDB)
    let mut pdb = vec![0u8; 32];
    pdb[0..10].copy_from_slice(b"HEADER    ");
    let rep_pdb = detect_bytes(&pdb, &opts).expect("detect pdb");
    assert_eq!(rep_pdb.format_id.as_deref(), Some("pdb"));
    assert_eq!(rep_pdb.category, "scientific");
    assert_eq!(
        compare_extension(&rep_pdb, Some(OsStr::new("pdb"))),
        ExtensionStatus::Match
    );

    // 6. Generic RIFF Container fallback
    let mut riff_generic = vec![0u8; 32];
    riff_generic[0..4].copy_from_slice(b"RIFF");
    riff_generic[8..12].copy_from_slice(b"UNKN");
    let rep_generic = detect_bytes(&riff_generic, &opts).expect("detect generic riff");
    assert_eq!(rep_generic.identification, Identification::ContainerOnly);
    assert_eq!(rep_generic.container_id.as_deref(), Some("riff"));
}

#[test]
fn test_detect_packages_and_disks() {
    let opts = DetectOptions::default();

    // 1. LZ4 Archive
    let mut lz4 = vec![0u8; 32];
    lz4[0..4].copy_from_slice(b"\x04\x22\x4d\x18");
    let rep_lz4 = detect_bytes(&lz4, &opts).expect("detect lz4");
    assert_eq!(rep_lz4.format_id.as_deref(), Some("lz4"));
    assert_eq!(rep_lz4.category, "archive");
    assert_eq!(
        compare_extension(&rep_lz4, Some(OsStr::new("lz4"))),
        ExtensionStatus::Match
    );

    // 2. PeaZip Archive (PEA)
    let mut pea = vec![0u8; 32];
    pea[0..3].copy_from_slice(b"\xea\x06\x00");
    let rep_pea = detect_bytes(&pea, &opts).expect("detect pea");
    assert_eq!(rep_pea.format_id.as_deref(), Some("pea"));
    assert_eq!(rep_pea.category, "archive");
    assert_eq!(
        compare_extension(&rep_pea, Some(OsStr::new("pea"))),
        ExtensionStatus::Match
    );

    // 3. QCOW v1
    let mut qcow = vec![0u8; 32];
    qcow[0..4].copy_from_slice(b"QFI\xeb");
    let rep_qcow = detect_bytes(&qcow, &opts).expect("detect qcow");
    assert_eq!(rep_qcow.format_id.as_deref(), Some("qcow"));
    assert_eq!(rep_qcow.category, "disk");
    assert_eq!(
        compare_extension(&rep_qcow, Some(OsStr::new("qcow"))),
        ExtensionStatus::Match
    );

    // 4. Apple Sparse Image
    let mut sprs = vec![0u8; 32];
    sprs[0..4].copy_from_slice(b"sprs");
    let rep_sprs = detect_bytes(&sprs, &opts).expect("detect sprs");
    assert_eq!(rep_sprs.format_id.as_deref(), Some("sparseimage"));
    assert_eq!(rep_sprs.category, "disk");
    assert_eq!(
        compare_extension(&rep_sprs, Some(OsStr::new("sparseimage"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_images_and_rom_patches() {
    let opts = DetectOptions::default();

    // 1. PCX
    let mut pcx = vec![0u8; 32];
    pcx[0..3].copy_from_slice(b"\x0a\x05\x01");
    let rep_pcx = detect_bytes(&pcx, &opts).expect("detect pcx");
    assert_eq!(rep_pcx.format_id.as_deref(), Some("pcx"));
    assert_eq!(rep_pcx.category, "image");
    assert_eq!(
        compare_extension(&rep_pcx, Some(OsStr::new("pcx"))),
        ExtensionStatus::Match
    );

    // 2. XPM
    let mut xpm = vec![0u8; 32];
    xpm[0..9].copy_from_slice(b"/* XPM */");
    let rep_xpm = detect_bytes(&xpm, &opts).expect("detect xpm");
    assert_eq!(rep_xpm.format_id.as_deref(), Some("xpm"));
    assert_eq!(rep_xpm.category, "image");
    assert_eq!(
        compare_extension(&rep_xpm, Some(OsStr::new("xpm"))),
        ExtensionStatus::Match
    );

    // 3. IPS Patch
    let mut ips = vec![0u8; 32];
    ips[0..5].copy_from_slice(b"PATCH");
    let rep_ips = detect_bytes(&ips, &opts).expect("detect ips");
    assert_eq!(rep_ips.format_id.as_deref(), Some("ips"));
    assert_eq!(rep_ips.category, "rom");
    assert_eq!(
        compare_extension(&rep_ips, Some(OsStr::new("ips"))),
        ExtensionStatus::Match
    );

    // 4. BPS Patch
    let mut bps = vec![0u8; 32];
    bps[0..4].copy_from_slice(b"BPS1");
    let rep_bps = detect_bytes(&bps, &opts).expect("detect bps");
    assert_eq!(rep_bps.format_id.as_deref(), Some("bps"));
    assert_eq!(rep_bps.category, "rom");
    assert_eq!(
        compare_extension(&rep_bps, Some(OsStr::new("bps"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_odg_and_kmz() {
    let opts = DetectOptions::default();
    let zip_opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // 1. OpenDocument Graphics (ODG)
    let cursor = Cursor::new(Vec::new());
    let mut zip_odg = ZipWriter::new(cursor);
    zip_odg
        .start_file("mimetype", zip_opts)
        .expect("start mimetype");
    zip_odg
        .write_all(b"application/vnd.oasis.opendocument.graphics")
        .expect("write mimetype");
    let odg_data = zip_odg.finish().expect("finish odg").into_inner();

    let rep_odg = detect_bytes(&odg_data, &opts).expect("detect odg");
    assert_eq!(rep_odg.identification, Identification::Format);
    assert_eq!(rep_odg.format_id.as_deref(), Some("odg"));
    assert_eq!(rep_odg.category, "document");
    assert_eq!(rep_odg.container_id.as_deref(), Some("zip"));
    assert_eq!(rep_odg.verification, Verification::Structure);
    assert_eq!(
        compare_extension(&rep_odg, Some(OsStr::new("odg"))),
        ExtensionStatus::Match
    );

    // 2. Keyhole Markup Language Zipped (KMZ)
    let cursor = Cursor::new(Vec::new());
    let mut zip_kmz = ZipWriter::new(cursor);
    zip_kmz
        .start_file("doc.kml", zip_opts)
        .expect("start doc.kml");
    zip_kmz
        .write_all(b"<kml xmlns=\"http://www.opengis.net/kml/2.2\"></kml>")
        .expect("write kml");
    let kmz_data = zip_kmz.finish().expect("finish kmz").into_inner();

    let rep_kmz = detect_bytes(&kmz_data, &opts).expect("detect kmz");
    assert_eq!(rep_kmz.identification, Identification::Format);
    assert_eq!(rep_kmz.format_id.as_deref(), Some("kmz"));
    assert_eq!(rep_kmz.category, "geospatial");
    assert_eq!(rep_kmz.container_id.as_deref(), Some("zip"));
    assert_eq!(rep_kmz.verification, Verification::Structure);
    assert_eq!(
        compare_extension(&rep_kmz, Some(OsStr::new("kmz"))),
        ExtensionStatus::Match
    );
}
