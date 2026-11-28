use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::{Cursor, Read, Seek};
use zip::ZipArchive;

use crate::budget::{DecompressLimitReader, DetectOptions};
use crate::model::{
    DetectionReport, EvidenceItem, Identification, Inspection, Verification, WarningItem,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ApkManifestStatus {
    Valid,
    Invalid,
    BudgetExceeded,
}

fn validate_axml_manifest(data: &[u8]) -> bool {
    // Compiled Android Binary XML (AXML):
    // ResXMLTree_header (type=0x0003, headerSize=0x0008, totalSize) followed by
    // ResStringPool_header (type=0x0001, headerSize>=0x001C, poolSize) containing "manifest".
    if data.len() < 36 || !data.starts_with(&[0x03, 0x00, 0x08, 0x00]) {
        return false;
    }
    let total_size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
    if total_size < 36 || total_size != data.len() {
        return false;
    }
    let chunk_type = u16::from_le_bytes([data[8], data[9]]);
    let chunk_hdr_size = u16::from_le_bytes([data[10], data[11]]) as usize;
    let pool_size = u32::from_le_bytes([data[12], data[13], data[14], data[15]]) as usize;
    let string_count = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
    let strings_start = u32::from_le_bytes([data[28], data[29], data[30], data[31]]) as usize;

    if chunk_type != 0x0001
        || chunk_hdr_size < 28
        || pool_size < chunk_hdr_size
        || 8usize.saturating_add(pool_size) > data.len()
        || string_count == 0
        || strings_start < chunk_hdr_size
        || strings_start > pool_size
    {
        return false;
    }

    let pool_bytes = &data[8 + strings_start..8 + pool_size];
    pool_bytes.windows(8).any(|w| w == b"manifest")
        || pool_bytes
            .windows(16)
            .any(|w| w == b"m\x00a\x00n\x00i\x00f\x00e\x00s\x00t\x00")
}

fn validate_plain_xml_manifest(
    data: &[u8],
    max_xml_depth: usize,
    report: &mut DetectionReport,
) -> ApkManifestStatus {
    let mut reader = Reader::from_reader(Cursor::new(data));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut depth = 0usize;
    let mut root_checked = false;
    let mut root_valid = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                depth += 1;
                if depth > max_xml_depth {
                    report.inspection = Inspection::Limited;
                    report.warnings.push(WarningItem::new(
                        "budget.xml_depth_exceeded",
                        format!(
                            "XML element depth ({}) exceeded allowed maximum ({})",
                            depth, max_xml_depth
                        ),
                    ));
                    return ApkManifestStatus::BudgetExceeded;
                }
                if !root_checked {
                    root_checked = true;
                    root_valid = has_valid_manifest_root(&e);
                    if !root_valid {
                        return ApkManifestStatus::Invalid;
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                let empty_depth = depth + 1;
                if empty_depth > max_xml_depth {
                    report.inspection = Inspection::Limited;
                    report.warnings.push(WarningItem::new(
                        "budget.xml_depth_exceeded",
                        format!(
                            "XML element depth ({}) exceeded allowed maximum ({})",
                            empty_depth, max_xml_depth
                        ),
                    ));
                    return ApkManifestStatus::BudgetExceeded;
                }
                if !root_checked {
                    root_checked = true;
                    root_valid = has_valid_manifest_root(&e);
                    if !root_valid {
                        return ApkManifestStatus::Invalid;
                    }
                }
            }
            Ok(Event::End(_)) => {
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) => break,
            Err(_) => return ApkManifestStatus::Invalid,
            _ => {}
        }
        buf.clear();
    }

    if root_valid {
        ApkManifestStatus::Valid
    } else {
        ApkManifestStatus::Invalid
    }
}

fn has_valid_manifest_root(elem: &quick_xml::events::BytesStart<'_>) -> bool {
    if elem.local_name().as_ref() != "manifest" {
        return false;
    }
    let mut has_package = false;
    let mut has_android_ns = false;
    for attr in elem.attributes().flatten() {
        let key: &str = attr.key.as_ref();
        if key == "package" && !attr.value.is_empty() {
            has_package = true;
        } else if key == "xmlns:android"
            && attr
                .value
                .as_bytes()
                .windows(7)
                .any(|w| w.eq_ignore_ascii_case(b"android"))
        {
            has_android_ns = true;
        }
    }
    has_package || has_android_ns
}

