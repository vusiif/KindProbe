pub mod mobi;

use crate::model::{DetectionCore, DetectionReport};
use mobi::{probe_mobi, probe_mobi_core, MobiMatch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EbookMatchCore {
    Confirmed(DetectionCore),
    Truncated(&'static str),
    None,
}

pub enum EbookMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes standalone e-book format signatures returning zero-allocation DetectionCore.
pub fn probe_ebook_signatures_core(header: &[u8]) -> EbookMatchCore {
    if header.len() < 68 && header.len() >= 64 && &header[60..64] == b"BOOK" {
        return EbookMatchCore::Truncated("mobi");
    }
    if let Some(core) = probe_mobi_core(header) {
        return EbookMatchCore::Confirmed(core);
    }
    EbookMatchCore::None
}

/// Probes standalone e-book format signatures (MOBI, PalmDOC).
pub fn probe_ebook_signatures(header: &[u8]) -> EbookMatch {
    match probe_mobi(header) {
        MobiMatch::Confirmed(rep) => EbookMatch::Confirmed(rep),
        MobiMatch::Truncated(fmt) => EbookMatch::Truncated(fmt),
        MobiMatch::None => EbookMatch::None,
    }
}
