use std::ffi::OsStr;
use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Verification};

#[test]
fn test_detect_batch_1000_finance_network_and_virtualization() {
    let opts = DetectOptions::default();

    // 1. FIX Protocol
    let fix = b"8=FIX.4.4\x019=123\x0135=D\x01";
    let rep = detect_bytes(fix, &opts).expect("detect fix");
    assert_eq!(rep.format_id.as_deref(), Some("fix_protocol"));
    assert_eq!(rep.category, "finance");
    assert_eq!(rep.verification, Verification::Signature);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("fix"))), ExtensionStatus::Match);

    // 2. SWIFT MT Message
    let swift = b"{1:F01BANKBEBBAXXX0000000000}{2:I103...";
    let rep = detect_bytes(swift, &opts).expect("detect swift");
    assert_eq!(rep.format_id.as_deref(), Some("swift_mt_msg"));
    assert_eq!(rep.category, "finance");

    // 3. GnuCash XML
    let gnc = b"<gnc-v2\n  xmlns:gnc=\"http://www.gnucash.org/XML/gnc\"";
    let rep = detect_bytes(gnc, &opts).expect("detect gnucash");
    assert_eq!(rep.format_id.as_deref(), Some("gnucash_xml"));
    assert_eq!(rep.category, "finance");

    // 4. sFlow v5
    let sflow = b"\x00\x00\x00\x05\x00\x00\x00\x01\x00\x00\x00\x00";
    let rep = detect_bytes(sflow, &opts).expect("detect sflow");
    assert_eq!(rep.format_id.as_deref(), Some("sflow_v5_dump"));
    assert_eq!(rep.category, "network");

    // 5. Cisco Running Config
    let cisco = b"!\n! Last configuration change at 12:00:00 UTC\nversion 15.1\n";
    let rep = detect_bytes(cisco, &opts).expect("detect cisco cfg");
    assert_eq!(rep.format_id.as_deref(), Some("cisco_running_config"));
    assert_eq!(rep.category, "network");

    // 6. Singularity SIF Container
    let sif = b"SIFIMAGE\x00\x00\x00\x00\x01\x00\x00\x00";
    let rep = detect_bytes(sif, &opts).expect("detect sif");
    assert_eq!(rep.format_id.as_deref(), Some("singularity_sif"));
    assert_eq!(rep.category, "virtualization");

    // 7. QEMU COW v1
    let qcow1 = b"OO-M\x00\x00\x00\x01\x00\x00\x00\x00";
    let rep = detect_bytes(qcow1, &opts).expect("detect qcow1");
    assert_eq!(rep.format_id.as_deref(), Some("qemu_cow_v1"));
    assert_eq!(rep.category, "virtualization");
}

#[test]
fn test_detect_batch_1000_crypto_compilers_and_bio() {
    let opts = DetectOptions::default();

    // 1. Bitcoin Block Data
    let blk = b"\xf9\xbe\xb4\xd9\x12\x34\x56\x78";
    let rep = detect_bytes(blk, &opts).expect("detect bitcoin blk");
    assert_eq!(rep.format_id.as_deref(), Some("bitcoin_blk_dat"));
    assert_eq!(rep.category, "blockchain");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("blk"))), ExtensionStatus::Match);

    // 2. Chia Plot
    let plot = b"Proof of Space Plot v1.0\x00";
    let rep = detect_bytes(plot, &opts).expect("detect chia plot");
    assert_eq!(rep.format_id.as_deref(), Some("chia_plot_v1"));
    assert_eq!(rep.category, "blockchain");

    // 3. Swift Serialized Intermediate SIL Bitstream
    let sib = b"SIBF\x00\x00\x00\x01\x00\x00\x00\x00";
    let rep = detect_bytes(sib, &opts).expect("detect swift sib");
    assert_eq!(rep.format_id.as_deref(), Some("swift_module_sib"));
    assert_eq!(rep.category, "bytecode");

    // 4. Dalvik ODEX
    let odex = b"dey\n036\x00\x01\x02\x03\x04";
    let rep = detect_bytes(odex, &opts).expect("detect odex");
    assert_eq!(rep.format_id.as_deref(), Some("dalvik_odex"));
    assert_eq!(rep.category, "bytecode");

    // 5. PLINK Binary BED
    let bed = b"\x6c\x1b\x01\x00\x00\x00";
    let rep = detect_bytes(bed, &opts).expect("detect plink bed");
    assert_eq!(rep.format_id.as_deref(), Some("plink_bed_binary"));
    assert_eq!(rep.category, "bioinformatics");

    // 6. GFF3 Annotation
    let gff3 = b"##gff-version 3\nchr1\t.\tgene\t1000\t2000";
    let rep = detect_bytes(gff3, &opts).expect("detect gff3");
    assert_eq!(rep.format_id.as_deref(), Some("gff3_annotation"));
    assert_eq!(rep.category, "bioinformatics");
}

