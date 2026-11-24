use crate::model::{
    DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection, Verification,
};
use super::archives::{probe_archive_signatures, ArchiveMatch};
use super::databases::{probe_database_signatures, DatabaseMatch};
use super::disks::{probe_disk_signatures, DiskMatch};
use super::ebooks::{probe_ebook_signatures, EbookMatch};
use super::executables::{probe_executable_signatures, ExecutableMatch};
use super::fonts::{probe_font_signatures, FontMatch};
use super::images::{probe_image_signatures, ImageMatch};
use super::media::{probe_media_signatures, MediaMatch};
use super::rules::{evaluate_indexed_rules, RuleMatch};

pub const PDF_SIGNATURE: &[u8] = b"%PDF-";

pub enum SignatureMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
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
                // 0x4D is also the first byte of modified PCAP (0x4D, 0x3C, 0xB2, 0xA1)
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

    // 2. Bounded non-zero offset signature probes
    if header.len() >= 8 && &header[4..8] == b"ftyp" {
        match probe_media_signatures(header) {
            MediaMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
            MediaMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
            MediaMatch::None => (),
        }
    }
    if header.len() >= 262 && &header[257..262] == b"ustar" {
        match probe_archive_signatures(header) {
            ArchiveMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
            ArchiveMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
            ArchiveMatch::None => (),
        }
    }
    if header.len() >= 64 {
        match probe_ebook_signatures(header) {
            EbookMatch::Confirmed(rep) => return SignatureMatch::Confirmed(rep),
            EbookMatch::Truncated(fmt) => return SignatureMatch::Truncated(fmt),
            EbookMatch::None => (),
        }
    }
    if header.len() >= 32773 {
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
    if scan_limit >= 5
        && header[..scan_limit].contains(&b'%')
        && header[..scan_limit].windows(5).any(|w| w == PDF_SIGNATURE)
    {
        return SignatureMatch::Confirmed(Box::new(make_pdf_report()));
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

fn make_riff_container_report() -> DetectionReport {
    DetectionReport {
        schema_version: 1,
        format_id: None,
        category: "container".to_string(),
        container_id: Some("riff".to_string()),
        identification: Identification::ContainerOnly,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: None,
        accepted_extensions: Vec::new(),
        extension_status: ExtensionStatus::Undetermined,
        evidence: vec![EvidenceItem::new("sig.riff.header")],
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }
}

pub fn make_pdf_report() -> DetectionReport {
    DetectionReport {
        schema_version: 1,
        format_id: Some("pdf".to_string()),
        category: "document".to_string(),
        container_id: None,
        identification: Identification::Format,
        verification: Verification::Signature,
        inspection: Inspection::Complete,
        recommended_extension: Some("pdf".to_string()),
        accepted_extensions: vec!["pdf".to_string()],
        extension_status: ExtensionStatus::Undetermined,
        evidence: vec![EvidenceItem::new("sig.pdf.header")],
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    }
}
