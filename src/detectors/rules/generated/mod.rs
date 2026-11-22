pub mod archives;
pub mod audio_chips;
pub mod bytecode;
pub mod crypto;
pub mod database;
pub mod disk_images;
pub mod documents;
pub mod fonts;
pub mod images;
pub mod media;
pub mod models;
pub mod multimedia_stream;
pub mod netpbm;
pub mod packages;
pub mod roms;
pub mod scientific;
pub mod textures;
pub mod vector_graphics;

pub use archives::ARCHIVES_RULES;
pub use audio_chips::AUDIO_CHIPS_RULES;
pub use bytecode::BYTECODE_RULES;
pub use crypto::CRYPTO_RULES;
pub use database::DATABASE_RULES;
pub use disk_images::DISK_IMAGES_RULES;
pub use documents::DOCUMENTS_RULES;
pub use fonts::FONTS_RULES;
pub use images::IMAGES_RULES;
pub use media::MEDIA_RULES;
pub use models::MODELS_RULES;
pub use multimedia_stream::MULTIMEDIA_STREAM_RULES;
pub use netpbm::NETPBM_RULES;
pub use packages::PACKAGES_RULES;
pub use roms::ROMS_RULES;
pub use scientific::SCIENTIFIC_RULES;
pub use textures::TEXTURES_RULES;
pub use vector_graphics::VECTOR_GRAPHICS_RULES;

use crate::detectors::rules::types::SignatureRule;

/// Combined view of all auto-generated declarative signature rules across 18 partitions.
pub static GENERATED_RULES: &[&[SignatureRule]] = &[
    ARCHIVES_RULES,
    AUDIO_CHIPS_RULES,
    BYTECODE_RULES,
    CRYPTO_RULES,
    DATABASE_RULES,
    DISK_IMAGES_RULES,
    DOCUMENTS_RULES,
    FONTS_RULES,
    IMAGES_RULES,
    MEDIA_RULES,
    MODELS_RULES,
    MULTIMEDIA_STREAM_RULES,
    NETPBM_RULES,
    PACKAGES_RULES,
    ROMS_RULES,
    SCIENTIFIC_RULES,
    TEXTURES_RULES,
    VECTOR_GRAPHICS_RULES,
];
