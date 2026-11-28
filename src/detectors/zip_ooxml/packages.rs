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

const RES_STRING_POOL_TYPE: u16 = 0x0001;
const RES_XML_TYPE: u16 = 0x0003;
const RES_XML_START_NAMESPACE_TYPE: u16 = 0x0100;
const RES_XML_END_NAMESPACE_TYPE: u16 = 0x0101;
const RES_XML_START_ELEMENT_TYPE: u16 = 0x0102;
const RES_XML_END_ELEMENT_TYPE: u16 = 0x0103;
const AXML_NO_INDEX: u32 = 0xFFFF_FFFF;

fn read_u16_le(buf: &[u8], off: usize) -> Option<u16> {
    let bytes: [u8; 2] = buf.get(off..off + 2)?.try_into().ok()?;
    Some(u16::from_le_bytes(bytes))
}

fn read_u32_le(buf: &[u8], off: usize) -> Option<u32> {
    let bytes: [u8; 4] = buf.get(off..off + 4)?.try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

struct AxmlStringPool<'a> {
    chunk: &'a [u8],
    hdr_size: usize,
    string_count: u32,
    is_utf8: bool,
    strings_start: usize,
}

impl<'a> AxmlStringPool<'a> {
    fn parse(chunk: &'a [u8], hdr_size: usize) -> Option<Self> {
        if hdr_size < 28 || chunk.len() < hdr_size {
            return None;
        }
        let string_count = read_u32_le(chunk, 8)?;
        let flags = read_u32_le(chunk, 16)?;
        let strings_start = read_u32_le(chunk, 20)? as usize;
        let offsets_end = hdr_size.checked_add((string_count as usize).checked_mul(4)?)?;
        if string_count == 0
            || offsets_end > chunk.len()
            || strings_start < offsets_end
            || strings_start > chunk.len()
        {
            return None;
        }
        Some(Self {
            chunk,
            hdr_size,
            string_count,
            is_utf8: (flags & 0x0100) != 0,
            strings_start,
        })
    }

    fn get(&self, idx: u32) -> Option<String> {
        if idx >= self.string_count {
            return None;
        }
        let rel_off = read_u32_le(self.chunk, self.hdr_size + (idx as usize) * 4)? as usize;
        let mut pos = self.strings_start.checked_add(rel_off)?;
        if pos >= self.chunk.len() {
            return None;
        }

        if self.is_utf8 {
            // Two length encodings: UTF-16 length (1 or 2 bytes), then UTF-8 byte length (1 or 2 bytes)
            let b0 = *self.chunk.get(pos)?;
            pos += if (b0 & 0x80) != 0 { 2 } else { 1 };
            let u0 = *self.chunk.get(pos)?;
            let byte_len = if (u0 & 0x80) != 0 {
                let u1 = *self.chunk.get(pos + 1)?;
                pos += 2;
                (((u0 & 0x7F) as usize) << 8) | (u1 as usize)
            } else {
                pos += 1;
                u0 as usize
            };
            let s_bytes = self.chunk.get(pos..pos.checked_add(byte_len)?)?;
            std::str::from_utf8(s_bytes).ok().map(|s| s.to_string())
        } else {
            let l0 = read_u16_le(self.chunk, pos)?;
            let char_len = if (l0 & 0x8000) != 0 {
                let l1 = read_u16_le(self.chunk, pos + 2)?;
                pos += 4;
                (((l0 & 0x7FFF) as usize) << 16) | (l1 as usize)
            } else {
                pos += 2;
                l0 as usize
            };
            let byte_len = char_len.checked_mul(2)?;
            let raw = self.chunk.get(pos..pos.checked_add(byte_len)?)?;
            let units: Vec<u16> = raw
                .as_chunks::<2>()
                .0
                .iter()
                .map(|c| u16::from_le_bytes(*c))
                .collect();
            String::from_utf16(&units).ok()
        }
    }
}

