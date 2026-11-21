pub mod ttf_otf;
pub mod woff;

use crate::model::DetectionReport;
use ttf_otf::{probe_ttf_otf, TtfOtfMatch};
use woff::{probe_woff, WoffMatch};

pub enum FontMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
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
