pub mod rar;
pub mod sevenz;

use crate::model::DetectionReport;
use rar::{make_rar4_report, make_rar5_report, probe_rar, RarProbeResult};
use sevenz::{make_sevenz_report, probe_sevenz, SevenZProbeResult};

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

    ArchiveMatch::None
}