fn validate_axml_manifest(
    data: &[u8],
    max_xml_depth: usize,
    report: &mut DetectionReport,
) -> ApkManifestStatus {
    // Compiled Android Binary XML (AXML):
    // ResXMLTree_header (type=0x0003, headerSize=0x0008, totalSize) followed by
    // ResStringPool_header and XML node chunks whose root StartElement is <manifest>.
    if data.len() < 36 {
        return ApkManifestStatus::Invalid;
    }
    let Some(root_type) = read_u16_le(data, 0) else {
        return ApkManifestStatus::Invalid;
    };
    let Some(root_hdr_size) = read_u16_le(data, 2).map(|v| v as usize) else {
        return ApkManifestStatus::Invalid;
    };
    let Some(total_size) = read_u32_le(data, 4).map(|v| v as usize) else {
        return ApkManifestStatus::Invalid;
    };
    if root_type != RES_XML_TYPE
        || root_hdr_size != 8
        || total_size < 36
        || total_size != data.len()
    {
        return ApkManifestStatus::Invalid;
    }

    let mut pos = root_hdr_size;
    let mut string_pool: Option<AxmlStringPool<'_>> = None;
    let mut has_android_ns = false;
    let mut saw_root_manifest = false;
    let mut closed_root_manifest = false;
    let mut depth = 0usize;

    while pos < total_size {
        let Some(chunk_type) = read_u16_le(data, pos) else {
            return ApkManifestStatus::Invalid;
        };
        let Some(chunk_hdr_size) = read_u16_le(data, pos + 2).map(|v| v as usize) else {
            return ApkManifestStatus::Invalid;
        };
        let Some(chunk_size) = read_u32_le(data, pos + 4).map(|v| v as usize) else {
            return ApkManifestStatus::Invalid;
        };
        let Some(chunk_end) = pos.checked_add(chunk_size) else {
            return ApkManifestStatus::Invalid;
        };
        if chunk_hdr_size < 8 || chunk_size < chunk_hdr_size || chunk_end > total_size {
            return ApkManifestStatus::Invalid;
        }
        let chunk = &data[pos..chunk_end];

        match chunk_type {
            RES_STRING_POOL_TYPE => {
                if string_pool.is_some() {
                    return ApkManifestStatus::Invalid;
                }
                let Some(pool) = AxmlStringPool::parse(chunk, chunk_hdr_size) else {
                    return ApkManifestStatus::Invalid;
                };
                string_pool = Some(pool);
            }
            RES_XML_START_NAMESPACE_TYPE => {
                if chunk_hdr_size < 16 || chunk.len() < chunk_hdr_size + 8 {
                    return ApkManifestStatus::Invalid;
                }
                if let Some(pool) = &string_pool {
                    if let Some(uri_idx) = read_u32_le(chunk, chunk_hdr_size + 4) {
                        if let Some(uri) = pool.get(uri_idx) {
                            if uri
                                .eq_ignore_ascii_case("http://schemas.android.com/apk/res/android")
                            {
                                has_android_ns = true;
                            }
                        }
                    }
                }
            }
            RES_XML_END_NAMESPACE_TYPE => {
                if chunk_hdr_size < 16 || chunk.len() < chunk_hdr_size + 8 {
                    return ApkManifestStatus::Invalid;
                }
            }
            RES_XML_START_ELEMENT_TYPE => {
                let Some(pool) = &string_pool else {
                    return ApkManifestStatus::Invalid;
                };
                if chunk_hdr_size < 16 || chunk.len() < chunk_hdr_size + 20 || closed_root_manifest
                {
                    return ApkManifestStatus::Invalid;
                }
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

                let Some(name_idx) = read_u32_le(chunk, chunk_hdr_size + 4) else {
                    return ApkManifestStatus::Invalid;
                };
                let Some(tag_name) = pool.get(name_idx) else {
                    return ApkManifestStatus::Invalid;
                };

                if !saw_root_manifest {
                    if tag_name != "manifest" {
                        return ApkManifestStatus::Invalid;
                    }
                    let Some(attr_start) =
                        read_u16_le(chunk, chunk_hdr_size + 8).map(|v| v as usize)
                    else {
                        return ApkManifestStatus::Invalid;
                    };
                    let Some(attr_size) =
                        read_u16_le(chunk, chunk_hdr_size + 10).map(|v| v as usize)
                    else {
                        return ApkManifestStatus::Invalid;
                    };
                    let Some(attr_count) =
                        read_u16_le(chunk, chunk_hdr_size + 12).map(|v| v as usize)
                    else {
                        return ApkManifestStatus::Invalid;
                    };
                    let attrs_base = chunk_hdr_size.saturating_add(attr_start);
                    let Some(attrs_end) = attr_count
                        .checked_mul(attr_size)
                        .and_then(|n| attrs_base.checked_add(n))
                    else {
                        return ApkManifestStatus::Invalid;
                    };
                    if (attr_count > 0 && attr_size < 20) || attrs_end > chunk.len() {
                        return ApkManifestStatus::Invalid;
                    }

                    let mut has_package_attr = false;
                    for i in 0..attr_count {
                        let a_off = attrs_base + i * attr_size;
                        let attr_name_idx = read_u32_le(chunk, a_off + 4).unwrap_or(AXML_NO_INDEX);
                        if pool.get(attr_name_idx).as_deref() == Some("package") {
                            let raw_idx = read_u32_le(chunk, a_off + 8).unwrap_or(AXML_NO_INDEX);
                            let data_val = read_u32_le(chunk, a_off + 16).unwrap_or(AXML_NO_INDEX);
                            if (raw_idx != AXML_NO_INDEX
                                && pool.get(raw_idx).is_some_and(|v| !v.is_empty()))
                                || (data_val != AXML_NO_INDEX
                                    && pool.get(data_val).is_some_and(|v| !v.is_empty()))
                            {
                                has_package_attr = true;
                                break;
                            }
                        }
                    }

                    if !has_package_attr && !has_android_ns {
                        return ApkManifestStatus::Invalid;
                    }
                    saw_root_manifest = true;
                }
            }
            RES_XML_END_ELEMENT_TYPE => {
                let Some(pool) = &string_pool else {
                    return ApkManifestStatus::Invalid;
                };
                if chunk_hdr_size < 16 || chunk.len() < chunk_hdr_size + 8 || depth == 0 {
                    return ApkManifestStatus::Invalid;
                }
                depth -= 1;
                if depth == 0 && saw_root_manifest {
                    let Some(name_idx) = read_u32_le(chunk, chunk_hdr_size + 4) else {
                        return ApkManifestStatus::Invalid;
                    };
                    if pool.get(name_idx).as_deref() != Some("manifest") {
                        return ApkManifestStatus::Invalid;
                    }
                    closed_root_manifest = true;
                }
            }
            _ => {}
        }

        pos = chunk_end;
    }

    if saw_root_manifest && closed_root_manifest && depth == 0 {
        ApkManifestStatus::Valid
    } else {
        ApkManifestStatus::Invalid
    }
}