#[test]
fn test_detect_batch_1000_geo_audio_and_office() {
    let opts = DetectOptions::default();

    // 1. DTED Elevation
    let dted = b"VOL1DTED\x00\x00\x00\x00\x00\x00";
    let rep = detect_bytes(dted, &opts).expect("detect dted");
    assert_eq!(rep.format_id.as_deref(), Some("dted_elevation"));
    assert_eq!(rep.category, "geospatial");

    // 2. Surfer ASCII Grid
    let dsaa = b"DSAA\n10 10\n0 1\n0 1\n";
    let rep = detect_bytes(dsaa, &opts).expect("detect surfer grid");
    assert_eq!(rep.format_id.as_deref(), Some("surfer_grid_ascii"));
    assert_eq!(rep.category, "geospatial");

    // 3. SoundFont 3
    let sf3 = b"SF3_PACKED_AUDIO\x00\x01\x00\x00";
    let rep = detect_bytes(sf3, &opts).expect("detect sf3");
    assert_eq!(rep.format_id.as_deref(), Some("soundfont_3"));
    assert_eq!(rep.category, "audio");

    // 4. Steinberg VST Preset
    let vst = b"VstP\x01\x00\x00\x00\x00\x00\x00\x00";
    let rep = detect_bytes(vst, &opts).expect("detect vstpreset");
    assert_eq!(rep.format_id.as_deref(), Some("steinberg_vstpreset"));
    assert_eq!(rep.category, "audio");

    // 5. Lotus 1-2-3 WK4
    let wk4 = b"\x00\x00\x1a\x00\x02\x10\x04\x00\x01\x00";
    let rep = detect_bytes(wk4, &opts).expect("detect wk4");
    assert_eq!(rep.format_id.as_deref(), Some("lotus_123_wk4"));
    assert_eq!(rep.category, "document");

    // 6. Lotus Word Pro
    let lwp = b"WordPro\x00\x00\x01\x00\x00\x00";
    let rep = detect_bytes(lwp, &opts).expect("detect lwp");
    assert_eq!(rep.format_id.as_deref(), Some("lotus_word_pro"));
    assert_eq!(rep.category, "document");
}

#[test]
fn test_detect_batch_1000_dumps_optics_db_auto_math_and_retro() {
    let opts = DetectOptions::default();

    // 1. Linux Vmcoreinfo
    let vmcore = b"VMCOREINFO\nOSRELEASE=5.15.0\nPAGESIZE=4096\n";
    let rep = detect_bytes(vmcore, &opts).expect("detect vmcoreinfo");
    assert_eq!(rep.format_id.as_deref(), Some("linux_vmcore_elf_note"));
    assert_eq!(rep.category, "forensics");

    // 2. CloneCD CCD
    let ccd = b"[CloneCD]\nVersion=3\n[Disc]\n";
    let rep = detect_bytes(ccd, &opts).expect("detect ccd");
    assert_eq!(rep.format_id.as_deref(), Some("clonecd_ccd"));
    assert_eq!(rep.category, "disk_image");

    // 3. MySQL InnoDB IBD
    let ibd = b"\xd4\x75\x5a\x6e\x00\x00\x00\x00";
    let rep = detect_bytes(ibd, &opts).expect("detect ibd");
    assert_eq!(rep.format_id.as_deref(), Some("mysql_innodb_ibd"));
    assert_eq!(rep.category, "database");

    // 4. ASAM MDF2
    let mdf2 = b"MDF     2.00    \x00\x00";
    let rep = detect_bytes(mdf2, &opts).expect("detect mdf2");
    assert_eq!(rep.format_id.as_deref(), Some("asam_mdf2"));
    assert_eq!(rep.category, "automotive");

    // 5. ASAM MDF3
    let mdf3 = b"MDF     3.30    \x00\x00";
    let rep = detect_bytes(mdf3, &opts).expect("detect mdf3");
    assert_eq!(rep.format_id.as_deref(), Some("asam_mdf3"));
    assert_eq!(rep.category, "automotive");

    // 6. Wolfram Mathematica Notebook
    let nb = b"(* Wolfram Notebook File *)\nNotebook[{\nCell[\"test\"]\n}]";
    let rep = detect_bytes(nb, &opts).expect("detect mathematica nb");
    assert_eq!(rep.format_id.as_deref(), Some("mathematica_notebook"));
    assert_eq!(rep.category, "scientific");

    // 7. Amiga HDF Hardfile
    let hdf = b"DOS\x03\x00\x00\x00\x00\x00\x00";
    let rep = detect_bytes(hdf, &opts).expect("detect amiga hdf");
    assert_eq!(rep.format_id.as_deref(), Some("amiga_hdf_hardfile"));
    assert_eq!(rep.category, "emulation");

    // 8. Vectrex Game ROM
    let vec = b"g GCE 1982\x00\x01\x02\x03";
    let rep = detect_bytes(vec, &opts).expect("detect vectrex");
    assert_eq!(rep.format_id.as_deref(), Some("vectrex_game_rom"));
    assert_eq!(rep.category, "emulation");
}
