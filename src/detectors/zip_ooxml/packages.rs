use std::io::{Read, Seek};
use zip::ZipArchive;

use crate::model::{
    DetectionReport, EvidenceItem, Identification, Inspection, Verification,
};

/// Inspects ZIP archive for package derivatives: APK, IPA, or JAR.
pub fn check_zip_packages<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    report: &mut DetectionReport,
) -> bool {
    // 1. Android Package (APK): AndroidManifest.xml and classes.dex
    let has_manifest = archive.by_name("AndroidManifest.xml").is_ok();
    let has_dex = archive.by_name("classes.dex").is_ok();
    if has_manifest || has_dex {
        report.identification = Identification::Format;
        report.format_id = Some("apk".to_string());
        report.category = "package".to_string();
        report.container_id = Some("zip".to_string());
        report.verification = Verification::Structure;
        report.inspection = Inspection::Complete;
        report.recommended_extension = Some("apk".to_string());
        report.accepted_extensions = vec!["apk".to_string(), "xapk".to_string()];
        if has_manifest {
            report.evidence.push(EvidenceItem::new("zip.apk.manifest"));
        }
        if has_dex {
            report.evidence.push(EvidenceItem::new("zip.apk.dex"));
        }
        return true;
    }

    // 2. iOS Application Archive (IPA): Payload/*.app/Info.plist
    let mut has_ipa_payload = false;
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index_raw(i) {
            let name = entry.name();
            if name.starts_with("Payload/") && name.ends_with(".app/Info.plist") {
                has_ipa_payload = true;
                break;
            }
        }
    }
    if has_ipa_payload {
        report.identification = Identification::Format;
        report.format_id = Some("ipa".to_string());
        report.category = "package".to_string();
        report.container_id = Some("zip".to_string());
        report.verification = Verification::Structure;
        report.inspection = Inspection::Complete;
        report.recommended_extension = Some("ipa".to_string());
        report.accepted_extensions = vec!["ipa".to_string()];
        report.evidence.push(EvidenceItem::new("zip.ipa.payload"));
        return true;
    }

    // 3. Java Archive (JAR): META-INF/MANIFEST.MF
    if archive.by_name("META-INF/MANIFEST.MF").is_ok() {
        report.identification = Identification::Format;
        report.format_id = Some("jar".to_string());
        report.category = "package".to_string();
        report.container_id = Some("zip".to_string());
        report.verification = Verification::Structure;
        report.inspection = Inspection::Complete;
        report.recommended_extension = Some("jar".to_string());
        report.accepted_extensions = vec!["jar".to_string(), "war".to_string(), "ear".to_string()];
        report.evidence.push(EvidenceItem::new("zip.jar.manifest"));
        return true;
    }

    false
}
