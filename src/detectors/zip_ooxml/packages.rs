use std::io::{Read, Seek};
use zip::ZipArchive;

use crate::model::{
    DetectionReport, EvidenceItem, Identification, Inspection, Verification,
};

/// Inspects ZIP archive for package derivatives: APK, IPA, JAR, or KMZ.
pub fn check_zip_packages<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    report: &mut DetectionReport,
) -> bool {
    // 1. Android Package (APK): AndroidManifest.xml and classes.dex
    let has_manifest = archive.by_name("AndroidManifest.xml").is_ok();
    let has_dex = archive.by_name("classes.dex").is_ok();
    if has_manifest || has_dex {
        apply_pkg(report, "apk", "package", "apk", &["apk", "xapk"]);
        if has_manifest {
            report.evidence.push(EvidenceItem::new("zip.apk.manifest"));
        }
        if has_dex {
            report.evidence.push(EvidenceItem::new("zip.apk.dex"));
        }
        return true;
    }

    // 2. iOS Application Archive (IPA): Payload/*.app/Info.plist
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index_raw(i) {
            if let Ok(name) = entry.name() {
                if name.starts_with("Payload/") && name.ends_with(".app/Info.plist") {
                    apply_pkg(report, "ipa", "package", "ipa", &["ipa"]);
                    report.evidence.push(EvidenceItem::new("zip.ipa.payload"));
                    return true;
                }
            }
        }
    }

    // 3. Java Archive (JAR): META-INF/MANIFEST.MF
    if archive.by_name("META-INF/MANIFEST.MF").is_ok() {
        apply_pkg(report, "jar", "package", "jar", &["jar", "war", "ear"]);
        report.evidence.push(EvidenceItem::new("zip.jar.manifest"));
        return true;
    }

    // 4. Keyhole Markup Zip (KMZ): doc.kml
    if archive.by_name("doc.kml").is_ok() {
        apply_pkg(report, "kmz", "geospatial", "kmz", &["kmz", "kml"]);
        report.evidence.push(EvidenceItem::new("zip.kmz.kml"));
        return true;
    }

    false
}

fn apply_pkg(rep: &mut DetectionReport, fid: &str, cat: &str, rec: &str, exts: &[&str]) {
    rep.identification = Identification::Format;
    rep.format_id = Some(fid.to_string());
    rep.category = cat.to_string();
    rep.container_id = Some("zip".to_string());
    rep.verification = Verification::Structure;
    rep.inspection = Inspection::Complete;
    rep.recommended_extension = Some(rec.to_string());
    rep.accepted_extensions = exts.iter().map(|s| s.to_string()).collect();
}
