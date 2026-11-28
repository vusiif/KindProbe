use super::opc::{normalize_opc_path, normalize_opc_path_cow};
use crate::budget::{BudgetReader, DetectOptions};
use crate::model::{DetectError, DetectionReport, Identification, Inspection, WarningItem};
use std::collections::HashSet;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug, Clone)]
pub struct ZipEntryMeta {
    pub name: String,
    pub compression_method: u16,
    pub flags: u16,
    pub compressed_size: u32,
    pub local_header_offset: u32,
}

const MAX_CACHED_XML_ENTRIES: usize = 128;

pub struct ZipCdPreCheckResult {
    pub cd_total_entries: usize,
    pub cd_size: usize,
    pub cd_offset: u64,
    pub is_zip64: bool,
    pub has_content_types: bool,
    pub has_rels: bool,
    pub has_ooxml_subdirs: bool,
    pub has_apk_manifest: bool,
    pub has_apk_dex: bool,
    pub apk_manifest_entry: Option<ZipEntryMeta>,
    pub has_aab_bundle: bool,
    pub has_npz_array: bool,
    pub has_ipa_payload: bool,
    pub has_jar_manifest: bool,
    pub has_kmz_kml: bool,
    pub has_mimetype: bool,
    pub has_epub_container: bool,
    pub has_whl_dist_info: bool,
    pub has_vsix_manifest: bool,
    pub has_nupkg_nuspec: bool,
    pub has_xpi_manifest: bool,
    pub has_msix_manifest: bool,
    pub content_types_entry: Option<ZipEntryMeta>,
    pub rels_entry: Option<ZipEntryMeta>,
    pub xml_entries: Vec<ZipEntryMeta>,
    pub xml_entries_capped: bool,
}

enum FastNameSet<T: Eq + std::hash::Hash> {
    Small(Vec<T>),
    Large(HashSet<T>),
}

impl<T: Eq + std::hash::Hash> FastNameSet<T> {
    fn with_expected_len(expected: usize) -> Self {
        if expected <= 32 {
            Self::Small(Vec::with_capacity(expected))
        } else {
            Self::Large(HashSet::with_capacity(expected))
        }
    }

    fn insert(&mut self, val: T) -> bool {
        match self {
            Self::Small(vec) => {
                if vec.iter().any(|existing| existing == &val) {
                    return false;
                }
                if vec.len() >= 32 {
                    let mut set = HashSet::with_capacity(vec.len() * 2);
                    for item in vec.drain(..) {
                        set.insert(item);
                    }
                    set.insert(val);
                    *self = Self::Large(set);
                } else {
                    vec.push(val);
                }
                true
            }
            Self::Large(set) => set.insert(val),
        }
    }
}

fn find_eocd_in_slice(slice: &[u8]) -> Option<(usize, usize, u64)> {
    for i in (0..=slice.len().saturating_sub(22)).rev() {
        if &slice[i..i + 4] == b"PK\x05\x06" {
            let comment_len = u16::from_le_bytes([slice[i + 20], slice[i + 21]]) as usize;
            if i + 22 + comment_len != slice.len() {
                continue;
            }
            let total_ent = u16::from_le_bytes([slice[i + 10], slice[i + 11]]) as usize;
            let size =
                u32::from_le_bytes([slice[i + 12], slice[i + 13], slice[i + 14], slice[i + 15]])
                    as usize;
            let offset =
                u32::from_le_bytes([slice[i + 16], slice[i + 17], slice[i + 18], slice[i + 19]])
                    as u64;
            return Some((total_ent, size, offset));
        }
    }
    None
}

