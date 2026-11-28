use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_path, ExtensionStatus, Identification, Verification};
use std::ffi::OsStr;
use std::path::PathBuf;

#[test]
fn test_real_fixtures_file_handle_matrix() {
    let opts = DetectOptions::default();
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("real");

    let matrix: &[(&str, &str, &str, Verification)] = &[
        ("sample.flv", "flv", "video", Verification::Signature),
        ("sample.amr", "amr", "audio", Verification::Signature),
        ("sample.ac3", "ac3", "audio", Verification::Signature),
        ("sample.caf", "caf", "audio", Verification::Signature),
        ("sample.drc", "drc", "video", Verification::Signature),
        ("sample.mng", "mng", "video", Verification::Signature),
        ("sample.xcf", "xcf", "image", Verification::Signature),
        ("sample.exr", "exr", "image", Verification::Signature),
        ("sample.fits", "fits", "image", Verification::Signature),
        ("sample.icns", "icns", "image", Verification::Signature),
        ("sample.jp2", "jp2", "image", Verification::Signature),
        ("sample.bpg", "bpg", "image", Verification::Signature),
        ("sample.pbm", "pbm", "image", Verification::Signature),
        ("sample.pgm", "pgm", "image", Verification::Signature),
        ("sample.ppm", "ppm", "image", Verification::Signature),
        ("sample.dcm", "dicom", "document", Verification::Signature),
        (
            "sample.ps",
            "postscript",
            "document",
            Verification::Signature,
        ),
        ("sample.rtf", "rtf", "document", Verification::Signature),
        ("sample.dwg", "dwg", "document", Verification::Signature),
        ("sample.cpio", "cpio", "archive", Verification::Signature),
        ("sample.lz", "lz", "archive", Verification::Signature),
        ("sample.cab", "cab", "archive", Verification::Signature),
        (
            "sample.sqlite3",
            "sqlite3",
            "database",
            Verification::Structure,
        ),
        ("sample.pcap", "pcap", "capture", Verification::Structure),
        (
            "sample.parquet",
            "parquet",
            "database",
            Verification::Structure,
        ),
    ];

    for &(file_name, expected_fid, expected_cat, expected_verif) in matrix {
        let file_path = fixtures_dir.join(file_name);
        assert!(
            file_path.is_file(),
            "Fixture file not found: {:?}",
            file_path
        );

        // 1. End-to-end OS handle detection
        let rep = detect_path(&file_path, &opts)
            .unwrap_or_else(|e| panic!("Failed to detect path {:?}: {:?}", file_path, e));

        assert_eq!(
            rep.identification,
            Identification::Format,
            "Mismatch for {}",
            file_name
        );
        assert_eq!(
            rep.format_id.as_deref(),
            Some(expected_fid),
            "Format ID mismatch for {}",
            file_name
        );
        assert_eq!(
            rep.category, expected_cat,
            "Category mismatch for {}",
            file_name
        );
        assert_eq!(
            rep.verification, expected_verif,
            "Verification level mismatch for {}",
            file_name
        );

        // 2. Extension diagnosis with real file extension
        let ext = file_path.extension().unwrap_or_default();
        let match_status = compare_extension(&rep, Some(ext));
        assert_eq!(
            match_status,
            ExtensionStatus::Match,
            "Expected extension match for {}",
            file_name
        );

        // 3. Extension diagnosis with intentionally mismatched extension
        let mismatch_status = compare_extension(&rep, Some(OsStr::new("mismatched_xyz_ext_test")));
        assert_eq!(mismatch_status, ExtensionStatus::Unrecognized);

        let conflict_status = compare_extension(&rep, Some(OsStr::new("png")));
        if expected_fid != "png" {
            assert_eq!(conflict_status, ExtensionStatus::Mismatch);
        }
    }
}
