use std::ffi::OsStr;
use kindprobe::budget::DetectOptions;
use kindprobe::detect_bytes;
use kindprobe::extension::compare_extension;
use kindprobe::model::ExtensionStatus;

#[test]
fn test_detect_game_assets_and_engines() {
    let opts = DetectOptions::default();

    // 1. Doom WAD (IWAD & PWAD)
    let mut iwad = vec![0u8; 32];
    iwad[0..4].copy_from_slice(b"IWAD");
    let rep_iwad = detect_bytes(&iwad, &opts).expect("detect iwad");
    assert_eq!(rep_iwad.format_id.as_deref(), Some("wad"));
    assert_eq!(compare_extension(&rep_iwad, Some(OsStr::new("wad"))), ExtensionStatus::Match);

    let mut pwad = vec![0u8; 32];
    pwad[0..4].copy_from_slice(b"PWAD");
    let rep_pwad = detect_bytes(&pwad, &opts).expect("detect pwad");
    assert_eq!(rep_pwad.format_id.as_deref(), Some("wad"));

    // 2. Quake PAK
    let mut pak = vec![0u8; 32];
    pak[0..4].copy_from_slice(b"PACK");
    let rep_pak = detect_bytes(&pak, &opts).expect("detect pak");
    assert_eq!(rep_pak.format_id.as_deref(), Some("pak"));
    assert_eq!(compare_extension(&rep_pak, Some(OsStr::new("pak"))), ExtensionStatus::Match);

    // 3. Godot Engine Package (PCK)
    let mut pck = vec![0u8; 32];
    pck[0..4].copy_from_slice(b"GDPC");
    let rep_pck = detect_bytes(&pck, &opts).expect("detect pck");
    assert_eq!(rep_pck.format_id.as_deref(), Some("pck"));
    assert_eq!(compare_extension(&rep_pck, Some(OsStr::new("pck"))), ExtensionStatus::Match);

    // 4. Unity AssetBundle (UnityFS)
    let mut unity = vec![0u8; 32];
    unity[0..7].copy_from_slice(b"UnityFS");
    let rep_unity = detect_bytes(&unity, &opts).expect("detect unity");
    assert_eq!(rep_unity.format_id.as_deref(), Some("unity3d"));
    assert_eq!(compare_extension(&rep_unity, Some(OsStr::new("unity3d"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_unity, Some(OsStr::new("bundle"))), ExtensionStatus::Match);

    // 5. Valve PacKage (VPK)
    let mut vpk = vec![0u8; 32];
    vpk[0..4].copy_from_slice(b"\x34\x12\xaa\x55");
    let rep_vpk = detect_bytes(&vpk, &opts).expect("detect vpk");
    assert_eq!(rep_vpk.format_id.as_deref(), Some("vpk"));
    assert_eq!(compare_extension(&rep_vpk, Some(OsStr::new("vpk"))), ExtensionStatus::Match);

    // 6. EA Games BIG
    let mut big = vec![0u8; 32];
    big[0..4].copy_from_slice(b"BIG4");
    let rep_big = detect_bytes(&big, &opts).expect("detect big");
    assert_eq!(rep_big.format_id.as_deref(), Some("big"));
    assert_eq!(compare_extension(&rep_big, Some(OsStr::new("big"))), ExtensionStatus::Match);

    // 7. Build Engine GRP
    let mut grp = vec![0u8; 32];
    grp[0..12].copy_from_slice(b"KenSilverman");
    let rep_grp = detect_bytes(&grp, &opts).expect("detect grp");
    assert_eq!(rep_grp.format_id.as_deref(), Some("grp"));
    assert_eq!(compare_extension(&rep_grp, Some(OsStr::new("grp"))), ExtensionStatus::Match);
}

#[test]
fn test_detect_robotics_and_firmware() {
    let opts = DetectOptions::default();

    // 1. Foxglove MCAP
    let mut mcap = vec![0u8; 32];
    mcap[0..8].copy_from_slice(b"\x89MCAP0\r\n");
    let rep_mcap = detect_bytes(&mcap, &opts).expect("detect mcap");
    assert_eq!(rep_mcap.format_id.as_deref(), Some("mcap"));
    assert_eq!(rep_mcap.category, "scientific");
    assert_eq!(compare_extension(&rep_mcap, Some(OsStr::new("mcap"))), ExtensionStatus::Match);

    // 2. ROS Bag V2.0
    let mut bag = vec![0u8; 32];
    bag[0..13].copy_from_slice(b"#ROSBAG V2.0\n");
    let rep_bag = detect_bytes(&bag, &opts).expect("detect bag");
    assert_eq!(rep_bag.format_id.as_deref(), Some("bag"));
    assert_eq!(rep_bag.category, "scientific");
    assert_eq!(compare_extension(&rep_bag, Some(OsStr::new("bag"))), ExtensionStatus::Match);

    // 3. Point Cloud Data (PCD)
    let mut pcd = vec![0u8; 32];
    pcd[0..8].copy_from_slice(b"# .PCD v");
    let rep_pcd = detect_bytes(&pcd, &opts).expect("detect pcd");
    assert_eq!(rep_pcd.format_id.as_deref(), Some("pcd"));
    assert_eq!(rep_pcd.category, "scientific");
    assert_eq!(compare_extension(&rep_pcd, Some(OsStr::new("pcd"))), ExtensionStatus::Match);

    // 4. Flattened Device Tree (DTB)
    let mut dtb = vec![0u8; 32];
    dtb[0..4].copy_from_slice(b"\xd0\x0d\xfe\xed");
    let rep_dtb = detect_bytes(&dtb, &opts).expect("detect dtb");
    assert_eq!(rep_dtb.format_id.as_deref(), Some("dtb"));
    assert_eq!(rep_dtb.category, "system");
    assert_eq!(compare_extension(&rep_dtb, Some(OsStr::new("dtb"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_dtb, Some(OsStr::new("dtbo"))), ExtensionStatus::Match);

    // 5. Das U-Boot Image (uImage)
    let mut uimage = vec![0u8; 32];
    uimage[0..4].copy_from_slice(b"\x27\x05\x19\x56");
    let rep_uimage = detect_bytes(&uimage, &opts).expect("detect uimage");
    assert_eq!(rep_uimage.format_id.as_deref(), Some("uimage"));
    assert_eq!(rep_uimage.category, "system");
    assert_eq!(compare_extension(&rep_uimage, Some(OsStr::new("uimage"))), ExtensionStatus::Match);

    // 6. CRAM Alignment
    let mut cram = vec![0u8; 32];
    cram[0..4].copy_from_slice(b"CRAM");
    let rep_cram = detect_bytes(&cram, &opts).expect("detect cram");
    assert_eq!(rep_cram.format_id.as_deref(), Some("cram"));
    assert_eq!(rep_cram.category, "scientific");
    assert_eq!(compare_extension(&rep_cram, Some(OsStr::new("cram"))), ExtensionStatus::Match);

    // 7. Binary VCF (BCF)
    let mut bcf = vec![0u8; 32];
    bcf[0..4].copy_from_slice(b"BCF\x02");
    let rep_bcf = detect_bytes(&bcf, &opts).expect("detect bcf");
    assert_eq!(rep_bcf.format_id.as_deref(), Some("bcf"));
    assert_eq!(rep_bcf.category, "scientific");
    assert_eq!(compare_extension(&rep_bcf, Some(OsStr::new("bcf"))), ExtensionStatus::Match);
}

#[test]
fn test_detect_fonts_and_vector_fig() {
    let opts = DetectOptions::default();

    // 1. Compact Font Format (CFF)
    let mut cff = vec![0u8; 32];
    cff[0..4].copy_from_slice(b"\x01\x00\x04\x01");
    let rep_cff = detect_bytes(&cff, &opts).expect("detect cff");
    assert_eq!(rep_cff.format_id.as_deref(), Some("cff"));
    assert_eq!(rep_cff.category, "font");
    assert_eq!(compare_extension(&rep_cff, Some(OsStr::new("cff"))), ExtensionStatus::Match);

    // 2. Windows Font Resource (FNT)
    let mut fnt = vec![0u8; 32];
    fnt[0..4].copy_from_slice(b"\x00\x03\x00\x00");
    let rep_fnt = detect_bytes(&fnt, &opts).expect("detect fnt");
    assert_eq!(rep_fnt.format_id.as_deref(), Some("fnt"));
    assert_eq!(rep_fnt.category, "font");
    assert_eq!(compare_extension(&rep_fnt, Some(OsStr::new("fnt"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_fnt, Some(OsStr::new("fon"))), ExtensionStatus::Match);

    // 3. Xfig Vector Drawing (FIG)
    let mut fig = vec![0u8; 32];
    fig[0..5].copy_from_slice(b"#FIG ");
    let rep_fig = detect_bytes(&fig, &opts).expect("detect fig");
    assert_eq!(rep_fig.format_id.as_deref(), Some("fig"));
    assert_eq!(rep_fig.category, "image");
    assert_eq!(compare_extension(&rep_fig, Some(OsStr::new("fig"))), ExtensionStatus::Match);
}
