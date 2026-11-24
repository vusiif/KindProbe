use std::io::{Read, Seek, SeekFrom};
use crate::budget::{BudgetReader, DetectOptions};
use crate::extension::find_registered_format;
use crate::model::DetectionReport;
use super::archives::{probe_archive_signatures, ArchiveMatch};
use super::cfb::{detect_cfb, CFB_MAGIC};
use super::databases::{probe_database_signatures, DatabaseMatch};
use super::disks::{probe_disk_signatures, DiskMatch};
use super::executables::{probe_executable_signatures, ExecutableMatch};
use super::fonts::{probe_font_signatures, FontMatch};
use super::images::{probe_image_signatures, ImageMatch};
use super::media::{probe_media_signatures, MediaMatch};
use super::rules::probe_rule_by_format;
use super::signatures::{make_pdf_report, PDF_SIGNATURE};
use super::text::probe_text;
use super::zip_ooxml::detect_zip_ooxml;

fn check_zip_or_cfb<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    header: &[u8],
    options: &DetectOptions,
    fid: &str,
) -> Result<Option<DetectionReport>, crate::model::DetectError> {
    let is_zip = matches!(fid, "zip" | "docx" | "xlsx" | "pptx" | "jar" | "apk" | "epub" | "odt" | "ods" | "odp" | "kmz" | "ipa");
    if is_zip && header.len() >= 4 && (header.starts_with(b"PK\x03\x04") || header.starts_with(b"PK\x05\x06")) {
        let _ = reader.seek(SeekFrom::Start(0));
        let rep = detect_zip_ooxml(reader, options)?;
        if rep.format_id.as_deref() == Some(fid) || fid == "zip" { return Ok(Some(rep)); }
        let _ = reader.seek(SeekFrom::Start(0));
    } else if matches!(fid, "cfb" | "doc" | "xls" | "ppt" | "msi") && header.len() >= 8 && header.starts_with(CFB_MAGIC) {
        let _ = reader.seek(SeekFrom::Start(0));
        let rep = detect_cfb(reader, options)?;
        if rep.format_id.as_deref() == Some(fid) || fid == "cfb" { return Ok(Some(rep)); }
        let _ = reader.seek(SeekFrom::Start(0));
    }
    Ok(None)
}

fn probe_by_fid(header: &[u8], fid: &str) -> Option<DetectionReport> {
    match fid {
        "png" | "jpeg" | "gif" => match probe_image_signatures(header) { ImageMatch::Confirmed(r) => Some(*r), _ => None },
        "7z" | "rar" | "tar.gz" | "gzip" | "bzip2" | "tar" | "xz" | "zstd" => match probe_archive_signatures(header) { ArchiveMatch::Confirmed(r) => Some(*r), _ => None },
        "mp4" | "mov" | "webm" | "mkv" | "wav" | "webp" | "avi" | "mp3" => match probe_media_signatures(header) { MediaMatch::Confirmed(r) => Some(*r), _ => None },
        "elf" | "pe" | "wasm" | "mach-o" => match probe_executable_signatures(header) { ExecutableMatch::Confirmed(r) => Some(*r), _ => None },
        "ttf" | "otf" | "woff" | "woff2" => match probe_font_signatures(header) { FontMatch::Confirmed(r) => Some(*r), _ => None },
        "sqlite3" | "pcap" | "pcapng" | "parquet" | "avro" => match probe_database_signatures(header) { DatabaseMatch::Confirmed(r) => Some(*r), _ => None },
        "qcow2" | "vmdk" | "vhd" | "vhdx" => match probe_disk_signatures(header) { DiskMatch::Confirmed(r) => Some(*r), _ => None },
        "json" | "xml" | "html" | "yaml" | "toml" | "csv" | "txt" => probe_text(header),
        _ => probe_rule_by_format(header, fid),
    }
}

/// Optimistically validates whether the input stream matches the extension hint.
pub fn probe_fast_path<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    header: &[u8],
    options: &DetectOptions,
) -> Result<Option<DetectionReport>, crate::model::DetectError> {
    let hint = match &options.extension_hint {
        Some(h) if !h.trim().is_empty() => h.trim().to_ascii_lowercase(),
        _ => return Ok(None),
    };
    let fid = find_registered_format(&hint).unwrap_or(hint.as_str());
    if matches!(fid, "zip" | "docx" | "xlsx" | "pptx" | "jar" | "apk" | "epub" | "odt" | "ods" | "odp" | "kmz" | "ipa" | "cfb" | "doc" | "xls" | "ppt" | "msi") {
        return check_zip_or_cfb(reader, header, options, fid);
    }
    if fid == "pdf" {
        let lim = header.len().min(1024);
        if lim >= 5 && header[..lim].windows(5).any(|w| w == PDF_SIGNATURE) {
            return Ok(Some(make_pdf_report()));
        }
        return Ok(None);
    }
    if let Some(r) = probe_by_fid(header, fid) {
        if r.format_id.as_deref() == Some(fid) { return Ok(Some(r)); }
    }
    Ok(None)
}
