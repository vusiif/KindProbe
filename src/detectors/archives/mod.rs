pub mod bzip2;
pub mod crc32;
pub mod gzip;
pub mod nested;
pub mod rar;
pub mod sevenz;
pub mod tar;
pub mod xz_zstd;

use bzip2::{probe_bzip2, probe_bzip2_core, Bzip2Match};
use gzip::{probe_gzip, probe_gzip_core, GzipMatch};
use nested::{probe_nested_gzip, probe_nested_gzip_core};
use rar::{make_rar4_core, make_rar4_report, make_rar5_core, make_rar5_report, probe_rar, RarProbeResult};
use sevenz::{make_sevenz_core, make_sevenz_report, probe_sevenz, SevenZProbeResult};
use tar::{probe_tar, probe_tar_core, TarMatch};
use xz_zstd::{probe_xz_zstd, probe_xz_zstd_core, XzZstdMatch};

use crate::model::{DetectionCore, DetectionReport};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveMatchCore {
    Confirmed(DetectionCore),
    Truncated(&'static str),
    None,
}

pub enum ArchiveMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes archive signatures returning zero-allocation DetectionCore.
pub fn probe_archive_signatures_core(header: &[u8]) -> ArchiveMatchCore {
    // 1. 7z
    match probe_sevenz(header) {
        SevenZProbeResult::Confirmed { has_start_header } => {
            return ArchiveMatchCore::Confirmed(make_sevenz_core(has_start_header));
        }
        SevenZProbeResult::Truncated => return ArchiveMatchCore::Truncated("7z"),
        SevenZProbeResult::None => (),
    }

    // 2. RAR (RAR4 / RAR5)
    match probe_rar(header) {
        RarProbeResult::ConfirmedRar5 => {
            return ArchiveMatchCore::Confirmed(make_rar5_core());
        }
        RarProbeResult::ConfirmedRar4 { has_main_header } => {
            return ArchiveMatchCore::Confirmed(make_rar4_core(has_main_header));
        }
        RarProbeResult::Truncated => return ArchiveMatchCore::Truncated("rar"),
        RarProbeResult::None => (),
    }

    // 3. Nested tar.gz or standalone Gzip
    if let Some(nested_core) = probe_nested_gzip_core(header) {
        return ArchiveMatchCore::Confirmed(nested_core);
    }
    if header.len() < 2 && !header.is_empty() && header[0] == 0x1F {
        return ArchiveMatchCore::Truncated("gzip");
    }
    if let Some(gzip_core) = probe_gzip_core(header) {
        return ArchiveMatchCore::Confirmed(gzip_core);
    }

    // 4. Bzip2
    if header.len() < 3 && !header.is_empty() && header.starts_with(b"BZ") {
        return ArchiveMatchCore::Truncated("bzip2");
    }
    if let Some(bzip2_core) = probe_bzip2_core(header) {
        return ArchiveMatchCore::Confirmed(bzip2_core);
    }

    // 5. XZ / Zstandard
    if let Some(xz_zstd_core) = probe_xz_zstd_core(header) {
        return ArchiveMatchCore::Confirmed(xz_zstd_core);
    }
    if !header.is_empty() {
        if header.len() < 6 && xz_zstd::XZ_MAGIC.starts_with(header) {
            return ArchiveMatchCore::Truncated("xz");
        } else if header.len() < 4 && xz_zstd::ZSTD_MAGIC.starts_with(header) {
            return ArchiveMatchCore::Truncated("zstd");
        }
    }

    // 6. Tar (POSIX ustar)
    if header.len() < 512 {
        if header.len() >= 265
            && (&header[257..262] == b"ustar" || &header[257..264] == b"ustar\0\0")
        {
            return ArchiveMatchCore::Truncated("tar");
        }
    } else if let Some(tar_core) = probe_tar_core(header) {
        return ArchiveMatchCore::Confirmed(tar_core);
    }

    ArchiveMatchCore::None
}

pub fn probe_archive_signatures(header: &[u8]) -> ArchiveMatch {
    // 1. 7z
    match probe_sevenz(header) {
        SevenZProbeResult::Confirmed { has_start_header } => {
            return ArchiveMatch::Confirmed(Box::new(make_sevenz_report(has_start_header)))
        }
        SevenZProbeResult::Truncated => return ArchiveMatch::Truncated("7z".to_string()),
        SevenZProbeResult::None => (),
    }

    // 2. RAR (RAR4 / RAR5)
    match probe_rar(header) {
        RarProbeResult::ConfirmedRar5 => {
            return ArchiveMatch::Confirmed(Box::new(make_rar5_report()))
        }
        RarProbeResult::ConfirmedRar4 { has_main_header } => {
            return ArchiveMatch::Confirmed(Box::new(make_rar4_report(has_main_header)))
        }
        RarProbeResult::Truncated => return ArchiveMatch::Truncated("rar".to_string()),
        RarProbeResult::None => (),
    }

    // 3. Nested tar.gz or standalone Gzip
    if let Some(nested_rep) = probe_nested_gzip(header) {
        return ArchiveMatch::Confirmed(Box::new(nested_rep));
    }
    match probe_gzip(header) {
        GzipMatch::Confirmed(rep) => return ArchiveMatch::Confirmed(rep),
        GzipMatch::Truncated(fmt) => return ArchiveMatch::Truncated(fmt),
        GzipMatch::None => (),
    }

    // 4. Bzip2
    match probe_bzip2(header) {
        Bzip2Match::Confirmed(rep) => return ArchiveMatch::Confirmed(rep),
        Bzip2Match::Truncated(fmt) => return ArchiveMatch::Truncated(fmt),
        Bzip2Match::None => (),
    }

    // 5. XZ / Zstandard
    match probe_xz_zstd(header) {
        XzZstdMatch::Confirmed(rep) => return ArchiveMatch::Confirmed(rep),
        XzZstdMatch::Truncated(fmt) => return ArchiveMatch::Truncated(fmt),
        XzZstdMatch::None => (),
    }

    // 6. Tar (POSIX ustar)
    match probe_tar(header) {
        TarMatch::Confirmed(rep) => return ArchiveMatch::Confirmed(rep),
        TarMatch::Truncated(fmt) => return ArchiveMatch::Truncated(fmt),
        TarMatch::None => (),
    }

    ArchiveMatch::None
}
