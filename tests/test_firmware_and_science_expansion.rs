use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Verification};
use std::ffi::OsStr;

#[test]
fn test_detect_firmware_and_forensics_batch() {
    let opts = DetectOptions::default();

    // 1. Rockchip Firmware (rkfw)
    let rkfw_data = b"RKFW\x00\x00\x01\x00\x00\x00\x00\x00";
    let rep = detect_bytes(rkfw_data, &opts).expect("detect rkfw");
    assert_eq!(rep.format_id.as_deref(), Some("rkfw"));
    assert_eq!(rep.category, "firmware");
    assert_eq!(rep.verification, Verification::Signature);
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("rkfw"))),
        ExtensionStatus::Match
    );

    // 2. Coreboot Filesystem (cbfs)
    let cbfs_data = b"ORBC\x00\x00\x00\x20\x00\x00\x00\x00";
    let rep = detect_bytes(cbfs_data, &opts).expect("detect cbfs");
    assert_eq!(rep.format_id.as_deref(), Some("cbfs"));
    assert_eq!(rep.category, "firmware");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("cbfs"))),
        ExtensionStatus::Match
    );

    // 3. Broadcom TRX Firmware (trx)
    let trx_data = b"HDR0\x00\x10\x00\x00\x00\x00\x00\x00";
    let rep = detect_bytes(trx_data, &opts).expect("detect trx");
    assert_eq!(rep.format_id.as_deref(), Some("trx"));
    assert_eq!(rep.category, "firmware");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("trx"))),
        ExtensionStatus::Match
    );

    // 4. Ubiquiti AirOS Firmware (ubnt)
    let ubnt_data = b"OPEN\x01\x00\x00\x00";
    let rep = detect_bytes(ubnt_data, &opts).expect("detect ubnt");
    assert_eq!(rep.format_id.as_deref(), Some("ubnt"));
    assert_eq!(rep.category, "firmware");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("ubnt"))),
        ExtensionStatus::Match
    );

    // 5. QNX Image Filesystem (ifs / qnxifs)
    let qnx_data = b"\xeb\x7e\xff\x00\x00\x00\x00\x00";
    let rep = detect_bytes(qnx_data, &opts).expect("detect qnxifs");
    assert_eq!(rep.format_id.as_deref(), Some("qnxifs"));
    assert_eq!(rep.category, "firmware");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("ifs"))),
        ExtensionStatus::Match
    );

    // 6. Android Sparse Image (simg)
    let simg_data = b"\x3a\xff\x26\xed\x01\x00\x00\x00";
    let rep = detect_bytes(simg_data, &opts).expect("detect simg");
    assert_eq!(rep.format_id.as_deref(), Some("simg"));
    assert_eq!(rep.category, "disk");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("simg"))),
        ExtensionStatus::Match
    );

    // 7. Android Boot Image (bootimg)
    let boot_data = b"ANDROID!\x00\x00\x00\x00\x00\x00\x00\x00";
    let rep = detect_bytes(boot_data, &opts).expect("detect bootimg");
    assert_eq!(rep.format_id.as_deref(), Some("bootimg"));
    assert_eq!(rep.category, "firmware");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("bootimg"))),
        ExtensionStatus::Match
    );

    // 8. Windows Event Log (evtx)
    let evtx_data = b"ElfFile\x00\x00\x00\x00\x00\x00\x00\x00\x00";
    let rep = detect_bytes(evtx_data, &opts).expect("detect evtx");
    assert_eq!(rep.format_id.as_deref(), Some("evtx"));
    assert_eq!(rep.category, "security");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("evtx"))),
        ExtensionStatus::Match
    );

    // 9. Windows Registry Hive (hve)
    let hve_data = b"regf\x00\x00\x00\x00\x00\x00\x00\x00";
    let rep = detect_bytes(hve_data, &opts).expect("detect hve");
    assert_eq!(rep.format_id.as_deref(), Some("hve"));
    assert_eq!(rep.category, "system");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("hve"))),
        ExtensionStatus::Match
    );

    // 10. Advanced Forensic Format (aff)
    let aff_data = b"AFF\x00\x00\x00\x00\x00";
    let rep = detect_bytes(aff_data, &opts).expect("detect aff");
    assert_eq!(rep.format_id.as_deref(), Some("aff"));
    assert_eq!(rep.category, "security");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("aff"))),
        ExtensionStatus::Match
    );

    // 11. Expert Witness Format (ewf / e01)
    let ewf_data = b"EVF\x09\x0d\x0a\xff\x00\x01\x00\x00\x00";
    let rep = detect_bytes(ewf_data, &opts).expect("detect ewf");
    assert_eq!(rep.format_id.as_deref(), Some("ewf"));
    assert_eq!(rep.category, "security");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("e01"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("ewf"))),
        ExtensionStatus::Match
    );

    // 12. Compiled Yara Rules (yac)
    let yac_data = b"YARA\x00\x00\x00\x00";
    let rep = detect_bytes(yac_data, &opts).expect("detect yac");
    assert_eq!(rep.format_id.as_deref(), Some("yac"));
    assert_eq!(rep.category, "security");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("yac"))),
        ExtensionStatus::Match
    );

    // 13. Microsoft XNA Game Studio Binary (xnb)
    let xnb_data = b"XNBw\x05\x01\x00\x00";
    let rep = detect_bytes(xnb_data, &opts).expect("detect xnb");
    assert_eq!(rep.format_id.as_deref(), Some("xnb"));
    assert_eq!(rep.category, "game");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("xnb"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_pro_audio_and_science_batch() {
    let opts = DetectOptions::default();

    // 1. Tascam GigaSampler (gig)
    let gig_data = b"RIFF\x20\x00\x00\x00GIG \x00\x00\x00\x00";
    let rep = detect_bytes(gig_data, &opts).expect("detect gig");
    assert_eq!(rep.format_id.as_deref(), Some("gig"));
    assert_eq!(rep.category, "audio");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("gig"))),
        ExtensionStatus::Match
    );

    // 2. Native Instruments Kontakt Instrument (nki)
    let nki_data = b"\x12\x90\xa8\x7f\x00\x00\x00\x00";
    let rep = detect_bytes(nki_data, &opts).expect("detect nki");
    assert_eq!(rep.format_id.as_deref(), Some("nki"));
    assert_eq!(rep.category, "audio");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("nki"))),
        ExtensionStatus::Match
    );

    // 3. Native Instruments Kore Sound (ksd)
    let ksd_data = b"\x5c\x12\x34\x78\x00\x00\x00\x00";
    let rep = detect_bytes(ksd_data, &opts).expect("detect ksd");
    assert_eq!(rep.format_id.as_deref(), Some("ksd"));
    assert_eq!(rep.category, "audio");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("ksd"))),
        ExtensionStatus::Match
    );

    // 4. VST Plugin Preset (fxp)
    let fxp_data = b"CcnK\x00\x00\x00\x01";
    let rep = detect_bytes(fxp_data, &opts).expect("detect fxp");
    assert_eq!(rep.format_id.as_deref(), Some("fxp"));
    assert_eq!(rep.category, "audio");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("fxp"))),
        ExtensionStatus::Match
    );

    // 5. VST Bank (fxb)
    let fxb_data = b"CbkB\x00\x00\x00\x01";
    let rep = detect_bytes(fxb_data, &opts).expect("detect fxb");
    assert_eq!(rep.format_id.as_deref(), Some("fxb"));
    assert_eq!(rep.category, "audio");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("fxb"))),
        ExtensionStatus::Match
    );

    // 6. Propellerhead Reason ReFill (rfl)
    let rfl_data = b"REASON\x00\x01\x00\x00";
    let rep = detect_bytes(rfl_data, &opts).expect("detect rfl");
    assert_eq!(rep.format_id.as_deref(), Some("rfl"));
    assert_eq!(rep.category, "audio");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("rfl"))),
        ExtensionStatus::Match
    );

    // 7. Korg Synthesizer Program (pcg)
    let pcg_data = b"KORG\x00\x00\x00\x01";
    let rep = detect_bytes(pcg_data, &opts).expect("detect pcg");
    assert_eq!(rep.format_id.as_deref(), Some("pcg"));
    assert_eq!(rep.category, "audio");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("pcg"))),
        ExtensionStatus::Match
    );

    // 8. Audiokinetic Wwise SoundBank (bnk)
    let bnk_data = b"BKHD\x10\x00\x00\x00\x01\x00\x00\x00";
    let rep = detect_bytes(bnk_data, &opts).expect("detect bnk");
    assert_eq!(rep.format_id.as_deref(), Some("bnk"));
    assert_eq!(rep.category, "audio");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("bnk"))),
        ExtensionStatus::Match
    );

    // 9. Unity Web Player Asset (unityweb)
    let unityweb_data = b"UnityWeb\x00\x00\x00\x00\x01";
    let rep = detect_bytes(unityweb_data, &opts).expect("detect unityweb");
    assert_eq!(rep.format_id.as_deref(), Some("unityweb"));
    assert_eq!(rep.category, "game");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("unityweb"))),
        ExtensionStatus::Match
    );

    // 10. ESRI Arc/Info Export Coverage (e00)
    let e00_data = b"EXP  0  /archive/coverage/data";
    let rep = detect_bytes(e00_data, &opts).expect("detect e00");
    assert_eq!(rep.format_id.as_deref(), Some("e00"));
    assert_eq!(rep.category, "gis");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("e00"))),
        ExtensionStatus::Match
    );

    // 11. ENVI Hyperspectral Header (envi)
    let envi_data = b"ENVI\ndescription = {Sample dataset}\nsamples = 100\n";
    let rep = detect_bytes(envi_data, &opts).expect("detect envi");
    assert_eq!(rep.format_id.as_deref(), Some("envi"));
    assert_eq!(rep.category, "gis");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("envi"))),
        ExtensionStatus::Match
    );

    // 12. SEG-2 Engineering Seismograph Data (sg2)
    let sg2_data = b"\x3a\x55\x00\x04\x00\x01";
    let rep = detect_bytes(sg2_data, &opts).expect("detect sg2");
    assert_eq!(rep.format_id.as_deref(), Some("sg2"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("sg2"))),
        ExtensionStatus::Match
    );

    // 13. European Data Format Biomedical (edf)
    let edf_data = b"0       Patient_001_EEG_Record_Header";
    let rep = detect_bytes(edf_data, &opts).expect("detect edf");
    assert_eq!(rep.format_id.as_deref(), Some("edf"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("edf"))),
        ExtensionStatus::Match
    );

    // 14. SAS Statistical Dataset (sas7bdat)
    let mut sas_data = vec![0u8; 32];
    sas_data[12..16].copy_from_slice(b"\xc2\xea\x81\x60");
    let rep = detect_bytes(&sas_data, &opts).expect("detect sas7bdat");
    assert_eq!(rep.format_id.as_deref(), Some("sas7bdat"));
    assert_eq!(rep.category, "database");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("sas7bdat"))),
        ExtensionStatus::Match
    );
}
