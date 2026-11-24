use std::collections::HashSet;
use std::io::{Read, Seek, SeekFrom};
use crate::budget::{BudgetReader, DetectOptions};
use crate::model::{DetectError, DetectionReport, Identification, Inspection, WarningItem};
use super::opc::{normalize_opc_path, normalize_opc_path_cow};

pub struct ZipCdPreCheckResult {
    pub cd_total_entries: usize,
    pub cd_size: usize,
    pub cd_offset: u64,
    pub has_content_types: bool,
    pub has_rels: bool,
    pub has_ooxml_subdirs: bool,
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
        if reader.seek(SeekFrom::Start(fast_seek_pos)).is_ok() {
            if let Ok(n) = reader.read(&mut fast_buf[..fast_probe_len]) {
                let slice = &fast_buf[..n];
                for i in (0..=slice.len().saturating_sub(22)).rev() {
                    if &slice[i..i + 4] == b"PK\x05\x06" {
                        let total_ent =
                            u16::from_le_bytes([slice[i + 10], slice[i + 11]]) as usize;
                        let size = u32::from_le_bytes([
                            slice[i + 12],
                            slice[i + 13],
                            slice[i + 14],
                            slice[i + 15],
                        ]) as usize;
                        let offset = u32::from_le_bytes([
                            slice[i + 16],
                            slice[i + 17],
                            slice[i + 18],
                            slice[i + 19],
                        ]) as u64;
                        cd_total_entries = total_ent;
                        cd_size = size;
                        cd_offset = offset;
                        eocd_found = true;
                        break;
                    }
                }
            }
        }

        // If not found in the last 1024 bytes and file is larger, fall back to full tail search
        if !eocd_found && file_len > 1024 {
            let seek_pos = file_len - tail_search_len as u64;
            if reader.seek(SeekFrom::Start(seek_pos)).is_ok() {
                let mut tail_buf = vec![0u8; tail_search_len];
                if let Ok(n) = reader.read(&mut tail_buf) {
                    tail_buf.truncate(n);
                    // Scan backwards for PK\x05\x06
                    for i in (0..=tail_buf.len().saturating_sub(22)).rev() {
                        if &tail_buf[i..i + 4] == b"PK\x05\x06" {
                            let total_ent =
                                u16::from_le_bytes([tail_buf[i + 10], tail_buf[i + 11]]) as usize;
                            let size = u32::from_le_bytes([
                                tail_buf[i + 12],
                                tail_buf[i + 13],
                                tail_buf[i + 14],
                                tail_buf[i + 15],
                            ]) as usize;
                            let offset = u32::from_le_bytes([
                                tail_buf[i + 16],
                                tail_buf[i + 17],
                                tail_buf[i + 18],
                                tail_buf[i + 19],
                            ]) as u64;
                            cd_total_entries = total_ent;
                            cd_size = size;
                            cd_offset = offset;
                            eocd_found = true;
                            break;
                        }
                    }
                }
            }
        }
    }

    let mut has_content_types = false;
    let mut has_rels = false;
    let mut has_ooxml_subdirs = false;

    if eocd_found {
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

        // Walk central directory headers to check for duplicate entry names or conflicting paths
        if cd_offset + (cd_size as u64) <= file_len
            && reader.seek(SeekFrom::Start(cd_offset)).is_ok()
        {
            let mut cd_bytes = vec![0u8; cd_size];
            if let Ok(n) = reader.read(&mut cd_bytes) {
                cd_bytes.truncate(n);
                let mut pos = 0;
                let mut seen_raw_names: HashSet<&[u8]> = HashSet::with_capacity(cd_total_entries);
                let mut seen_normalized_names: HashSet<std::borrow::Cow<str>> =
                    HashSet::with_capacity(cd_total_entries);
                let mut duplicate_entry: Option<String> = None;
                let mut conflicting_norm: Option<String> = None;

                while pos + 46 <= cd_bytes.len() {
                    if &cd_bytes[pos..pos + 4] != b"PK\x01\x02" {
                        break;
                    }
                    let fn_len =
                        u16::from_le_bytes([cd_bytes[pos + 28], cd_bytes[pos + 29]]) as usize;
                    let extra_len =
                        u16::from_le_bytes([cd_bytes[pos + 30], cd_bytes[pos + 31]]) as usize;
                    let comment_len =
                        u16::from_le_bytes([cd_bytes[pos + 32], cd_bytes[pos + 33]]) as usize;
                    let fn_start = pos + 46;
                    let fn_end = fn_start + fn_len;
                    if fn_end > cd_bytes.len() {
                        break;
                    }
                    let raw_bytes = &cd_bytes[fn_start..fn_end];
                    if !seen_raw_names.insert(raw_bytes) && duplicate_entry.is_none() {
                        duplicate_entry = Some(String::from_utf8_lossy(raw_bytes).to_string());
                    }

                    // Check OOXML member indicators on raw bytes without string conversion
                    if raw_bytes == b"[Content_Types].xml" {
                        has_content_types = true;
                    } else if raw_bytes == b"_rels/.rels" {
                        has_rels = true;
                    } else if raw_bytes.starts_with(b"word/")
                        || raw_bytes.starts_with(b"xl/")
                        || raw_bytes.starts_with(b"ppt/")
                        || raw_bytes.starts_with(b"customXml/")
                        || raw_bytes.starts_with(b"docProps/")
                    {
                        has_ooxml_subdirs = true;
                    }

                    let norm: std::borrow::Cow<'_, str> = match std::str::from_utf8(raw_bytes) {
                        Ok(s) => normalize_opc_path_cow(s),
                        Err(_) => std::borrow::Cow::Owned(normalize_opc_path(&String::from_utf8_lossy(raw_bytes))),
                    };
                    if !norm.is_empty()
                        && !seen_normalized_names.insert(norm.clone())
                        && conflicting_norm.is_none()
                        && duplicate_entry.is_none()
                    {
                        conflicting_norm = Some(norm.into_owned());
                    }
                    pos = fn_end + extra_len + comment_len;
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
            }
        }
    }

    Ok(Some(ZipCdPreCheckResult {
        cd_total_entries,
        cd_size,
        cd_offset,
        has_content_types,
        has_rels,
        has_ooxml_subdirs,
    }))
}
