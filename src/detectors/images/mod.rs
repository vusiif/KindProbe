pub mod gif;
pub mod jpeg;
pub mod png;

use crate::model::{DetectionCore, DetectionReport, ExtensionStatus};
pub use gif::{make_gif_core, make_gif_report, probe_gif, GifProbeResult};
pub use jpeg::{make_jpeg_core, make_jpeg_report, probe_jpeg, JpegProbeResult};
pub use png::{make_png_core, make_png_report, probe_png, PngProbeResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageMatchCore {
    Confirmed(DetectionCore),
    Truncated(&'static str),
    None,
}

pub enum ImageMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

pub fn probe_image_signatures_core(header: &[u8]) -> ImageMatchCore {
    // 1. PNG
    match probe_png(header) {
        PngProbeResult::Confirmed { has_ihdr } => {
            return ImageMatchCore::Confirmed(make_png_core(has_ihdr))
        }
        PngProbeResult::Truncated => return ImageMatchCore::Truncated("png"),
        PngProbeResult::None => (),
    }

    // 2. JPEG
    match probe_jpeg(header) {
        JpegProbeResult::Confirmed { has_segment } => {
            return ImageMatchCore::Confirmed(make_jpeg_core(has_segment))
        }
        JpegProbeResult::Truncated => return ImageMatchCore::Truncated("jpeg"),
        JpegProbeResult::None => (),
    }

    // 3. GIF
    match probe_gif(header) {
        GifProbeResult::Confirmed { has_screen_desc } => {
            return ImageMatchCore::Confirmed(make_gif_core(has_screen_desc))
        }
        GifProbeResult::Truncated => return ImageMatchCore::Truncated("gif"),
        GifProbeResult::None => (),
    }

    ImageMatchCore::None
}

pub fn probe_image_signatures(header: &[u8]) -> ImageMatch {
    match probe_image_signatures_core(header) {
        ImageMatchCore::Confirmed(core) => ImageMatch::Confirmed(Box::new(
            core.to_report(Default::default(), ExtensionStatus::Undetermined),
        )),
        ImageMatchCore::Truncated(fmt) => ImageMatch::Truncated(fmt.to_string()),
        ImageMatchCore::None => ImageMatch::None,
    }
}
