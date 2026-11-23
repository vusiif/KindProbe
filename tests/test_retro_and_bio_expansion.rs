use std::ffi::OsStr;
use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Verification};

#[test]
fn test_detect_retro_and_virtualization_batch() {
    let opts = DetectOptions::default();

    // 1. Amiga Disk File (adf)
    let adf_data = b"DOS\x00\x00\x00\x00\x00";
    let rep = detect_bytes(adf_data, &opts).expect("detect adf");
    assert_eq!(rep.format_id.as_deref(), Some("adf"));
    assert_eq!(rep.category, "disk");
    assert_eq!(rep.verification, Verification::Signature);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("adf"))), ExtensionStatus::Match);

    // 2. Amstrad CPC Disk (dsk)
    let dsk_data = b"MV - CPC Format Disk Image\r\n";
    let rep = detect_bytes(dsk_data, &opts).expect("detect dsk");
    assert_eq!(rep.format_id.as_deref(), Some("dsk"));
    assert_eq!(rep.category, "disk");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("dsk"))), ExtensionStatus::Match);

    // 3. ZX Spectrum Tape (tzx)
    let tzx_data = b"ZXTape!\x1a\x01\x20";
    let rep = detect_bytes(tzx_data, &opts).expect("detect tzx");
    assert_eq!(rep.format_id.as_deref(), Some("tzx"));
    assert_eq!(rep.category, "archive");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("tzx"))), ExtensionStatus::Match);

    // 4. Commodore 64 Raw Tape (tap)
    let tap_data = b"C64-TAPE-RAW\x01\x00\x00\x00";
    let rep = detect_bytes(tap_data, &opts).expect("detect tap");
    assert_eq!(rep.format_id.as_deref(), Some("tap"));
    assert_eq!(rep.category, "archive");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("tap"))), ExtensionStatus::Match);

    // 5. MSX Cassette Tape (cas)
    let cas_data = b"\x1f\xa6\xde\xba\xcc\x13\x7d\x74\x00\x00";
    let rep = detect_bytes(cas_data, &opts).expect("detect cas");
    assert_eq!(rep.format_id.as_deref(), Some("cas"));
    assert_eq!(rep.category, "archive");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("cas"))), ExtensionStatus::Match);

    // 6. Atari 8-bit Executable (xex)
    let xex_data = b"\xff\xff\x00\x00\x00\x01";
    let rep = detect_bytes(xex_data, &opts).expect("detect xex");
    assert_eq!(rep.format_id.as_deref(), Some("xex"));
    assert_eq!(rep.category, "executable");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("xex"))), ExtensionStatus::Match);

    // 7. Atari Lynx Cartridge (lnx)
    let lnx_data = b"LYNX\x01\x00\x00\x00";
    let rep = detect_bytes(lnx_data, &opts).expect("detect lnx");
    assert_eq!(rep.format_id.as_deref(), Some("lnx"));
    assert_eq!(rep.category, "game");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("lnx"))), ExtensionStatus::Match);

    // 8. Macintosh HFS Virtual Volume (hfv)
    let hfv_data = b"\x42\x44\x00\x00\x00\x00";
    let rep = detect_bytes(hfv_data, &opts).expect("detect hfv");
    assert_eq!(rep.format_id.as_deref(), Some("hfv"));
    assert_eq!(rep.category, "disk");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("hfv"))), ExtensionStatus::Match);

    // 9. OctaMED Amiga Tracker (med)
    let med_data = b"MED\x03\x00\x00\x00\x00";
    let rep = detect_bytes(med_data, &opts).expect("detect med");
    assert_eq!(rep.format_id.as_deref(), Some("med"));
    assert_eq!(rep.category, "audio");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("med"))), ExtensionStatus::Match);

    // 10. Reality Adlib Tracker (rad)
    let rad_data = b"RAD by REALITY!!\x10\x00";
    let rep = detect_bytes(rad_data, &opts).expect("detect rad");
    assert_eq!(rep.format_id.as_deref(), Some("rad"));
    assert_eq!(rep.category, "audio");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("rad"))), ExtensionStatus::Match);

    // 11. Doom Classic Music (mus)
    let mus_data = b"MUS\x1a\x10\x00\x00\x00";
    let rep = detect_bytes(mus_data, &opts).expect("detect mus");
    assert_eq!(rep.format_id.as_deref(), Some("mus"));
    assert_eq!(rep.category, "audio");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("mus"))), ExtensionStatus::Match);

    // 12. Xbox Executable (xbe)
    let xbe_data = b"XBEH\x01\x00\x00\x00";
    let rep = detect_bytes(xbe_data, &opts).expect("detect xbe");
    assert_eq!(rep.format_id.as_deref(), Some("xbe"));
    assert_eq!(rep.category, "executable");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("xbe"))), ExtensionStatus::Match);

    // 13. Nintendo Switch Package (nsp)
    let nsp_data = b"PFS0\x04\x00\x00\x00";
    let rep = detect_bytes(nsp_data, &opts).expect("detect nsp");
    assert_eq!(rep.format_id.as_deref(), Some("nsp"));
    assert_eq!(rep.category, "game");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("nsp"))), ExtensionStatus::Match);
}

