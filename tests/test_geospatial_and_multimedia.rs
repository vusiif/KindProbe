use std::ffi::OsStr;
use std::io::Write;
use flate2::write::GzEncoder;
use flate2::Compression;
use kindprobe::budget::DetectOptions;
use kindprobe::detect_bytes;
use kindprobe::extension::compare_extension;
use kindprobe::model::{ExtensionStatus, Identification, Verification};

#[test]
fn test_detect_geospatial_and_markup() {
    let opts = DetectOptions::default();

    // 1. OGC GeoPackage (.gpkg in SQLite container)
    let mut gpkg = vec![0u8; 100];
    gpkg[0..16].copy_from_slice(b"SQLite format 3\0");
    gpkg[16..18].copy_from_slice(&4096u16.to_be_bytes());
    gpkg[18] = 1;
    gpkg[19] = 1;
    gpkg[68..72].copy_from_slice(b"GPKG");
    let rep_gpkg = detect_bytes(&gpkg, &opts).expect("detect gpkg");
    assert_eq!(rep_gpkg.format_id.as_deref(), Some("gpkg"));
    assert_eq!(rep_gpkg.container_id.as_deref(), Some("sqlite3"));
    assert_eq!(rep_gpkg.category, "database");
    assert_eq!(rep_gpkg.verification, Verification::Structure);
    assert_eq!(compare_extension(&rep_gpkg, Some(OsStr::new("gpkg"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_gpkg, Some(OsStr::new("sqlite3"))), ExtensionStatus::Match);

    // 2. Scalable Vector Graphics (.svg)
    let svg = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\"><circle cx=\"50\" cy=\"50\" r=\"40\"/></svg>";
    let rep_svg = detect_bytes(svg, &opts).expect("detect svg");
    assert_eq!(rep_svg.format_id.as_deref(), Some("svg"));
    assert_eq!(rep_svg.category, "image");
    assert_eq!(rep_svg.container_id.as_deref(), Some("xml"));
    assert_eq!(compare_extension(&rep_svg, Some(OsStr::new("svg"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_svg, Some(OsStr::new("png"))), ExtensionStatus::Mismatch);

    // 3. Keyhole Markup Language (.kml)
    let kml = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<kml xmlns=\"http://www.opengis.net/kml/2.2\"><Placemark><name>Point</name></Placemark></kml>";
    let rep_kml = detect_bytes(kml, &opts).expect("detect kml");
    assert_eq!(rep_kml.format_id.as_deref(), Some("kml"));
    assert_eq!(rep_kml.category, "geospatial");
    assert_eq!(compare_extension(&rep_kml, Some(OsStr::new("kml"))), ExtensionStatus::Match);

    // 4. GPS Exchange Format (.gpx)
    let gpx = b"<?xml version=\"1.0\"?>\n<gpx version=\"1.1\" creator=\"KindProbe\"><trk><trkseg><trkpt lat=\"47.64\" lon=\"-122.13\"/></trkseg></trk></gpx>";
    let rep_gpx = detect_bytes(gpx, &opts).expect("detect gpx");
    assert_eq!(rep_gpx.format_id.as_deref(), Some("gpx"));
    assert_eq!(rep_gpx.category, "geospatial");
    assert_eq!(compare_extension(&rep_gpx, Some(OsStr::new("gpx"))), ExtensionStatus::Match);

    // 5. GeoJSON (.geojson)
    let geojson = b"{\n  \"type\": \"FeatureCollection\",\n  \"features\": [\n    {\"type\": \"Feature\", \"geometry\": {\"type\": \"Point\", \"coordinates\": [102.0, 0.5]}}\n  ]\n}";
    let rep_geo = detect_bytes(geojson, &opts).expect("detect geojson");
    assert_eq!(rep_geo.format_id.as_deref(), Some("geojson"));
    assert_eq!(rep_geo.category, "geospatial");
    assert_eq!(rep_geo.container_id.as_deref(), Some("json"));
    assert_eq!(compare_extension(&rep_geo, Some(OsStr::new("geojson"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_geo, Some(OsStr::new("json"))), ExtensionStatus::Match);

    // 6. COLLADA 3D Asset Exchange (.dae)
    let dae = b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<COLLADA xmlns=\"http://www.collada.org/2005/11/COLLADASchema\" version=\"1.4.1\"></COLLADA>";
    let rep_dae = detect_bytes(dae, &opts).expect("detect dae");
    assert_eq!(rep_dae.format_id.as_deref(), Some("dae"));
    assert_eq!(rep_dae.category, "model");
    assert_eq!(compare_extension(&rep_dae, Some(OsStr::new("dae"))), ExtensionStatus::Match);

    // 7. Apple Property List (.plist)
    let plist = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\"><dict></dict></plist>";
    let rep_plist = detect_bytes(plist, &opts).expect("detect plist");
    assert_eq!(rep_plist.format_id.as_deref(), Some("plist"));
    assert_eq!(rep_plist.category, "configuration");
    assert_eq!(compare_extension(&rep_plist, Some(OsStr::new("plist"))), ExtensionStatus::Match);
}

#[test]
fn test_detect_nested_svgz() {
    let opts = DetectOptions::default();

    let raw_svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"200\" height=\"200\"><rect width=\"100\" height=\"100\" fill=\"red\"/></svg>";
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(raw_svg).expect("compress svg");
    let svgz_bytes = encoder.finish().expect("finish gzip");

    let rep = detect_bytes(&svgz_bytes, &opts).expect("detect svgz");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("svgz"));
    assert_eq!(rep.category, "image");
    assert_eq!(rep.container_id.as_deref(), Some("gzip"));
    assert_eq!(rep.verification, Verification::Structure);
    assert!(rep.evidence.iter().any(|e| e.code == "sig.svg.embedded"));
    assert_eq!(compare_extension(&rep, Some(OsStr::new("svgz"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("svg"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("gz"))), ExtensionStatus::Match);
}

#[test]
fn test_detect_stream_and_modern_assets() {
    let opts = DetectOptions::default();

    // 1. MPEG-2 Transport Stream (.ts)
    let mut ts = vec![0u8; 600];
    ts[0] = 0x47;
    ts[188] = 0x47;
    ts[376] = 0x47;
    ts[564] = 0x47;
    let rep_ts = detect_bytes(&ts, &opts).expect("detect ts");
    assert_eq!(rep_ts.format_id.as_deref(), Some("ts"));
    assert_eq!(rep_ts.category, "video");
    assert_eq!(compare_extension(&rep_ts, Some(OsStr::new("ts"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_ts, Some(OsStr::new("m2ts"))), ExtensionStatus::Match);

    // 2. BDAV MPEG-2 Transport Stream (.m2ts)
    let mut m2ts = vec![0u8; 600];
    m2ts[4] = 0x47;
    m2ts[196] = 0x47;
    m2ts[388] = 0x47;
    m2ts[580] = 0x47;
    let rep_m2ts = detect_bytes(&m2ts, &opts).expect("detect m2ts");
    assert_eq!(rep_m2ts.format_id.as_deref(), Some("m2ts"));
    assert_eq!(rep_m2ts.category, "video");
    assert_eq!(compare_extension(&rep_m2ts, Some(OsStr::new("m2ts"))), ExtensionStatus::Match);

    // 3. Pixar Universal Scene Description Crate (.usdc)
    let mut usdc = vec![0u8; 32];
    usdc[0..8].copy_from_slice(b"PXR-USDC");
    let rep_usdc = detect_bytes(&usdc, &opts).expect("detect usdc");
    assert_eq!(rep_usdc.format_id.as_deref(), Some("usdc"));
    assert_eq!(rep_usdc.category, "model");
    assert_eq!(compare_extension(&rep_usdc, Some(OsStr::new("usdc"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_usdc, Some(OsStr::new("usd"))), ExtensionStatus::Match);

    // 4. Universal 3D (.u3d)
    let mut u3d = vec![0u8; 32];
    u3d[0..4].copy_from_slice(b"U3D\x00");
    let rep_u3d = detect_bytes(&u3d, &opts).expect("detect u3d");
    assert_eq!(rep_u3d.format_id.as_deref(), Some("u3d"));
    assert_eq!(rep_u3d.category, "model");
    assert_eq!(compare_extension(&rep_u3d, Some(OsStr::new("u3d"))), ExtensionStatus::Match);

    // 5. Valve Texture Format (.vtf)
    let mut vtf = vec![0u8; 32];
    vtf[0..4].copy_from_slice(b"VTF\x00");
    let rep_vtf = detect_bytes(&vtf, &opts).expect("detect vtf");
    assert_eq!(rep_vtf.format_id.as_deref(), Some("vtf"));
    assert_eq!(rep_vtf.category, "image");
    assert_eq!(compare_extension(&rep_vtf, Some(OsStr::new("vtf"))), ExtensionStatus::Match);

    // 6. ETC Texture PKM (.pkm)
    let mut pkm = vec![0u8; 32];
    pkm[0..6].copy_from_slice(b"PKM 10");
    let rep_pkm = detect_bytes(&pkm, &opts).expect("detect pkm");
    assert_eq!(rep_pkm.format_id.as_deref(), Some("pkm"));
    assert_eq!(rep_pkm.category, "image");
    assert_eq!(compare_extension(&rep_pkm, Some(OsStr::new("pkm"))), ExtensionStatus::Match);
}

#[test]
fn test_detect_bytecode_archives_and_security() {
    let opts = DetectOptions::default();

    // 1. BEAM Bytecode (Erlang / Elixir)
    let mut beam = vec![0u8; 32];
    beam[0..4].copy_from_slice(b"FOR1");
    beam[8..12].copy_from_slice(b"BEAM");
    let rep_beam = detect_bytes(&beam, &opts).expect("detect beam");
    assert_eq!(rep_beam.format_id.as_deref(), Some("beam"));
    assert_eq!(rep_beam.category, "executable");
    assert_eq!(compare_extension(&rep_beam, Some(OsStr::new("beam"))), ExtensionStatus::Match);

    // 2. Python Bytecode (.pyc)
    let mut pyc = vec![0u8; 32];
    pyc[0..4].copy_from_slice(&[0x6f, 0x0d, 0x0d, 0x0a]); // Python 3.10
    let rep_pyc = detect_bytes(&pyc, &opts).expect("detect pyc");
    assert_eq!(rep_pyc.format_id.as_deref(), Some("pyc"));
    assert_eq!(rep_pyc.category, "executable");
    assert_eq!(compare_extension(&rep_pyc, Some(OsStr::new("pyc"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_pyc, Some(OsStr::new("pyo"))), ExtensionStatus::Match);

    // 3. FreeArc (.arc)
    let mut arc = vec![0u8; 32];
    arc[0..4].copy_from_slice(b"ArC\x01");
    let rep_arc = detect_bytes(&arc, &opts).expect("detect arc");
    assert_eq!(rep_arc.format_id.as_deref(), Some("arc"));
    assert_eq!(rep_arc.category, "archive");
    assert_eq!(compare_extension(&rep_arc, Some(OsStr::new("arc"))), ExtensionStatus::Match);

    // 4. ALZip (.alz)
    let mut alz = vec![0u8; 32];
    alz[0..4].copy_from_slice(b"ALZ\x01");
    let rep_alz = detect_bytes(&alz, &opts).expect("detect alz");
    assert_eq!(rep_alz.format_id.as_deref(), Some("alz"));
    assert_eq!(rep_alz.category, "archive");
    assert_eq!(compare_extension(&rep_alz, Some(OsStr::new("alz"))), ExtensionStatus::Match);

    // 5. OpenSSH Private Key (.key)
    let sshkey = b"-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAA=\n-----END OPENSSH PRIVATE KEY-----";
    let rep_ssh = detect_bytes(sshkey, &opts).expect("detect sshkey");
    assert_eq!(rep_ssh.format_id.as_deref(), Some("sshkey"));
    assert_eq!(rep_ssh.category, "security");
    assert_eq!(compare_extension(&rep_ssh, Some(OsStr::new("key"))), ExtensionStatus::Match);

    // 6. Direct-Access-Archive (.daa)
    let mut daa = vec![0u8; 32];
    daa[0..4].copy_from_slice(b"DAA\x00");
    let rep_daa = detect_bytes(&daa, &opts).expect("detect daa");
    assert_eq!(rep_daa.format_id.as_deref(), Some("daa"));
    assert_eq!(rep_daa.category, "disk");
    assert_eq!(compare_extension(&rep_daa, Some(OsStr::new("daa"))), ExtensionStatus::Match);

    // 7. ECM (.ecm)
    let mut ecm = vec![0u8; 32];
    ecm[0..4].copy_from_slice(b"ECM\x00");
    let rep_ecm = detect_bytes(&ecm, &opts).expect("detect ecm");
    assert_eq!(rep_ecm.format_id.as_deref(), Some("ecm"));
    assert_eq!(rep_ecm.category, "disk");
    assert_eq!(compare_extension(&rep_ecm, Some(OsStr::new("ecm"))), ExtensionStatus::Match);

    // 8. WIM (.wim)
    let mut wim = vec![0u8; 32];
    wim[0..8].copy_from_slice(b"MSWIM\x00\x00\x00");
    let rep_wim = detect_bytes(&wim, &opts).expect("detect wim");
    assert_eq!(rep_wim.format_id.as_deref(), Some("wim"));
    assert_eq!(rep_wim.category, "archive");
    assert_eq!(compare_extension(&rep_wim, Some(OsStr::new("wim"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_wim, Some(OsStr::new("swm"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_wim, Some(OsStr::new("esd"))), ExtensionStatus::Match);
}
