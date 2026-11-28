use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Verification};
use std::ffi::OsStr;

#[test]
fn test_detect_batch_800_industrial_and_legacy() {
    let opts = DetectOptions::default();

    // 1. Siemens Step 7 Project
    let s7p = b"S7P\x00\x01\x02\x03";
    let rep = detect_bytes(s7p, &opts).expect("detect s7p");
    assert_eq!(rep.format_id.as_deref(), Some("s7p"));
    assert_eq!(rep.category, "engineering");
    assert_eq!(rep.verification, Verification::Signature);
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("s7p"))),
        ExtensionStatus::Match
    );

    // 2. Rockwell APA Project Archive
    let apa = b"APA_ARC_BACKUP";
    let rep = detect_bytes(apa, &opts).expect("detect apa");
    assert_eq!(rep.format_id.as_deref(), Some("apa"));
    assert_eq!(rep.category, "engineering");

    // 3. Omron CX-Programmer
    let cxp = b"OMRON CX-Programmer Project File";
    let rep = detect_bytes(cxp, &opts).expect("detect cxp");
    assert_eq!(rep.format_id.as_deref(), Some("cxp"));
    assert_eq!(rep.category, "engineering");

    // 4. Hercules Tape
    let het = b"HET\x01\x00\x00\x00";
    let rep = detect_bytes(het, &opts).expect("detect het");
    assert_eq!(rep.format_id.as_deref(), Some("het"));
    assert_eq!(rep.category, "virtualization");

    // 5. Atari SIO Disk Image
    let atr = b"\x96\x02\x80\x16";
    let rep = detect_bytes(atr, &opts).expect("detect atr");
    assert_eq!(rep.format_id.as_deref(), Some("atr"));
    assert_eq!(rep.category, "disk_image");
}

#[test]
fn test_detect_batch_800_physics_and_video() {
    let opts = DetectOptions::default();

    // 1. CERN ROOT
    let root = b"root\x00\x00\x01\x00";
    let rep = detect_bytes(root, &opts).expect("detect root");
    assert_eq!(rep.format_id.as_deref(), Some("root"));
    assert_eq!(rep.category, "scientific");

    // 2. Mascot Generic Format
    let mgf = b"BEGIN IONS\nPEPMASS=543.21\nCHARGE=2+\n";
    let rep = detect_bytes(mgf, &opts).expect("detect mgf");
    assert_eq!(rep.format_id.as_deref(), Some("mgf"));
    assert_eq!(rep.category, "scientific");

    // 3. 3D LUT Cube
    let cube = b"LUT_3D_SIZE 33\n0.0 0.0 0.0\n";
    let rep = detect_bytes(cube, &opts).expect("detect cube");
    assert_eq!(rep.format_id.as_deref(), Some("cube"));
    assert_eq!(rep.category, "video");

    // 4. Adobe After Effects Project (RIFX Egg!)
    let aep = b"RIFX\x00\x00\x00\x10Egg!DATA";
    let rep = detect_bytes(aep, &opts).expect("detect aep");
    assert_eq!(rep.format_id.as_deref(), Some("aep"));
    assert_eq!(rep.category, "video");

    // 5. OpenTimelineIO
    let otio = b"\"OTIO_SCHEMA\": \"Timeline.1\"\n";
    let rep = detect_bytes(otio, &opts).expect("detect otio");
    assert_eq!(rep.format_id.as_deref(), Some("otio"));
    assert_eq!(rep.category, "video");
}

#[test]
fn test_detect_batch_800_eda_and_sensors() {
    let opts = DetectOptions::default();

    // 1. WMO BUFR
    let bufr = b"BUFR\x00\x00\x00\x20";
    let rep = detect_bytes(bufr, &opts).expect("detect bufr");
    assert_eq!(rep.format_id.as_deref(), Some("bufr"));
    assert_eq!(rep.category, "scientific");

    // 2. ASTM E57 3D LiDAR
    let e57 = b"ASTM-E57 1.0\x00\x00";
    let rep = detect_bytes(e57, &opts).expect("detect e57");
    assert_eq!(rep.format_id.as_deref(), Some("e57"));
    assert_eq!(rep.category, "scientific");

    // 3. u-blox GNSS Binary Protocol
    let ubx = b"\xb5\x62\x01\x02\x1c\x00";
    let rep = detect_bytes(ubx, &opts).expect("detect ubx");
    assert_eq!(rep.format_id.as_deref(), Some("ubx"));
    assert_eq!(rep.category, "location");

    // 4. Calma GDSII Layout
    let gds = b"\x00\x06\x00\x02\x00\x07";
    let rep = detect_bytes(gds, &opts).expect("detect gdsii");
    assert_eq!(rep.format_id.as_deref(), Some("gdsii"));
    assert_eq!(rep.category, "engineering");

    // 5. SEMI OASIS Layout
    let oasis = b"%START-OASIS\r\n1.0\r\n";
    let rep = detect_bytes(oasis, &opts).expect("detect oasis");
    assert_eq!(rep.format_id.as_deref(), Some("oasis"));
    assert_eq!(rep.category, "engineering");
}

#[test]
fn test_detect_batch_800_systems_and_codecs() {
    let opts = DetectOptions::default();

    // 1. TMSi Poly5 Biosignals
    let poly5 = b"POLY SAMPLE FILEversion 2.03";
    let rep = detect_bytes(poly5, &opts).expect("detect tmsi_poly5");
    assert_eq!(rep.format_id.as_deref(), Some("tmsi_poly5"));
    assert_eq!(rep.category, "scientific");

    // 2. Android HCI Bluetooth Snoop
    let btsnoop = b"btsnoop\x00\x00\x00\x00\x01";
    let rep = detect_bytes(btsnoop, &opts).expect("detect btsnoop");
    assert_eq!(rep.format_id.as_deref(), Some("btsnoop"));
    assert_eq!(rep.category, "telecom");

    // 3. Valve Source Engine BSP
    let bsp = b"VBSP\x14\x00\x00\x00";
    let rep = detect_bytes(bsp, &opts).expect("detect source_bsp");
    assert_eq!(rep.format_id.as_deref(), Some("source_bsp"));
    assert_eq!(rep.category, "game_asset");

    // 4. ClamAV Virus Database Container
    let cvd = b"ClamAV-VDB:26-Nov-2026";
    let rep = detect_bytes(cvd, &opts).expect("detect clamav_cvd");
    assert_eq!(rep.format_id.as_deref(), Some("clamav_cvd"));
    assert_eq!(rep.category, "security");

    // 5. Khronos SPIR-V Shader
    let spv = b"\x03\x02\x23\x07\x00\x00\x01\x00";
    let rep = detect_bytes(spv, &opts).expect("detect spirv_shader");
    assert_eq!(rep.format_id.as_deref(), Some("spirv_shader"));
    assert_eq!(rep.category, "code");

    // 6. Linux CramFS ROM Image
    let cramfs = b"\x28\xcd\x3d\x45\x00\x00\x10\x00";
    let rep = detect_bytes(cramfs, &opts).expect("detect cramfs_img");
    assert_eq!(rep.format_id.as_deref(), Some("cramfs_img"));
    assert_eq!(rep.category, "disk");
}
