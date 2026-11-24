use std::borrow::Cow;
use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};
use super::archives::{probe_archive_signatures, probe_archive_signatures_core, ArchiveMatch, ArchiveMatchCore};
use super::databases::{probe_database_signatures, probe_database_signatures_core, DatabaseMatch, DatabaseMatchCore};
use super::disks::{probe_disk_signatures, probe_disk_signatures_core, DiskMatch, DiskMatchCore};
use super::ebooks::{probe_ebook_signatures, probe_ebook_signatures_core, EbookMatch, EbookMatchCore};
use super::executables::{probe_executable_signatures, probe_executable_signatures_core, ExecutableMatch, ExecutableMatchCore};
use super::fonts::{probe_font_signatures, probe_font_signatures_core, FontMatch, FontMatchCore};
use super::images::{probe_image_signatures, probe_image_signatures_core, ImageMatch, ImageMatchCore};
use super::media::{probe_media_signatures, probe_media_signatures_core, MediaMatch, MediaMatchCore};
use super::rules::{
    evaluate_indexed_rules, evaluate_indexed_rules_core, RuleMatch, RuleMatchCore,
};

pub const PDF_SIGNATURE: &[u8] = b"%PDF-";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureMatchCore {
    Confirmed(DetectionCore),
    Truncated(Cow<'static, str>),
    None,
}

pub enum SignatureMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn probe_signatures_core(header: &[u8]) -> SignatureMatchCore {
    if header.is_empty() {
        return SignatureMatchCore::None;
    }

    let b0 = header[0];

    // 1. First-byte O(1) jump table for offset-0 signatures
    match b0 {
        0x89 => {
            // PNG
            match probe_image_signatures_core(header) {
                ImageMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                ImageMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                ImageMatchCore::None => (),
            }
        }
        0xFF => {
            // JPEG or MP3 frame sync
            match probe_image_signatures_core(header) {
                ImageMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                ImageMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                ImageMatchCore::None => (),
            }
            match probe_media_signatures_core(header) {
                MediaMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                MediaMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                MediaMatchCore::None => (),
            }
        }
        b'G' => {
            // GIF
            match probe_image_signatures_core(header) {
                ImageMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                ImageMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                ImageMatchCore::None => (),
            }
        }
        b'7' => {
            // 7z
            match probe_archive_signatures_core(header) {
                ArchiveMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                ArchiveMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                ArchiveMatchCore::None => (),
            }
        }
        b'R' => {
            // RAR or RIFF (WAV, WebP, AVI)
            match probe_archive_signatures_core(header) {
                ArchiveMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                ArchiveMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                ArchiveMatchCore::None => (),
            }
            match probe_media_signatures_core(header) {
                MediaMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                MediaMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                MediaMatchCore::None => (),
            }
        }
        0x1F | b'B' | 0xFD | 0x28 => {
            // Gzip (0x1F), Bzip2 ('B'), XZ (0xFD), Zstd (0x28)
            match probe_archive_signatures_core(header) {
                ArchiveMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                ArchiveMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                ArchiveMatchCore::None => (),
            }
        }
        0x1A | b'I' => {
            // WebM / MKV (0x1A) or MP3 ID3 ('I')
            match probe_media_signatures_core(header) {
                MediaMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                MediaMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                MediaMatchCore::None => (),
            }
        }
        0x7F | b'M' | 0xFE | 0xCE | 0xCF | 0xCA | 0xBE | 0xED => {
            // ELF (0x7F), PE ('M'), Mach-O / Class (0xFE, 0xCE, 0xCF, 0xCA, 0xBE, 0xED)
            match probe_executable_signatures_core(header) {
                ExecutableMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                ExecutableMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                ExecutableMatchCore::None => (),
            }
            if b0 == b'M' {
                // 0x4D is also the first byte of modified PCAP (0x4D, 0x3C, 0xB2, 0xA1)
                match probe_database_signatures_core(header) {
                    DatabaseMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                    DatabaseMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                    DatabaseMatchCore::None => (),
                }
            }
        }
        0x00 => {
            // WASM (0x00), TTF (0x00), or MP4 box size prefix (\0\0\0)
            if header.starts_with(&[0, 0, 0]) {
                match probe_media_signatures_core(header) {
                    MediaMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                    MediaMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                    MediaMatchCore::None => (),
                }
            }
            match probe_executable_signatures_core(header) {
                ExecutableMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                ExecutableMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                ExecutableMatchCore::None => (),
            }
            match probe_font_signatures_core(header) {
                FontMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                FontMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                FontMatchCore::None => (),
            }
        }
        b'f' => {
            if header.starts_with(b"ftyp") {
                match probe_media_signatures_core(header) {
                    MediaMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                    MediaMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                    MediaMatchCore::None => (),
                }
            }
        }
        b'O' => {
            // OTF ('O') or Avro ('O')
            match probe_font_signatures_core(header) {
                FontMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                FontMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                FontMatchCore::None => (),
            }
            match probe_database_signatures_core(header) {
                DatabaseMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                DatabaseMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                DatabaseMatchCore::None => (),
            }
        }
        b'w' => {
            // WOFF / WOFF2 ('w')
            match probe_font_signatures_core(header) {
                FontMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                FontMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                FontMatchCore::None => (),
            }
        }
        b'S' | 0xD4 | 0xA1 | 0x0A | b'P' => {
            // SQLite3 ('S'), PCAP (0xD4, 0xA1), PCAPNG (0x0A), Parquet ('P')
            match probe_database_signatures_core(header) {
                DatabaseMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                DatabaseMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                DatabaseMatchCore::None => (),
            }
        }
        b'Q' | b'K' | b'#' | b'c' | b'v' => {
            // QCOW ('Q'), VMDK ('K', '#'), VHD ('c'), VHDX ('v')
            match probe_disk_signatures_core(header) {
                DiskMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
                DiskMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
                DiskMatchCore::None => (),
            }
        }
        _ => (),
    }

    // 2. Bounded non-zero offset signature probes
    if header.len() >= 8 && &header[4..8] == b"ftyp" {
        match probe_media_signatures_core(header) {
            MediaMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
            MediaMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
            MediaMatchCore::None => (),
        }
    }
    if header.len() >= 262 && &header[257..262] == b"ustar" {
        match probe_archive_signatures_core(header) {
            ArchiveMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
            ArchiveMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
            ArchiveMatchCore::None => (),
        }
    }
    if header.len() >= 64 && (&header[60..64] == b"BOOK" || &header[60..64] == b"TEXt") {
        match probe_ebook_signatures_core(header) {
            EbookMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
            EbookMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
            EbookMatchCore::None => (),
        }
    }
    if header.len() >= 32774 && (&header[32768..32774] == b"\x01CD001" || &header[32768..32774] == b"\x02CD001") {
        match probe_disk_signatures_core(header) {
            DiskMatchCore::Confirmed(c) => return SignatureMatchCore::Confirmed(c),
            DiskMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
            DiskMatchCore::None => (),
        }
    }

    // 3. Probe all declarative and generated rules via 256-bucket indexed dispatch
    match evaluate_indexed_rules_core(header) {
        RuleMatchCore::Confirmed(core) => return SignatureMatchCore::Confirmed(core),
        RuleMatchCore::Truncated(fmt) => return SignatureMatchCore::Truncated(Cow::Borrowed(fmt)),
        RuleMatchCore::None => (),
    }

    // 11. Probe PDF signature (fast prefix check, followed by bounded window scan)
    if header.starts_with(PDF_SIGNATURE) {
        return SignatureMatchCore::Confirmed(make_pdf_core());
    }
    let scan_limit = header.len().min(1024);
    if scan_limit >= 5 && header[..scan_limit].contains(&b'%') {
        let slice = &header[..scan_limit];
        let mut idx = 0;
        while idx + 5 <= scan_limit {
            if let Some(pos) = slice[idx..].iter().position(|&b| b == b'%') {
                let match_pos = idx + pos;
                if match_pos + 5 <= scan_limit && &slice[match_pos..match_pos + 5] == PDF_SIGNATURE {
                    return SignatureMatchCore::Confirmed(make_pdf_core());
                }
                idx = match_pos + 1;
            } else {
                break;
            }
        }
    }

    if (header.len() >= 3 && header.starts_with(b"%PD"))
        || (!header.is_empty() && header[0] == b'%' && header.len() < 5)
    {
        return SignatureMatchCore::Truncated(Cow::Borrowed("pdf"));
    }

    // 12. Fallback for generic RIFF container when no subtype matched
    if header.len() >= 12 && header.starts_with(b"RIFF") {
        return SignatureMatchCore::Confirmed(make_riff_container_core());
    }

    SignatureMatchCore::None
}

pub fn probe_signatures(header: &[u8]) -> SignatureMatch {
    if header.is_empty() {
        return SignatureMatch::None;
    }

    let b0 = header[0];

    // 1. First-byte O(1) jump table for offset-0 signatures
    match b0 {
        0x89 => {
            // PNG
            match probe_image_signatures(header) {
                ImageMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                ImageMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                ImageMatch::None => (),
            }
        }
        0xFF => {
            // JPEG or MP3 frame sync
            match probe_image_signatures(header) {
                ImageMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                ImageMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                ImageMatch::None => (),
            }
            match probe_media_signatures(header) {
                MediaMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                MediaMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                MediaMatch::None => (),
            }
        }
        b'G' => {
            // GIF
            match probe_image_signatures(header) {
                ImageMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                ImageMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                ImageMatch::None => (),
            }
        }
        b'7' => {
            // 7z
            match probe_archive_signatures(header) {
                ArchiveMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                ArchiveMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                ArchiveMatch::None => (),
            }
        }
        b'R' => {
            // RAR or RIFF (WAV, WebP, AVI)
            match probe_archive_signatures(header) {
                ArchiveMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                ArchiveMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                ArchiveMatch::None => (),
            }
            match probe_media_signatures(header) {
                MediaMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                MediaMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                MediaMatch::None => (),
            }
        }
        0x1F | b'B' | 0xFD | 0x28 => {
            // Gzip (0x1F), Bzip2 ('B'), XZ (0xFD), Zstd (0x28)
            match probe_archive_signatures(header) {
                ArchiveMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                ArchiveMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                ArchiveMatch::None => (),
            }
        }
        0x1A | b'I' => {
            // WebM / MKV (0x1A) or MP3 ID3 ('I')
            match probe_media_signatures(header) {
                MediaMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                MediaMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                MediaMatch::None => (),
            }
        }
        0x7F | b'M' | 0xFE | 0xCE | 0xCF | 0xCA | 0xBE | 0xED => {
            // ELF (0x7F), PE ('M'), Mach-O / Class (0xFE, 0xCE, 0xCF, 0xCA, 0xBE, 0xED)
            match probe_executable_signatures(header) {
                ExecutableMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                ExecutableMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                ExecutableMatch::None => (),
            }
            if b0 == b'M' {
                match probe_database_signatures(header) {
                    DatabaseMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                    DatabaseMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                    DatabaseMatch::None => (),
                }
            }
        }
        0x00 => {
            // WASM (0x00), TTF (0x00), or MP4 box size prefix (\0\0\0)
            if header.starts_with(&[0, 0, 0]) {
                match probe_media_signatures(header) {
                    MediaMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                    MediaMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                    MediaMatch::None => (),
                }
            }
            match probe_executable_signatures(header) {
                ExecutableMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                ExecutableMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                ExecutableMatch::None => (),
            }
            match probe_font_signatures(header) {
                FontMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                FontMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                FontMatch::None => (),
            }
        }
        b'f' => {
            if header.starts_with(b"ftyp") {
                match probe_media_signatures(header) {
                    MediaMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                    MediaMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                    MediaMatch::None => (),
                }
            }
        }
        b'O' => {
            // OTF ('O') or Avro ('O')
            match probe_font_signatures(header) {
                FontMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                FontMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                FontMatch::None => (),
            }
            match probe_database_signatures(header) {
                DatabaseMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                DatabaseMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                DatabaseMatch::None => (),
            }
        }
        b'w' => {
            // WOFF / WOFF2 ('w')
            match probe_font_signatures(header) {
                FontMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                FontMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                FontMatch::None => (),
            }
        }
        b'S' | 0xD4 | 0xA1 | 0x0A | b'P' => {
            // SQLite3 ('S'), PCAP (0xD4, 0xA1), PCAPNG (0x0A), Parquet ('P')
            match probe_database_signatures(header) {
                DatabaseMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                DatabaseMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                DatabaseMatch::None => (),
            }
        }
        b'Q' | b'K' | b'#' | b'c' | b'v' => {
            // QCOW ('Q'), VMDK ('K', '#'), VHD ('c'), VHDX ('v')
            match probe_disk_signatures(header) {
                DiskMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
                DiskMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
                DiskMatch::None => (),
            }
        }
        _ => (),
    }

    // 2. High-offset signatures gated by length and byte checks
    if header.len() >= 262 && (&header[257..262] == b"ustar" || &header[257..262] == b"GNUtar") {
        match probe_archive_signatures(header) {
            ArchiveMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
            ArchiveMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
            ArchiveMatch::None => (),
        }
    }
    if header.len() >= 64 && (&header[60..64] == b"BOOK" || &header[60..64] == b"TEXt") {
        match probe_ebook_signatures(header) {
            EbookMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
            EbookMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
            EbookMatch::None => (),
        }
    }
    if header.len() >= 32774 && (&header[32768..32774] == b"\x01CD001" || &header[32768..32774] == b"\x02CD001") {
        match probe_disk_signatures(header) {
            DiskMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
            DiskMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
            DiskMatch::None => (),
        }
    }

    // 3. Probe all declarative and generated rules via 256-bucket indexed dispatch
    match evaluate_indexed_rules(header) {
        RuleMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
        RuleMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
        RuleMatch::None => (),
    }

    // 11. Probe PDF signature (fast prefix check, followed by bounded window scan)
    if header.starts_with(PDF_SIGNATURE) {
        return SignatureMatch::Confirmed(Box::new(make_pdf_report()));
    }
    let scan_limit = header.len().min(1024);
    if scan_limit >= 5 && header[..scan_limit].contains(&b'%') {
        let slice = &header[..scan_limit];
        let mut idx = 0;
        while idx + 5 <= scan_limit {
            if let Some(pos) = slice[idx..].iter().position(|&b| b == b'%') {
                let match_pos = idx + pos;
                if match_pos + 5 <= scan_limit && &slice[match_pos..match_pos + 5] == PDF_SIGNATURE {
                    return SignatureMatch::Confirmed(Box::new(make_pdf_report()));
                }
                idx = match_pos + 1;
            } else {
                break;
            }
        }
    }

    if (header.len() >= 3 && header.starts_with(b"%PD"))
        || (!header.is_empty() && header[0] == b'%' && header.len() < 5)
    {
        return SignatureMatch::Truncated("pdf".to_string());
    }

    // 12. Fallback for generic RIFF container when no subtype matched
    if header.len() >= 12 && header.starts_with(b"RIFF") {
        return SignatureMatch::Confirmed(Box::new(make_riff_container_report()));
    }

    SignatureMatch::None
}

pub const fn make_riff_container_core() -> DetectionCore {
    DetectionCore {
        format_id: None,
        category: "container",
        container_id: Some("riff"),
        identification: Identification::ContainerOnly,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: None,
        accepted_extensions: &[],
        primary_evidence: Some("sig.riff.header"),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}

pub fn make_riff_container_report() -> DetectionReport {
    make_riff_container_core().to_report(Default::default(), ExtensionStatus::Undetermined)
}

pub const fn make_pdf_core() -> DetectionCore {
    DetectionCore {
        format_id: Some("pdf"),
        category: "document",
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Signature,
        inspection: Inspection::Complete,
        recommended_extension: Some("pdf"),
        accepted_extensions: &["pdf"],
        primary_evidence: Some("sig.pdf.header"),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    }
}

pub fn make_pdf_report() -> DetectionReport {
    make_pdf_core().to_report(Default::default(), ExtensionStatus::Undetermined)
}