pub(super) fn verify_apk_manifest_stream<R: Read>(
    stream: R,
    options: &DetectOptions,
    report: &mut DetectionReport,
) -> ApkManifestStatus {
    let remaining_total = options
        .max_total_decompressed_bytes
        .saturating_sub(report.stats.decompressed_bytes);
    let allowed = options.max_single_decompressed_bytes.min(remaining_total);
    if allowed == 0 {
        report.inspection = Inspection::Limited;
        report.warnings.push(WarningItem::new(
            "budget.total_decompressed_exceeded",
            "Cumulative decompressed byte budget exceeded",
        ));
        return ApkManifestStatus::BudgetExceeded;
    }

    let mut limit_reader = DecompressLimitReader::new(stream, allowed);
    let mut data = Vec::with_capacity(allowed.min(16 * 1024));
    let read_res = limit_reader.read_to_end(&mut data);
    report.stats.decompressed_bytes += limit_reader.bytes_read();

    if limit_reader.is_exceeded() {
        report.inspection = Inspection::Limited;
        if report.stats.decompressed_bytes >= options.max_total_decompressed_bytes {
            report.warnings.push(WarningItem::new(
                "budget.total_decompressed_exceeded",
                "Cumulative decompressed byte budget exceeded",
            ));
        } else {
            report.warnings.push(WarningItem::new(
                "budget.xml_output_exceeded",
                "Decompressed size of AndroidManifest.xml exceeded single entry limit",
            ));
        }
        return ApkManifestStatus::BudgetExceeded;
    }

    if read_res.is_err() {
        return ApkManifestStatus::Invalid;
    }

    if data.starts_with(&[0x03, 0x00, 0x08, 0x00]) {
        if validate_axml_manifest(&data) {
            ApkManifestStatus::Valid
        } else {
            ApkManifestStatus::Invalid
        }
    } else {
        validate_plain_xml_manifest(&data, options.max_xml_depth, report)
    }
}

pub(super) fn apply_apk(report: &mut DetectionReport, has_dex: bool) {
    apply_pkg(report, "apk", "package", "apk", &["apk", "xapk"]);
    report.evidence.push(EvidenceItem::new("zip.apk.manifest"));
    if has_dex {
        report.evidence.push(EvidenceItem::new("zip.apk.dex"));
    }
}

/// Inspects ZIP archive for package derivatives: APK, IPA, JAR, or KMZ.
pub fn check_zip_packages<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    options: &DetectOptions,
    report: &mut DetectionReport,
) -> bool {
    // 1. Android Package (APK): requires structurally valid AndroidManifest.xml (DEX is optional)
    let manifest_status = match archive.by_name("AndroidManifest.xml") {
        Ok(manifest) => Some(verify_apk_manifest_stream(manifest, options, report)),
        Err(_) => None,
    };
    match manifest_status {
        Some(ApkManifestStatus::Valid) => {
            let has_dex = archive.by_name("classes.dex").is_ok();
            apply_apk(report, has_dex);
            return true;
        }
        Some(ApkManifestStatus::BudgetExceeded) => {
            return true;
        }
        Some(ApkManifestStatus::Invalid) | None => {}
    }

    // 1b. Android App Bundle (AAB): BundleConfig.pb
    if archive.by_name("BundleConfig.pb").is_ok() {
        apply_pkg(report, "aab", "package", "aab", &["aab"]);
        report.evidence.push(EvidenceItem::new("zip.aab.bundle"));
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

    // 5. VS Code Extension (.vsix) & Firefox Add-on (.xpi) & Wheel (.whl) & NuGet (.nupkg)
    if archive.by_name("extension.vsixmanifest").is_ok() {
        apply_pkg(report, "vsix", "package", "vsix", &["vsix"]);
        report.evidence.push(EvidenceItem::new("zip.vsix.manifest"));
        return true;
    }
    if archive.by_name("install.rdf").is_ok() || archive.by_name("META-INF/mozilla.rsa").is_ok() {
        apply_pkg(report, "xpi", "package", "xpi", &["xpi"]);
        report.evidence.push(EvidenceItem::new("zip.xpi.manifest"));
        return true;
    }
    if archive.by_name("AppxManifest.xml").is_ok() {
        apply_pkg(report, "msix", "package", "msix", &["msix", "appx"]);
        report.evidence.push(EvidenceItem::new("zip.msix.manifest"));
        return true;
    }
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index_raw(i) {
            if let Ok(name) = entry.name() {
                if name.ends_with(".dist-info/WHEEL") {
                    apply_pkg(report, "whl", "package", "whl", &["whl"]);
                    report.evidence.push(EvidenceItem::new("zip.whl.wheel"));
                    return true;
                }
                if name.ends_with(".nuspec") && !name.contains('/') {
                    apply_pkg(report, "nupkg", "package", "nupkg", &["nupkg"]);
                    report.evidence.push(EvidenceItem::new("zip.nupkg.nuspec"));
                    return true;
                }
            }
        }
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
