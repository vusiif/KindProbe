pub mod bzip2;
pub mod crc32;
pub mod gzip;
pub mod nested;
pub mod rar;
pub mod sevenz;
pub mod tar;
pub mod xz_zstd;

use bzip2::{probe_bzip2, Bzip2Match};
use gzip::{probe_gzip, GzipMatch};
use nested::probe_nested_gzip;
use rar::{make_rar4_report, make_rar5_report, probe_rar, RarProbeResult};
use sevenz::{make_sevenz_report, probe_sevenz, SevenZProbeResult};
use tar::{probe_tar, TarMatch};
use xz_zstd::{probe_xz_zstd, XzZstdMatch};

use crate::model::DetectionReport;

pub enum ArchiveMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
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
