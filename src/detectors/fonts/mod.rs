pub mod ttf_otf;
pub mod woff;

use crate::model::{DetectionCore, DetectionReport};
use ttf_otf::{probe_ttf_otf, probe_ttf_otf_core, TtfOtfMatch};
use woff::{probe_woff, probe_woff_core, WoffMatch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontMatchCore {
    Confirmed(DetectionCore),
    Truncated(&'static str),
    None,
}

pub enum FontMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes font signatures returning zero-allocation DetectionCore.
pub fn probe_font_signatures_core(header: &[u8]) -> FontMatchCore {
    // 1. TTF / OTF
    if header.len() < 4 && !header.is_empty() && (header.starts_with(&[0x00, 0x01]) || header.starts_with(b"OT")) {
        return FontMatchCore::Truncated("ttf");
    }
    if let Some(core) = probe_ttf_otf_core(header) {
        return FontMatchCore::Confirmed(core);
    }

    // 2. WOFF / WOFF2
    if header.len() < 4 && !header.is_empty() && header.starts_with(b"wO") {
        return FontMatchCore::Truncated("woff");
    }
    if let Some(core) = probe_woff_core(header) {
        return FontMatchCore::Confirmed(core);
    }

    FontMatchCore::None
}

/// Probes font signatures (TTF, OTF, WOFF, WOFF2).
pub fn probe_font_signatures(header: &[u8]) -> FontMatch {
    // 1. TTF / OTF
    match probe_ttf_otf(header) {
        TtfOtfMatch::Confirmed(rep) => return FontMatch::Confirmed(rep),
        TtfOtfMatch::Truncated(fmt) => return FontMatch::Truncated(fmt),
        TtfOtfMatch::None => (),
    }

    // 2. WOFF / WOFF2
    match probe_woff(header) {
        WoffMatch::Confirmed(rep) => return FontMatch::Confirmed(rep),
        WoffMatch::Truncated(fmt) => return FontMatch::Truncated(fmt),
        WoffMatch::None => (),
    }

    FontMatch::None
}
