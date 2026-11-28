use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Identification, Verification};
use std::ffi::OsStr;

#[test]
fn test_detect_3d_models_and_cad() {
    let opts = DetectOptions::default();

    // 1. Blender 3D project
    let blend_bytes = b"BLENDER_v293\x00\x00\x00\x00";
    let rep_blend = detect_bytes(blend_bytes, &opts).expect("detect blend");
    assert_eq!(rep_blend.identification, Identification::Format);
    assert_eq!(rep_blend.format_id.as_deref(), Some("blend"));
    assert_eq!(rep_blend.category, "model");
    assert_eq!(rep_blend.verification, Verification::Signature);
    assert_eq!(
        compare_extension(&rep_blend, Some(OsStr::new("blend"))),
        ExtensionStatus::Match
    );

    // 2. glTF 2.0 Binary (GLB)
    let glb_bytes = b"glTF\x02\x00\x00\x00\x00\x00\x01\x00\x00\x00\x00\x00";
    let rep_glb = detect_bytes(glb_bytes, &opts).expect("detect glb");
    assert_eq!(rep_glb.format_id.as_deref(), Some("glb"));
    assert_eq!(rep_glb.category, "model");
    assert_eq!(
        compare_extension(&rep_glb, Some(OsStr::new("glb"))),
        ExtensionStatus::Match
    );

    // 3. Autodesk FBX Binary
    let fbx_bytes = b"Kaydara FBX Binary  \x00\x1a\x00\x00\x00\x00";
    let rep_fbx = detect_bytes(fbx_bytes, &opts).expect("detect fbx");
    assert_eq!(rep_fbx.format_id.as_deref(), Some("fbx"));
    assert_eq!(rep_fbx.category, "model");

    // 4. 3D Studio (3DS)
    let d3s_bytes = b"\x4d\x4d\x02\x00\x00\x00";
    let rep_3ds = detect_bytes(d3s_bytes, &opts).expect("detect 3ds");
    assert_eq!(rep_3ds.format_id.as_deref(), Some("3ds"));

    // 5. Polygon File Format (PLY)
    let ply_bytes = b"ply\nformat ascii 1.0\ncomment test\n";
    let rep_ply = detect_bytes(ply_bytes, &opts).expect("detect ply");
    assert_eq!(rep_ply.format_id.as_deref(), Some("ply"));

    // 6. STEP CAD
    let step_bytes = b"ISO-10303-21;\nHEADER;\nFILE_DESCRIPTION(('Test'),'2;1');\n";
    let rep_step = detect_bytes(step_bytes, &opts).expect("detect step");
    assert_eq!(rep_step.format_id.as_deref(), Some("step"));
    assert_eq!(
        compare_extension(&rep_step, Some(OsStr::new("stp"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_step, Some(OsStr::new("step"))),
        ExtensionStatus::Match
    );

    // 7. LightWave 3D Object (LWO)
    let lwo_bytes = b"FORM\x00\x00\x00\x20LWO2TAGS\x00\x00\x00\x04";
    let rep_lwo = detect_bytes(lwo_bytes, &opts).expect("detect lwo");
    assert_eq!(rep_lwo.format_id.as_deref(), Some("lwo"));
    assert_eq!(rep_lwo.category, "model");
    assert_eq!(
        compare_extension(&rep_lwo, Some(OsStr::new("lwo"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_scientific_and_geospatial() {
    let opts = DetectOptions::default();

    // 1. HDF5 hierarchical data
    let hdf5_bytes = b"\x89HDF\r\n\x1a\n\x00\x00\x00\x00";
    let rep_h5 = detect_bytes(hdf5_bytes, &opts).expect("detect hdf5");
    assert_eq!(rep_h5.format_id.as_deref(), Some("hdf5"));
    assert_eq!(rep_h5.category, "scientific");
    assert_eq!(
        compare_extension(&rep_h5, Some(OsStr::new("h5"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_h5, Some(OsStr::new("hdf5"))),
        ExtensionStatus::Match
    );

    // 2. NetCDF
    let cdf_bytes = b"CDF\x01\x00\x00\x00\x00\x00\x00\x00";
    let rep_nc = detect_bytes(cdf_bytes, &opts).expect("detect netcdf");
    assert_eq!(rep_nc.format_id.as_deref(), Some("netcdf"));
    assert_eq!(
        compare_extension(&rep_nc, Some(OsStr::new("nc"))),
        ExtensionStatus::Match
    );

    // 3. ESRI Shapefile
    let shp_bytes = b"\x00\x00\x27\x0a\x00\x00\x00\x00\x00\x00\x00\x00";
    let rep_shp = detect_bytes(shp_bytes, &opts).expect("detect shp");
    assert_eq!(rep_shp.format_id.as_deref(), Some("shp"));

    // 4. GRIB weather data
    let grib_bytes = b"GRIB\x00\x00\x01\x02";
    let rep_grib = detect_bytes(grib_bytes, &opts).expect("detect grib");
    assert_eq!(rep_grib.format_id.as_deref(), Some("grib"));

    // 5. MATLAB v5 Data
    let mat_bytes = b"MATLAB 5.0 MAT-file, Platform: PCWIN64, Created on: Sat Nov 21 2026";
    let rep_mat = detect_bytes(mat_bytes, &opts).expect("detect mat");
    assert_eq!(rep_mat.format_id.as_deref(), Some("mat5"));
    assert_eq!(
        compare_extension(&rep_mat, Some(OsStr::new("mat"))),
        ExtensionStatus::Match
    );

    // 6. CERN ROOT High-Energy Physics
    let root_bytes = b"root\x00\x00\x01\x00\x00\x00\x00\x00";
    let rep_root = detect_bytes(root_bytes, &opts).expect("detect root");
    assert_eq!(rep_root.format_id.as_deref(), Some("root"));

    // 7. MRC Cryo-EM map (offset 208)
    let mut mrc_bytes = vec![0u8; 208];
    mrc_bytes.extend_from_slice(b"MAP \x00\x00\x00\x00");
    let rep_mrc = detect_bytes(&mrc_bytes, &opts).expect("detect mrc");
    assert_eq!(rep_mrc.format_id.as_deref(), Some("mrc"));
}

#[test]
fn test_detect_audio_chips_and_archives() {
    let opts = DetectOptions::default();

    // 1. Commodore 64 SID
    let sid_bytes = b"PSID\x00\x02\x00\x7c\x00\x00\x00\x00";
    let rep_sid = detect_bytes(sid_bytes, &opts).expect("detect sid");
    assert_eq!(rep_sid.format_id.as_deref(), Some("sid"));
    assert_eq!(rep_sid.category, "audio");

    // 2. SNES SPC700
    let spc_bytes = b"SNES-SPC700 sound file data v0.30\x1a\x1a";
    let rep_spc = detect_bytes(spc_bytes, &opts).expect("detect spc");
    assert_eq!(rep_spc.format_id.as_deref(), Some("spc"));

    // 3. BitTorrent Metainfo
    let torrent_bytes =
        b"d8:announce41:http://tracker.example.com/announce13:creation datei1600000000ee";
    let rep_torrent = detect_bytes(torrent_bytes, &opts).expect("detect torrent");
    assert_eq!(rep_torrent.format_id.as_deref(), Some("torrent"));

    // 4. Windows Imaging Format (WIM)
    let wim_bytes = b"MSWIM\x00\x00\x00\xd0\x00\x00\x00\x00\x01\x00\x00";
    let rep_wim = detect_bytes(wim_bytes, &opts).expect("detect wim");
    assert_eq!(rep_wim.format_id.as_deref(), Some("wim"));
    assert_eq!(rep_wim.category, "archive");
    assert_eq!(
        compare_extension(&rep_wim, Some(OsStr::new("wim"))),
        ExtensionStatus::Match
    );

    // 5. DjVu Document Image (multistage magic)
    let djvu_bytes = b"AT&TFORM\x00\x00\x00\x10DJVUINFO\x00\x00\x00\x0a";
    let rep_djvu = detect_bytes(djvu_bytes, &opts).expect("detect djvu");
    assert_eq!(rep_djvu.format_id.as_deref(), Some("djvu"));
    assert_eq!(rep_djvu.category, "image");
    assert_eq!(
        compare_extension(&rep_djvu, Some(OsStr::new("djvu"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_djvu, Some(OsStr::new("djv"))),
        ExtensionStatus::Match
    );

    // 6. NITF Satellite Image
    let nitf_bytes = b"NITF02.10\x00\x00\x00\x00";
    let rep_nitf = detect_bytes(nitf_bytes, &opts).expect("detect nitf");
    assert_eq!(rep_nitf.format_id.as_deref(), Some("nitf"));
    assert_eq!(
        compare_extension(&rep_nitf, Some(OsStr::new("ntf"))),
        ExtensionStatus::Match
    );

    // 7. StuffIt Mac Archive
    let sit_bytes = b"StuffIt (c)1997-1998 Aladdin Systems, Inc.";
    let rep_sit = detect_bytes(sit_bytes, &opts).expect("detect sit");
    assert_eq!(rep_sit.format_id.as_deref(), Some("sit"));
}
