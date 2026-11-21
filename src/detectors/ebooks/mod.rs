pub mod mobi;

use crate::model::DetectionReport;
use mobi::{probe_mobi, MobiMatch};

pub enum EbookMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes standalone e-book format signatures (MOBI, PalmDOC).
pub fn probe_ebook_signatures(header: &[u8]) -> EbookMatch {
    match probe_mobi(header) {
        MobiMatch::Confirmed(rep) => EbookMatch::Confirmed(rep),
        MobiMatch::Truncated(fmt) => EbookMatch::Truncated(fmt),
        MobiMatch::None => EbookMatch::None,
    }
}
