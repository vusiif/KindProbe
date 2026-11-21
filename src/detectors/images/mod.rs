pub mod gif;
pub mod jpeg;
pub mod png;

use crate::model::DetectionReport;
use gif::{make_gif_report, probe_gif, GifProbeResult};
use jpeg::{make_jpeg_report, probe_jpeg, JpegProbeResult};
use png::{make_png_report, probe_png, PngProbeResult};

pub enum ImageMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn probe_image_signatures(header: &[u8]) -> ImageMatch {
    // 1. PNG
    match probe_png(header) {
        PngProbeResult::Confirmed { has_ihdr } => {
            return ImageMatch::Confirmed(Box::new(make_png_report(has_ihdr)))
        }
        PngProbeResult::Truncated => return ImageMatch::Truncated("png".to_string()),
        PngProbeResult::None => (),
    }

    // 2. JPEG
    match probe_jpeg(header) {
        JpegProbeResult::Confirmed { has_segment } => {
            return ImageMatch::Confirmed(Box::new(make_jpeg_report(has_segment)))
        }
        JpegProbeResult::Truncated => return ImageMatch::Truncated("jpeg".to_string()),
        JpegProbeResult::None => (),
    }

    // 3. GIF
    match probe_gif(header) {
        GifProbeResult::Confirmed { has_screen_desc } => {
            return ImageMatch::Confirmed(Box::new(make_gif_report(has_screen_desc)))
        }
        GifProbeResult::Truncated => return ImageMatch::Truncated("gif".to_string()),
        GifProbeResult::None => (),
    }

    ImageMatch::None
}
