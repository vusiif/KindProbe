use std::borrow::Cow;
use std::io::{Read, Seek, SeekFrom};
use crate::budget::{BudgetReader, DetectOptions};
use crate::extension::find_registered_format;
use crate::model::{DetectionCore, DetectionReport, ExtensionStatus};
use super::archives::{probe_archive_signatures, probe_archive_signatures_core, ArchiveMatch, ArchiveMatchCore};
use super::cfb::{detect_cfb, CFB_MAGIC};
use super::databases::{probe_database_signatures, probe_database_signatures_core, DatabaseMatch, DatabaseMatchCore};
use super::disks::{probe_disk_signatures, probe_disk_signatures_core, DiskMatch, DiskMatchCore};
use super::ebooks::{probe_ebook_signatures, probe_ebook_signatures_core, EbookMatch, EbookMatchCore};
use super::executables::{probe_executable_signatures, probe_executable_signatures_core, ExecutableMatch, ExecutableMatchCore};
use super::fonts::{probe_font_signatures, probe_font_signatures_core, FontMatch, FontMatchCore};
use super::media::{probe_media_signatures, probe_media_signatures_core, MediaMatch, MediaMatchCore};
use super::rules::{probe_rule_by_format, probe_rule_by_format_core};
use super::signatures::{make_pdf_core, make_pdf_report, PDF_SIGNATURE};
use super::text::{probe_text, probe_text_core};
use super::zip_ooxml::detect_zip_ooxml;

fn normalize_hint(options: &DetectOptions) -> Option<Cow<'_, str>> {
    match &options.extension_hint {
        Some(h) if !h.trim().is_empty() => {
            let trimmed = h.trim();
            if trimmed.bytes().any(|b| b.is_ascii_uppercase()) {
                Some(Cow::Owned(trimmed.to_ascii_lowercase()))
            } else {
                Some(Cow::Borrowed(trimmed))
            }
        }
        _ => None,
    }
}

fn is_container_fid(fid: &str) -> bool {
    matches!(
        fid,
        "zip" | "docx" | "xlsx" | "pptx" | "docm" | "xlsm" | "pptm"
            | "jar" | "apk" | "epub" | "odt" | "ods" | "odp" | "kmz" | "ipa"
            | "cfb" | "doc" | "xls" | "ppt" | "msi"
    )
}

fn matches_pdf_prefix_or_window(header: &[u8]) -> bool {
    if header.starts_with(PDF_SIGNATURE) {
        return true;
    }
    let lim = header.len().min(1024);
    lim >= 5 && header[..lim].contains(&b'%') && header[..lim].windows(5).any(|w| w == PDF_SIGNATURE)
}

fn check_zip_or_cfb<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    header: &[u8],
    options: &DetectOptions,
    fid: &str,
) -> Result<Option<DetectionReport>, crate::model::DetectError> {
    let is_zip = matches!(
        fid,
        "zip" | "docx" | "xlsx" | "pptx" | "docm" | "xlsm" | "pptm"
            | "jar" | "apk" | "epub" | "odt" | "ods" | "odp" | "kmz" | "ipa"
    );
    if is_zip && header.len() >= 4 && (header.starts_with(b"PK\x03\x04") || header.starts_with(b"PK\x05\x06")) {
        let _ = reader.seek(SeekFrom::Start(0));
        let rep = detect_zip_ooxml(reader, options)?;
        return Ok(Some(rep));
    } else if matches!(fid, "cfb" | "doc" | "xls" | "ppt" | "msi") && header.len() >= 8 && header.starts_with(CFB_MAGIC) {
        let _ = reader.seek(SeekFrom::Start(0));
        let rep = detect_cfb(reader, options)?;
        return Ok(Some(rep));
    }
    Ok(None)
}