/// Pre-checks ZIP End of Central Directory and Central Directory Headers
/// to enforce max_zip_directory_window, max_entries, and detect duplicate/conflicting members.
pub fn precheck_zip_central_directory<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    options: &DetectOptions,
    report: &mut DetectionReport,
) -> Result<Option<ZipCdPreCheckResult>, DetectError> {
    let file_len = match reader.seek(SeekFrom::End(0)) {
        Ok(l) => l,
        Err(e) => {
            if reader.is_budget_exceeded() {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.source_read_exceeded",
                    "Total source read budget exceeded during detection",
                ));
                return Ok(None);
            }
            return Err(DetectError::Io(e));
        }
    };

    let tail_search_len = (65536 + 22).min(file_len as usize);
    let mut eocd_found = false;
    let mut cd_total_entries = 0usize;
    let mut cd_size = 0usize;
    let mut cd_offset = 0u64;

    if file_len >= 22 {
        // Fast probe: check the last 1024 bytes using a stack buffer
        let fast_probe_len = 1024.min(file_len as usize);
        let fast_seek_pos = file_len - fast_probe_len as u64;
        let mut fast_buf = [0u8; 1024];
        if reader.seek(SeekFrom::Start(fast_seek_pos)).is_ok()
            && reader.read_exact(&mut fast_buf[..fast_probe_len]).is_ok()
        {
            if let Some((total_ent, size, offset)) = find_eocd_in_slice(&fast_buf[..fast_probe_len])
            {
                cd_total_entries = total_ent;
                cd_size = size;
                cd_offset = offset;
                eocd_found = true;
            }
        }

        // If not found in the last 1024 bytes and file is larger, fall back to full tail search
        if !eocd_found && file_len > 1024 {
            let seek_pos = file_len - tail_search_len as u64;
            if reader.seek(SeekFrom::Start(seek_pos)).is_ok() {
                let mut tail_buf = vec![0u8; tail_search_len];
                if reader.read_exact(&mut tail_buf).is_ok() {
                    if let Some((total_ent, size, offset)) = find_eocd_in_slice(&tail_buf) {
                        cd_total_entries = total_ent;
                        cd_size = size;
                        cd_offset = offset;
                        eocd_found = true;
                    }
                }
            }
        }
    }

    if !eocd_found {
        report.inspection = Inspection::Limited;
        report.warnings.push(WarningItem::new(
            "zip.truncated_or_corrupted",
            "Failed to find ZIP End of Central Directory record",
        ));
        return Ok(None);
    }

    if cd_size > options.max_zip_directory_window {
        report.inspection = Inspection::Limited;
        report.warnings.push(WarningItem::new(
            "budget.zip_directory_window_exceeded",
            format!(
                "ZIP central directory size ({}) exceeded budget limit ({})",
                cd_size, options.max_zip_directory_window
            ),
        ));
        return Ok(None);
    }

    if cd_total_entries > options.max_entries {
        report.inspection = Inspection::Limited;
        report.warnings.push(WarningItem::new(
            "budget.entries_exceeded",
            format!(
                "ZIP entry count ({}) exceeded budget limit ({})",
                cd_total_entries, options.max_entries
            ),
        ));
        return Ok(None);
    }

    if cd_offset + (cd_size as u64) > file_len {
        report.inspection = Inspection::Limited;
        report.warnings.push(WarningItem::new(
            "zip.truncated_or_corrupted",
            "Central directory offset and size exceed file length",
        ));
        return Ok(None);
    }

    let mut has_content_types = false;
    let mut has_rels = false;
    let mut has_ooxml_subdirs = false;
    let mut has_apk_manifest = false;
    let mut has_apk_dex = false;
    let mut apk_manifest_entry: Option<ZipEntryMeta> = None;
    let mut has_aab_bundle = false;
    let mut has_npz_array = false;
    let mut has_ipa_payload = false;
    let mut has_jar_manifest = false;
    let mut has_kmz_kml = false;
    let mut has_mimetype = false;
    let mut has_epub_container = false;
    let mut has_whl_dist_info = false;
    let mut has_vsix_manifest = false;
    let mut has_nupkg_nuspec = false;
    let mut has_xpi_manifest = false;
    let mut has_msix_manifest = false;
    let mut content_types_entry: Option<ZipEntryMeta> = None;
    let mut rels_entry: Option<ZipEntryMeta> = None;
    let mut xml_entries: Vec<ZipEntryMeta> = Vec::new();
    let mut xml_entries_capped = false;
    let mut any_zip64 = cd_total_entries == 0xFFFF || cd_offset == u64::from(u32::MAX);

    // Walk central directory headers to check for duplicate entry names or conflicting paths
    if let Err(e) = reader.seek(SeekFrom::Start(cd_offset)) {
        if reader.is_budget_exceeded() {
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "budget.source_read_exceeded",
                "Total source read budget exceeded during detection",
            ));
            return Ok(None);
        }
        return Err(DetectError::Io(e));
    }

    let mut cd_bytes = vec![0u8; cd_size];
    if let Err(e) = reader.read_exact(&mut cd_bytes) {
        if reader.is_budget_exceeded() {
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "budget.source_read_exceeded",
                "Total source read budget exceeded during detection",
            ));
            return Ok(None);
        }
        report.inspection = Inspection::Limited;
        report.warnings.push(WarningItem::new(
            "zip.truncated_or_corrupted",
            format!("Failed to read ZIP central directory: {}", e),
        ));
        return Ok(None);
    }

    let mut pos = 0;
    let mut actual_entries = 0usize;
    let mut seen_raw_names: FastNameSet<&[u8]> = FastNameSet::with_expected_len(cd_total_entries);
    let mut seen_normalized_names: FastNameSet<std::borrow::Cow<str>> =
        FastNameSet::with_expected_len(cd_total_entries);
    let mut duplicate_entry: Option<String> = None;
    let mut conflicting_norm: Option<String> = None;
    let mut overlap_or_oob = false;
    let mut corrupted_cd = false;

    while pos + 46 <= cd_bytes.len() {
        if &cd_bytes[pos..pos + 4] != b"PK\x01\x02" {
            corrupted_cd = true;
            break;
        }
        actual_entries += 1;
        if actual_entries > options.max_entries {
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "budget.entries_exceeded",
                format!(
                    "ZIP entry count exceeded budget limit ({})",
                    options.max_entries
                ),
            ));
            return Ok(None);
        }
        let flags = u16::from_le_bytes([cd_bytes[pos + 8], cd_bytes[pos + 9]]);
        let compression_method = u16::from_le_bytes([cd_bytes[pos + 10], cd_bytes[pos + 11]]);
        let compressed_size = u32::from_le_bytes([
            cd_bytes[pos + 20],
            cd_bytes[pos + 21],
            cd_bytes[pos + 22],
            cd_bytes[pos + 23],
        ]);
        let uncompressed_size = u32::from_le_bytes([
            cd_bytes[pos + 24],
            cd_bytes[pos + 25],
            cd_bytes[pos + 26],
            cd_bytes[pos + 27],
        ]);
        let fn_len = u16::from_le_bytes([cd_bytes[pos + 28], cd_bytes[pos + 29]]) as usize;
        let extra_len = u16::from_le_bytes([cd_bytes[pos + 30], cd_bytes[pos + 31]]) as usize;
        let comment_len = u16::from_le_bytes([cd_bytes[pos + 32], cd_bytes[pos + 33]]) as usize;
        let local_header_offset = u32::from_le_bytes([
            cd_bytes[pos + 42],
            cd_bytes[pos + 43],
            cd_bytes[pos + 44],
            cd_bytes[pos + 45],
        ]);
        if local_header_offset == u32::MAX
            || compressed_size == u32::MAX
            || uncompressed_size == u32::MAX
        {
            any_zip64 = true;
        }
        if local_header_offset != u32::MAX && cd_offset > 0 {
            let lfh_off = local_header_offset as u64;
            if lfh_off >= cd_offset
                || (compressed_size != u32::MAX
                    && lfh_off
                        .saturating_add(30)
                        .saturating_add(fn_len as u64)
                        .saturating_add(compressed_size as u64)
                        > cd_offset)
            {
                overlap_or_oob = true;
                break;
            }
        }
        let fn_start = pos + 46;
        let Some(fn_end) = fn_start.checked_add(fn_len) else {
            corrupted_cd = true;
            break;
        };
        let Some(next_pos) = fn_end
            .checked_add(extra_len)
            .and_then(|v| v.checked_add(comment_len))
        else {
            corrupted_cd = true;
            break;
        };
        if next_pos > cd_bytes.len() {
            corrupted_cd = true;
            break;
        }
        let raw_bytes = &cd_bytes[fn_start..fn_end];
        if !seen_raw_names.insert(raw_bytes) && duplicate_entry.is_none() {
            duplicate_entry = Some(String::from_utf8_lossy(raw_bytes).to_string());
        }

        let make_meta = |name_bytes: &[u8]| -> Option<ZipEntryMeta> {
            if (flags & 0x0001) != 0
                || (compression_method != 0 && compression_method != 8)
                || local_header_offset == u32::MAX
                || compressed_size == u32::MAX
                || uncompressed_size == u32::MAX
            {
                return None;
            }
            std::str::from_utf8(name_bytes)
                .ok()
                .map(|name| ZipEntryMeta {
                    name: name.to_string(),
                    compression_method,
                    flags,
                    compressed_size,
                    local_header_offset,
                })
        };

        // Check OOXML, package, and container indicators on raw bytes without string conversion
        if raw_bytes == b"[Content_Types].xml" {
            has_content_types = true;
            if content_types_entry.is_none() {
                content_types_entry = make_meta(raw_bytes);
            }
        } else if raw_bytes == b"_rels/.rels" {
            has_rels = true;
            if rels_entry.is_none() {
                rels_entry = make_meta(raw_bytes);
            }
        } else {
            if raw_bytes.len() > 4 && raw_bytes[raw_bytes.len() - 4..].eq_ignore_ascii_case(b".xml")
            {
                if xml_entries.len() < MAX_CACHED_XML_ENTRIES {
                    if let Some(meta) = make_meta(raw_bytes) {
                        xml_entries.push(meta);
                    } else {
                        xml_entries_capped = true;
                    }
                } else {
                    xml_entries_capped = true;
                }
            }
            if raw_bytes.starts_with(b"word/")
                || raw_bytes.starts_with(b"xl/")
                || raw_bytes.starts_with(b"ppt/")
                || raw_bytes.starts_with(b"customXml/")
                || raw_bytes.starts_with(b"docProps/")
            {
                has_ooxml_subdirs = true;
            } else if raw_bytes == b"AndroidManifest.xml" {
                has_apk_manifest = true;
                if apk_manifest_entry.is_none() {
                    apk_manifest_entry = make_meta(raw_bytes);
                }
            } else if raw_bytes == b"classes.dex" {
                has_apk_dex = true;
            } else if raw_bytes == b"BundleConfig.pb" {
                has_aab_bundle = true;
            } else if raw_bytes.ends_with(b".npy") {
                has_npz_array = true;
            } else if raw_bytes.starts_with(b"Payload/") && raw_bytes.ends_with(b".app/Info.plist")
            {
                has_ipa_payload = true;
            } else if raw_bytes == b"META-INF/MANIFEST.MF" {
                has_jar_manifest = true;
            } else if raw_bytes == b"doc.kml" {
                has_kmz_kml = true;
            } else if raw_bytes == b"mimetype" {
                has_mimetype = true;
            } else if raw_bytes == b"META-INF/container.xml" {
                has_epub_container = true;
            } else if raw_bytes.ends_with(b".dist-info/WHEEL") {
                has_whl_dist_info = true;
            } else if raw_bytes == b"extension.vsixmanifest" {
                has_vsix_manifest = true;
            } else if raw_bytes.ends_with(b".nuspec") && !raw_bytes.contains(&b'/') {
                has_nupkg_nuspec = true;
            } else if raw_bytes == b"install.rdf" || raw_bytes == b"META-INF/mozilla.rsa" {
                has_xpi_manifest = true;
            } else if raw_bytes == b"AppxManifest.xml" {
                has_msix_manifest = true;
            }
        }

        let norm: std::borrow::Cow<'_, str> = match std::str::from_utf8(raw_bytes) {
            Ok(s) => normalize_opc_path_cow(s),
            Err(_) => {
                std::borrow::Cow::Owned(normalize_opc_path(&String::from_utf8_lossy(raw_bytes)))
            }
        };
        if !norm.is_empty()
            && !seen_normalized_names.insert(norm.clone())
            && conflicting_norm.is_none()
            && duplicate_entry.is_none()
        {
            conflicting_norm = Some(norm.into_owned());
        }
        pos = next_pos;
    }

    if !overlap_or_oob
        && (pos != cd_bytes.len()
            || (cd_total_entries != 0xFFFF && actual_entries != cd_total_entries))
    {
        corrupted_cd = true;
    }

    if overlap_or_oob {
        report.inspection = Inspection::Limited;
        report.warnings.push(WarningItem::new(
            "zip.truncated_or_corrupted",
            "ZIP local file header or compressed payload overlaps central directory",
        ));
        return Ok(None);
    }

    if corrupted_cd {
        report.inspection = Inspection::Limited;
        report.warnings.push(WarningItem::new(
            "zip.truncated_or_corrupted",
            "ZIP central directory is truncated or contains invalid entry headers",
        ));
        return Ok(None);
    }

    if let Some(dup) = duplicate_entry {
        report.identification = Identification::ContainerOnly;
        report.format_id = None;
        report.recommended_extension = None;
        report.inspection = Inspection::Ambiguous;
        report.warnings.push(WarningItem::new(
            "zip.conflicting_entries",
            format!("Duplicate ZIP entry detected: '{}'", dup),
        ));
        return Ok(None);
    }

    if let Some(conf) = conflicting_norm {
        report.identification = Identification::ContainerOnly;
        report.format_id = None;
        report.recommended_extension = None;
        report.inspection = Inspection::Ambiguous;
        report.warnings.push(WarningItem::new(
            "zip.conflicting_entries",
            format!("Conflicting normalized ZIP path detected: '{}'", conf),
        ));
        return Ok(None);
    }

    Ok(Some(ZipCdPreCheckResult {
        cd_total_entries: actual_entries,
        cd_size,
        cd_offset,
        is_zip64: any_zip64,
        has_content_types,
        has_rels,
        has_ooxml_subdirs,
        has_apk_manifest,
        has_apk_dex,
        apk_manifest_entry,
        has_aab_bundle,
        has_npz_array,
        has_ipa_payload,
        has_jar_manifest,
        has_kmz_kml,
        has_mimetype,
        has_epub_container,
        has_whl_dist_info,
        has_vsix_manifest,
        has_nupkg_nuspec,
        has_xpi_manifest,
        has_msix_manifest,
        content_types_entry,
        rels_entry,
        xml_entries,
        xml_entries_capped,
    }))
}
