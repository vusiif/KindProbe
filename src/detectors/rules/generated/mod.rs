pub mod archives;
pub mod astronomy_and_space;
pub mod audio_chips;
pub mod automotive_and_eda;
pub mod bio_and_creative_design;
pub mod bytecode;
pub mod cad_and_gnss;
pub mod crypto;
pub mod database;
pub mod disk_images;
pub mod documents;
pub mod embedded_rtos_images;
pub mod firmware_and_forensics;
pub mod fonts;
pub mod fonts_and_typography;
pub mod game_assets;
pub mod geospatial_vector_cad;
pub mod images;
pub mod media;
pub mod models;
pub mod multimedia_stream;
pub mod netpbm;
pub mod packages;
pub mod playlists;
pub mod pro_audio_and_science;
pub mod retro_and_virtualization;
pub mod robotics_firmware;
pub mod roms;
pub mod scientific;
pub mod subtitles;
pub mod telecom_exchange;
pub mod textures;
pub mod vector_graphics;
pub mod video_streams_and_rf;

use crate::detectors::rules::types::SignatureRule;

/// Combined view of all auto-generated declarative signature rules across 34 partitions.
pub static GENERATED_RULES: &[&[SignatureRule]] = &[
    archives::ARCHIVES_RULES,
    astronomy_and_space::ASTRONOMY_AND_SPACE_RULES,
    audio_chips::AUDIO_CHIPS_RULES,
    automotive_and_eda::AUTOMOTIVE_AND_EDA_RULES,
    bio_and_creative_design::BIO_AND_CREATIVE_DESIGN_RULES,
    bytecode::BYTECODE_RULES,
    cad_and_gnss::CAD_AND_GNSS_RULES,
    crypto::CRYPTO_RULES,
    database::DATABASE_RULES,
    disk_images::DISK_IMAGES_RULES,
    documents::DOCUMENTS_RULES,
    embedded_rtos_images::EMBEDDED_RTOS_IMAGES_RULES,
    firmware_and_forensics::FIRMWARE_AND_FORENSICS_RULES,
    fonts::FONTS_RULES,
    fonts_and_typography::FONTS_AND_TYPOGRAPHY_RULES,
    game_assets::GAME_ASSETS_RULES,
    geospatial_vector_cad::GEOSPATIAL_VECTOR_CAD_RULES,
    images::IMAGES_RULES,
    media::MEDIA_RULES,
    models::MODELS_RULES,
    multimedia_stream::MULTIMEDIA_STREAM_RULES,
    netpbm::NETPBM_RULES,
    packages::PACKAGES_RULES,
    playlists::PLAYLISTS_RULES,
    pro_audio_and_science::PRO_AUDIO_AND_SCIENCE_RULES,
    retro_and_virtualization::RETRO_AND_VIRTUALIZATION_RULES,
    robotics_firmware::ROBOTICS_FIRMWARE_RULES,
    roms::ROMS_RULES,
    scientific::SCIENTIFIC_RULES,
    subtitles::SUBTITLES_RULES,
    telecom_exchange::TELECOM_EXCHANGE_RULES,
    textures::TEXTURES_RULES,
    vector_graphics::VECTOR_GRAPHICS_RULES,
    video_streams_and_rf::VIDEO_STREAMS_AND_RF_RULES,
];
