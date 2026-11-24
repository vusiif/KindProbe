pub mod matroska;
pub mod mp3;
pub mod mp4;
pub mod riff;

use crate::model::{DetectionCore, DetectionReport};
use matroska::{probe_matroska, probe_matroska_core, MatroskaMatch, MatroskaMatchCore};
use mp3::{probe_mp3, probe_mp3_core, Mp3Match, Mp3MatchCore};
use mp4::{probe_mp4, probe_mp4_core, Mp4Match, Mp4MatchCore};
use riff::{probe_riff, probe_riff_core, RiffMatch, RiffMatchCore};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaMatchCore {
    Confirmed(DetectionCore),
    Truncated(&'static str),
    None,
}

pub enum MediaMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes media signatures returning zero-allocation DetectionCore.
pub fn probe_media_signatures_core(header: &[u8]) -> MediaMatchCore {
    // 1. MP4 / MOV (ISO-BMFF)
    match probe_mp4_core(header) {
        Mp4MatchCore::Confirmed(c) => return MediaMatchCore::Confirmed(c),
        Mp4MatchCore::Truncated(fmt) => return MediaMatchCore::Truncated(fmt),
        Mp4MatchCore::None => (),
    }

    // 2. WebM / MKV (EBML)
    match probe_matroska_core(header) {
        MatroskaMatchCore::Confirmed(c) => return MediaMatchCore::Confirmed(c),
        MatroskaMatchCore::Truncated(fmt) => return MediaMatchCore::Truncated(fmt),
        MatroskaMatchCore::None => (),
    }

    // 3. RIFF (WAV, WebP, AVI)
    match probe_riff_core(header) {
        RiffMatchCore::Confirmed(c) => return MediaMatchCore::Confirmed(c),
        RiffMatchCore::Truncated(fmt) => return MediaMatchCore::Truncated(fmt),
        RiffMatchCore::None => (),
    }

    // 4. MP3 (ID3v2 & frame sync)
    match probe_mp3_core(header) {
        Mp3MatchCore::Confirmed(c) => return MediaMatchCore::Confirmed(c),
        Mp3MatchCore::Truncated(fmt) => return MediaMatchCore::Truncated(fmt),
        Mp3MatchCore::None => (),
    }

    MediaMatchCore::None
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