fn validate_plain_xml_manifest(
    data: &[u8],
    max_xml_depth: usize,
    report: &mut DetectionReport,
) -> ApkManifestStatus {
    let mut reader = Reader::from_reader(Cursor::new(data));
    reader.config_mut().trim_text(true);
    reader.config_mut().check_end_names = true;
    let mut buf = Vec::new();
    let mut depth = 0usize;
    let mut root_count = 0usize;
    let mut root_valid = false;
    let mut root_closed = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                if depth == 0 {
                    root_count += 1;
                    if root_count > 1 {
                        return ApkManifestStatus::Invalid;
                    }
                    root_valid = has_valid_manifest_root(&e);
                    if !root_valid {
                        return ApkManifestStatus::Invalid;
                    }
                }
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
                if depth == 0 {
                    root_count += 1;
                    if root_count > 1 {
                        return ApkManifestStatus::Invalid;
                    }
                    root_valid = has_valid_manifest_root(&e);
                    if !root_valid {
                        return ApkManifestStatus::Invalid;
                    }
                    root_closed = true;
                }
            }
            Ok(Event::End(_)) => {
                if depth == 0 {
                    return ApkManifestStatus::Invalid;
                }
                depth -= 1;
                if depth == 0 {
                    root_closed = true;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => return ApkManifestStatus::Invalid,
            _ => {}
        }
        buf.clear();
    }

    if root_count == 1 && root_valid && root_closed && depth == 0 {
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
        validate_axml_manifest(&data, options.max_xml_depth, report)
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

/// Inspects ZIP archive for package derivatives: APK, IPA, JAR, or KMZ using default budgets.
pub fn check_zip_packages<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    report: &mut DetectionReport,
) -> bool {
    check_zip_packages_with_options(archive, &DetectOptions::default(), report)
}

/// Inspects ZIP archive for package derivatives: APK, IPA, JAR, or KMZ under caller-supplied budgets.
pub fn check_zip_packages_with_options<R: Read + Seek>(
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