#[test]
fn test_detect_bio_and_creative_design_batch() {
    let opts = DetectOptions::default();

    // 1. NIfTI-1 Neuroimaging (nii)
    let mut nii_data = vec![0u8; 352];
    nii_data[344..348].copy_from_slice(b"n+1\x00");
    let rep = detect_bytes(&nii_data, &opts).expect("detect nii");
    assert_eq!(rep.format_id.as_deref(), Some("nii"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("nii"))), ExtensionStatus::Match);

    // 2. Applied Biosystems Genetic Analyzer Chromatogram (ab1)
    let ab1_data = b"ABIF\x01\x00\x00\x00";
    let rep = detect_bytes(ab1_data, &opts).expect("detect ab1");
    assert_eq!(rep.format_id.as_deref(), Some("ab1"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("ab1"))), ExtensionStatus::Match);

    // 3. Flow Cytometry Standard (fcs)
    let fcs_data = b"FCS3.0  header bytes here";
    let rep = detect_bytes(fcs_data, &opts).expect("detect fcs");
    assert_eq!(rep.format_id.as_deref(), Some("fcs"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("fcs"))), ExtensionStatus::Match);

    // 4. ChemDraw Chemical Structure (cdx)
    let cdx_data = b"VjCD0100\x00\x00\x00\x00";
    let rep = detect_bytes(cdx_data, &opts).expect("detect cdx");
    assert_eq!(rep.format_id.as_deref(), Some("cdx"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("cdx"))), ExtensionStatus::Match);

    // 5. Tripos Sybyl Mol2 Molecular Structure (mol2)
    let mol2_data = b"@<TRIPOS>MOLECULE\nCompound_001\n";
    let rep = detect_bytes(mol2_data, &opts).expect("detect mol2");
    assert_eq!(rep.format_id.as_deref(), Some("mol2"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("mol2"))), ExtensionStatus::Match);

    // 6. Adobe Swatch Exchange (ase)
    let ase_data = b"ASEF\x00\x01\x00\x00";
    let rep = detect_bytes(ase_data, &opts).expect("detect ase");
    assert_eq!(rep.format_id.as_deref(), Some("ase"));
    assert_eq!(rep.category, "image");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("ase"))), ExtensionStatus::Match);

    // 7. Adobe Photoshop Pattern (pat)
    let pat_data = b"8BPT\x00\x01\x00\x00";
    let rep = detect_bytes(pat_data, &opts).expect("detect pat");
    assert_eq!(rep.format_id.as_deref(), Some("pat"));
    assert_eq!(rep.category, "image");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("pat"))), ExtensionStatus::Match);

    // 8. Corel PHOTO-PAINT (cpt)
    let cpt_data = b"CPTFILE\x00\x00\x00\x00";
    let rep = detect_bytes(cpt_data, &opts).expect("detect cpt");
    assert_eq!(rep.format_id.as_deref(), Some("cpt"));
    assert_eq!(rep.category, "image");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("cpt"))), ExtensionStatus::Match);

    // 9. Affinity Designer (afdesign)
    let afdesign_data = b"\x00\xff\x00\x00\x00\x00\x00\x00T3D\x01\x00";
    let rep = detect_bytes(afdesign_data, &opts).expect("detect afdesign");
    assert_eq!(rep.format_id.as_deref(), Some("afdesign"));
    assert_eq!(rep.category, "image");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("afdesign"))), ExtensionStatus::Match);

    // 10. Macromedia FreeHand (fh)
    let fh_data = b"AGGD\x00\x01\x00\x00";
    let rep = detect_bytes(fh_data, &opts).expect("detect fh");
    assert_eq!(rep.format_id.as_deref(), Some("fh"));
    assert_eq!(rep.category, "image");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("fh"))), ExtensionStatus::Match);

    // 11. AOL ART Image (art)
    let art_data = b"JG\x04\x0e\x00\x00\x00\x00";
    let rep = detect_bytes(art_data, &opts).expect("detect art");
    assert_eq!(rep.format_id.as_deref(), Some("art"));
    assert_eq!(rep.category, "image");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("art"))), ExtensionStatus::Match);

    // 12. WordPerfect Graphics (wpg)
    let wpg_data = b"\xffWPC\x10\x00\x00\x00";
    let rep = detect_bytes(wpg_data, &opts).expect("detect wpg");
    assert_eq!(rep.format_id.as_deref(), Some("wpg"));
    assert_eq!(rep.category, "image");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("wpg"))), ExtensionStatus::Match);

    // 13. CALS Raster Military Image (cals / cal)
    let cals_data = b"srcdocid:MIL-R-28002A-SAMPLE";
    let rep = detect_bytes(cals_data, &opts).expect("detect cals");
    assert_eq!(rep.format_id.as_deref(), Some("cals"));
    assert_eq!(rep.category, "image");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("cals"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("cal"))), ExtensionStatus::Match);
}
