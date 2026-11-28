use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Verification};
use std::ffi::OsStr;

#[test]
fn test_detect_automotive_and_eda_batch() {
    let opts = DetectOptions::default();

    // 1. ASAM MDF4 Measurement Data Format (mdf / mf4)
    let mdf_data = b"MDF4    \x00\x00\x00\x00";
    let rep = detect_bytes(mdf_data, &opts).expect("detect mdf");
    assert_eq!(rep.format_id.as_deref(), Some("mdf"));
    assert_eq!(rep.category, "database");
    assert_eq!(rep.verification, Verification::Signature);
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("mdf"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("mf4"))),
        ExtensionStatus::Match
    );

    // 2. Value Change Dump (vcd)
    let vcd_data = b"$timescale 1ns $end\n$scope module top $end\n";
    let rep = detect_bytes(vcd_data, &opts).expect("detect vcd");
    assert_eq!(rep.format_id.as_deref(), Some("vcd"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("vcd"))),
        ExtensionStatus::Match
    );

    // 3. Fast Signal Database (fsdb)
    let fsdb_data = b"FSDB\x00\x01\x00\x00";
    let rep = detect_bytes(fsdb_data, &opts).expect("detect fsdb");
    assert_eq!(rep.format_id.as_deref(), Some("fsdb"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("fsdb"))),
        ExtensionStatus::Match
    );

    // 4. Electronic Design Interchange Format (edif)
    let edif_data = b"(edif test_circuit (edifVersion 2 0 0))\n";
    let rep = detect_bytes(edif_data, &opts).expect("detect edif");
    assert_eq!(rep.format_id.as_deref(), Some("edif"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("edif"))),
        ExtensionStatus::Match
    );

    // 5. ASAM MCD-2 MC Calibration File (a2l)
    let a2l_data = b"ASAP2_VERSION 1 71\n/begin PROJECT sample \"\"\n";
    let rep = detect_bytes(a2l_data, &opts).expect("detect a2l");
    assert_eq!(rep.format_id.as_deref(), Some("a2l"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("a2l"))),
        ExtensionStatus::Match
    );

    // 6. Serial Vector Format (svf)
    let svf_data = b"FREQUENCY 1000000 HZ;\nTRST OFF;\n";
    let rep = detect_bytes(svf_data, &opts).expect("detect svf");
    assert_eq!(rep.format_id.as_deref(), Some("svf"));
    assert_eq!(rep.category, "hardware");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("svf"))),
        ExtensionStatus::Match
    );

    // 7. Xilinx FPGA Bitstream (bit)
    let bit_data = b"\x00\x09\x0f\xf0\x0f\xf0\x0f\xf0\x0f\xf0\x00\x00\x01\x61\x00\x0b";
    let rep = detect_bytes(bit_data, &opts).expect("detect bit");
    assert_eq!(rep.format_id.as_deref(), Some("bit"));
    assert_eq!(rep.category, "hardware");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("bit"))),
        ExtensionStatus::Match
    );

    // 8. CAN Database Network Description (dbc)
    let dbc_data = b"VERSION \"\"\n\nNS_ :\nBS_:\n";
    let rep = detect_bytes(dbc_data, &opts).expect("detect dbc");
    assert_eq!(rep.format_id.as_deref(), Some("dbc"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("dbc"))),
        ExtensionStatus::Match
    );

    // 9. LIN Description File (ldf)
    let ldf_data = b"LIN_description_file;\nLIN_protocol_version = \"2.1\";\n";
    let rep = detect_bytes(ldf_data, &opts).expect("detect ldf");
    assert_eq!(rep.format_id.as_deref(), Some("ldf"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("ldf"))),
        ExtensionStatus::Match
    );

    // 10. Boundary Scan Description Language (bsdl / bsd)
    let bsdl_data =
        b"entity sample_chip is\n  generic (PHYSICAL_STYLE : string);\nend sample_chip;\n";
    let rep = detect_bytes(bsdl_data, &opts).expect("detect bsdl");
    assert_eq!(rep.format_id.as_deref(), Some("bsdl"));
    assert_eq!(rep.category, "hardware");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("bsd"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("bsdl"))),
        ExtensionStatus::Match
    );

    // 11. JEDEC Fuse Map (jed)
    let jed_data = b"\x02\r\nNOTE JEDEC LOGIC DEVICE FUSE MAP*\r\nQF1024*\r\n";
    let rep = detect_bytes(jed_data, &opts).expect("detect jed");
    assert_eq!(rep.format_id.as_deref(), Some("jed"));
    assert_eq!(rep.category, "hardware");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("jed"))),
        ExtensionStatus::Match
    );
}
