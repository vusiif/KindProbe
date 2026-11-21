pub mod matroska;
pub mod mp3;
pub mod mp4;
pub mod riff;

use crate::model::DetectionReport;
use matroska::{probe_matroska, MatroskaMatch};
use mp3::{probe_mp3, Mp3Match};
use mp4::{probe_mp4, Mp4Match};
use riff::{probe_riff, RiffMatch};

pub enum MediaMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes media container and codec signatures (MP4, MOV, WebM, MKV, WAV, WebP, AVI, MP3).
pub fn probe_media_signatures(header: &[u8]) -> MediaMatch {
    // 1. MP4 / MOV (ISO-BMFF)
    match probe_mp4(header) {
        Mp4Match::Confirmed(rep) => return MediaMatch::Confirmed(rep),
        Mp4Match::Truncated(fmt) => return MediaMatch::Truncated(fmt),
        Mp4Match::None => (),
    }

    // 2. WebM / MKV (EBML)
    match probe_matroska(header) {
        MatroskaMatch::Confirmed(rep) => return MediaMatch::Confirmed(rep),
        MatroskaMatch::Truncated(fmt) => return MediaMatch::Truncated(fmt),
        MatroskaMatch::None => (),
    }

    // 3. RIFF (WAV, WebP, AVI)
    match probe_riff(header) {
        RiffMatch::Confirmed(rep) => return MediaMatch::Confirmed(rep),
        RiffMatch::Truncated(fmt) => return MediaMatch::Truncated(fmt),
        RiffMatch::None => (),
    }

    // 4. MP3 (ID3v2 & frame sync)
    match probe_mp3(header) {
        Mp3Match::Confirmed(rep) => return MediaMatch::Confirmed(rep),
        Mp3Match::Truncated(fmt) => return MediaMatch::Truncated(fmt),
        Mp3Match::None => (),
    }

    MediaMatch::None
}
