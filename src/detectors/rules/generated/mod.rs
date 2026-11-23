pub mod archives;
pub mod audio_chips;
pub mod bytecode;
pub mod cad_and_gnss;
pub mod crypto;
pub mod database;
pub mod disk_images;
pub mod documents;
pub mod firmware_and_forensics;
pub mod fonts;
pub mod game_assets;
pub mod images;
pub mod media;
pub mod models;
pub mod multimedia_stream;
pub mod netpbm;
pub mod packages;
pub mod playlists;
pub mod pro_audio_and_science;
pub mod robotics_firmware;
pub mod roms;
pub mod scientific;
pub mod subtitles;
pub mod telecom_exchange;
pub mod textures;
pub mod vector_graphics;

use crate::detectors::rules::types::SignatureRule;

/// Combined view of all auto-generated declarative signature rules across 26 partitions.
pub static GENERATED_RULES: &[&[SignatureRule]] = &[
    archives::ARCHIVES_RULES,
    audio_chips::AUDIO_CHIPS_RULES,
    bytecode::BYTECODE_RULES,
    cad_and_gnss::CAD_AND_GNSS_RULES,
    crypto::CRYPTO_RULES,
    database::DATABASE_RULES,
    disk_images::DISK_IMAGES_RULES,
    documents::DOCUMENTS_RULES,
    firmware_and_forensics::FIRMWARE_AND_FORENSICS_RULES,
    fonts::FONTS_RULES,
    game_assets::GAME_ASSETS_RULES,
    images::IMAGES_RULES,
    media::MEDIA_RULES,
    models::MODELS_RULES,
    multimedia_stream::MULTIMEDIA_STREAM_RULES,
    netpbm::NETPBM_RULES,
    packages::PACKAGES_RULES,
    playlists::PLAYLISTS_RULES,
    pro_audio_and_science::PRO_AUDIO_AND_SCIENCE_RULES,
    robotics_firmware::ROBOTICS_FIRMWARE_RULES,
    roms::ROMS_RULES,
    scientific::SCIENTIFIC_RULES,
    subtitles::SUBTITLES_RULES,
    telecom_exchange::TELECOM_EXCHANGE_RULES,
    textures::TEXTURES_RULES,
    vector_graphics::VECTOR_GRAPHICS_RULES,
];