fn probe_by_fid_core(header: &[u8], fid: &str) -> Option<DetectionCore> {
    match fid {
        "png" => match super::images::png::probe_png(header) {
            super::images::png::PngProbeResult::Confirmed { has_ihdr } => Some(super::images::png::make_png_core(has_ihdr)),
            _ => None,
        },
        "jpeg" => match super::images::jpeg::probe_jpeg(header) {
            super::images::jpeg::JpegProbeResult::Confirmed { has_segment } => Some(super::images::jpeg::make_jpeg_core(has_segment)),
            _ => None,
        },
        "gif" => match super::images::gif::probe_gif(header) {
            super::images::gif::GifProbeResult::Confirmed { has_screen_desc } => Some(super::images::gif::make_gif_core(has_screen_desc)),
            _ => None,
        },
        "7z" | "rar" | "tar.gz" | "svgz" | "gzip" | "bzip2" | "tar" | "xz" | "zstd" => match probe_archive_signatures_core(header) {
            ArchiveMatchCore::Confirmed(c) => Some(c),
            _ => None,
        },
        "mp4" | "mov" | "m4a" | "webm" | "mkv" | "wav" | "webp" | "avi" | "mp3" => match probe_media_signatures_core(header) {
            MediaMatchCore::Confirmed(c) => Some(c),
            _ => None,
        },
        "elf" | "pe" | "wasm" | "macho" | "class" => match probe_executable_signatures_core(header) {
            ExecutableMatchCore::Confirmed(c) => Some(c),
            _ => None,
        },
        "ttf" | "otf" | "woff" | "woff2" => match probe_font_signatures_core(header) {
            FontMatchCore::Confirmed(c) => Some(c),
            _ => None,
        },
        "sqlite3" | "gpkg" | "pcap" | "pcapng" | "parquet" | "avro" => match probe_database_signatures_core(header) {
            DatabaseMatchCore::Confirmed(c) => Some(c),
            _ => None,
        },
        "qcow2" | "vmdk" | "vhd" | "vhdx" | "iso" => match probe_disk_signatures_core(header) {
            DiskMatchCore::Confirmed(c) => Some(c),
            _ => None,
        },
        "mobi" => match probe_ebook_signatures_core(header) {
            EbookMatchCore::Confirmed(c) => Some(c),
            _ => None,
        },
        "json" | "geojson" | "xml" | "html" | "svg" | "kml" | "gpx" | "dae" | "plist"
        | "yaml" | "toml" | "csv" | "tsv" | "txt" | "rs" | "py" | "c" | "cpp"
        | "java" | "go" | "js" | "sh" | "v" | "md" | "diff" | "bat" => probe_text_core(header),
        _ => probe_rule_by_format_core(header, fid),
    }
}

fn probe_by_fid(header: &[u8], fid: &str) -> Option<DetectionReport> {
    match fid {
        "png" => match super::images::png::probe_png(header) {
            super::images::png::PngProbeResult::Confirmed { has_ihdr } => Some(super::images::png::make_png_report(has_ihdr)),
            _ => None,
        },
        "jpeg" => match super::images::jpeg::probe_jpeg(header) {
            super::images::jpeg::JpegProbeResult::Confirmed { has_segment } => Some(super::images::jpeg::make_jpeg_report(has_segment)),
            _ => None,
        },
        "gif" => match super::images::gif::probe_gif(header) {
            super::images::gif::GifProbeResult::Confirmed { has_screen_desc } => Some(super::images::gif::make_gif_report(has_screen_desc)),
            _ => None,
        },
        "7z" | "rar" | "tar.gz" | "svgz" | "gzip" | "bzip2" | "tar" | "xz" | "zstd" => match probe_archive_signatures(header) { ArchiveMatch::Confirmed(r) => Some(*r), _ => None },
        "mp4" | "mov" | "m4a" | "webm" | "mkv" | "wav" | "webp" | "avi" | "mp3" => match probe_media_signatures(header) { MediaMatch::Confirmed(r) => Some(*r), _ => None },
        "elf" | "pe" | "wasm" | "macho" | "class" => match probe_executable_signatures(header) { ExecutableMatch::Confirmed(r) => Some(*r), _ => None },
        "ttf" | "otf" | "woff" | "woff2" => match probe_font_signatures(header) { FontMatch::Confirmed(r) => Some(*r), _ => None },
        "sqlite3" | "gpkg" | "pcap" | "pcapng" | "parquet" | "avro" => match probe_database_signatures(header) { DatabaseMatch::Confirmed(r) => Some(*r), _ => None },
        "qcow2" | "vmdk" | "vhd" | "vhdx" | "iso" => match probe_disk_signatures(header) { DiskMatch::Confirmed(r) => Some(*r), _ => None },
        "mobi" => match probe_ebook_signatures(header) { EbookMatch::Confirmed(r) => Some(*r), _ => None },
        "json" | "geojson" | "xml" | "html" | "svg" | "kml" | "gpx" | "dae" | "plist"
        | "yaml" | "toml" | "csv" | "tsv" | "txt" | "rs" | "py" | "c" | "cpp"
        | "java" | "go" | "js" | "sh" | "v" | "md" | "diff" | "bat" => probe_text(header),
        _ => probe_rule_by_format(header, fid),
    }
}

pub enum FastPathCoreOutcome {
    Core(DetectionCore),
    Report(DetectionReport),
    None,
}

/// Zero-allocation fast path probe for streams, delegating only ZIP/CFB containers to full parsers.
pub fn probe_fast_path_core<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    header: &[u8],
    options: &DetectOptions,
) -> Result<FastPathCoreOutcome, crate::model::DetectError> {
    let Some(hint) = normalize_hint(options) else {
        return Ok(FastPathCoreOutcome::None);
    };
    let fid = find_registered_format(&hint).unwrap_or(&hint);
    if is_container_fid(fid) {
        return Ok(match check_zip_or_cfb(reader, header, options, fid)? {
            Some(rep) => FastPathCoreOutcome::Report(rep),
            None => FastPathCoreOutcome::None,
        });
    }
    if fid == "pdf" {
        return Ok(if matches_pdf_prefix_or_window(header) {
            FastPathCoreOutcome::Core(make_pdf_core())
        } else {
            FastPathCoreOutcome::None
        });
    }
    if let Some(core) = probe_by_fid_core(header, fid) {
        if core.format_id == Some(fid) {
            return Ok(FastPathCoreOutcome::Core(core));
        }
    }
    Ok(FastPathCoreOutcome::None)
}

/// Optimistically validates whether the input stream matches the extension hint.
pub fn probe_fast_path<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    header: &[u8],
    options: &DetectOptions,
) -> Result<Option<DetectionReport>, crate::model::DetectError> {
    let Some(hint) = normalize_hint(options) else {
        return Ok(None);
    };
    let fid = find_registered_format(&hint).unwrap_or(&hint);
    if is_container_fid(fid) {
        return check_zip_or_cfb(reader, header, options, fid);
    }
    if fid == "pdf" {
        return Ok(if matches_pdf_prefix_or_window(header) {
            Some(make_pdf_report())
        } else {
            None
        });
    }
    if let Some(r) = probe_by_fid(header, fid) {
        if r.format_id.as_deref() == Some(fid) {
            return Ok(Some(r));
        }
    }
    Ok(None)
}

/// Slice-only fast path probe returning a zero-allocation DetectionCore.
pub fn probe_fast_path_slice_core(
    header: &[u8],
    options: &DetectOptions,
) -> Result<Option<DetectionCore>, crate::model::DetectError> {
    let Some(hint) = normalize_hint(options) else {
        return Ok(None);
    };
    let fid = find_registered_format(&hint).unwrap_or(&hint);
    if is_container_fid(fid) {
        return Ok(None);
    }
    if fid == "pdf" {
        return Ok(if matches_pdf_prefix_or_window(header) {
            Some(make_pdf_core())
        } else {
            None
        });
    }
    if let Some(core) = probe_by_fid_core(header, fid) {
        if core.format_id == Some(fid) {
            return Ok(Some(core));
        }
    }
    Ok(None)
}

/// Slice-only fast path probe (skips container readers).
pub fn probe_fast_path_slice(
    header: &[u8],
    options: &DetectOptions,
) -> Result<Option<DetectionReport>, crate::model::DetectError> {
    match probe_fast_path_slice_core(header, options)? {
        Some(core) => Ok(Some(core.to_report(Default::default(), ExtensionStatus::Undetermined))),
        None => Ok(None),
    }
}
