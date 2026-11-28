use super::types::{MagicPattern, SignatureRule, ValidationStatus};
use crate::model::Verification;

fn validate_flac(header: &[u8]) -> ValidationStatus {
    if header.len() < 8 {
        return ValidationStatus::Truncated;
    }
    if (header[4] & 0x7F) == 0 {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}

fn validate_ogg(header: &[u8]) -> ValidationStatus {
    if header.len() < 6 {
        return ValidationStatus::Truncated;
    }
    if header[4] == 0 && header[5] <= 7 {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}

fn validate_tiff_le(header: &[u8]) -> ValidationStatus {
    if header.len() < 8 {
        return ValidationStatus::Truncated;
    }
    let ifd = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
    if (8..=0x7FFF_FFFF).contains(&ifd) {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}

fn validate_tiff_be(header: &[u8]) -> ValidationStatus {
    if header.len() < 8 {
        return ValidationStatus::Truncated;
    }
    let ifd = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);
    if (8..=0x7FFF_FFFF).contains(&ifd) {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}

fn validate_ico_or_cur(header: &[u8]) -> ValidationStatus {
    if header.len() < 6 {
        return ValidationStatus::Truncated;
    }
    let count = u16::from_le_bytes([header[4], header[5]]);
    if (1..=256).contains(&count) {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}

fn validate_psd(header: &[u8]) -> ValidationStatus {
    if header.len() < 12 {
        return ValidationStatus::Truncated;
    }
    let version = u16::from_be_bytes([header[4], header[5]]);
    let reserved_zero = header[6..12] == [0, 0, 0, 0, 0, 0];
    if version == 1 && reserved_zero {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}

fn validate_rpm(header: &[u8]) -> ValidationStatus {
    if header.len() < 6 {
        return ValidationStatus::Truncated;
    }
    if header[4] == 3 || header[4] == 4 {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}

fn validate_crx(header: &[u8]) -> ValidationStatus {
    if header.len() < 8 {
        return ValidationStatus::Truncated;
    }
    let ver = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
    if ver == 2 || ver == 3 {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}

fn validate_mid(header: &[u8]) -> ValidationStatus {
    if header.len() < 8 {
        return ValidationStatus::Truncated;
    }
    let length = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);
    if length == 6 {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}

fn validate_chm(header: &[u8]) -> ValidationStatus {
    if header.len() < 8 {
        return ValidationStatus::Truncated;
    }
    let ver = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
    if ver == 3 || ver == 4 {
        ValidationStatus::Valid
    } else {
        ValidationStatus::Invalid
    }
}

pub static DECLARATIVE_RULES: &[SignatureRule] = &[
    // 1. Debian Package (Multi-pattern magic)
    SignatureRule {
        format_id: "deb",
        category: "package",
        container_id: Some("ar"),
        recommended_ext: "deb",
        accepted_exts: &["deb", "udeb"],
        patterns: &[
            MagicPattern::new(0, b"!<arch>\n"),
            MagicPattern::new(8, b"debian-binary"),
        ],
        min_header_len: 21,
        priority: 95,
        evidence_code: "sig.deb.magic",
        verification: Verification::Structure,
        validator: None,
    },
    // 2. Red Hat Package (RPM)
    SignatureRule {
        format_id: "rpm",
        category: "package",
        container_id: None,
        recommended_ext: "rpm",
        accepted_exts: &["rpm"],
        patterns: &[MagicPattern::new(0, &[0xED, 0xAB, 0xEE, 0xDB])],
        min_header_len: 6,
        priority: 95,
        evidence_code: "sig.rpm.magic",
        verification: Verification::Structure,
        validator: Some(validate_rpm),
    },
    // 3. Chrome Extension (CRX)
    SignatureRule {
        format_id: "crx",
        category: "package",
        container_id: None,
        recommended_ext: "crx",
        accepted_exts: &["crx"],
        patterns: &[MagicPattern::new(0, b"Cr24")],
        min_header_len: 8,
        priority: 95,
        evidence_code: "sig.crx.magic",
        verification: Verification::Structure,
        validator: Some(validate_crx),
    },
    // 4. FLAC Audio
    SignatureRule {
        format_id: "flac",
        category: "audio",
        container_id: None,
        recommended_ext: "flac",
        accepted_exts: &["flac"],
        patterns: &[MagicPattern::new(0, b"fLaC")],
        min_header_len: 8,
        priority: 90,
        evidence_code: "sig.flac.magic",
        verification: Verification::Structure,
        validator: Some(validate_flac),
    },
    // 5. Ogg Media Container
    SignatureRule {
        format_id: "ogg",
        category: "audio",
        container_id: Some("ogg"),
        recommended_ext: "ogg",
        accepted_exts: &["ogg", "oga", "ogv", "ogx", "opus"],
        patterns: &[MagicPattern::new(0, b"OggS")],
        min_header_len: 6,
        priority: 90,
        evidence_code: "sig.ogg.magic",
        verification: Verification::Structure,
        validator: Some(validate_ogg),
    },
    // 6. Photoshop Document (PSD)
    SignatureRule {
        format_id: "psd",
        category: "image",
        container_id: None,
        recommended_ext: "psd",
        accepted_exts: &["psd"],
        patterns: &[MagicPattern::new(0, b"8BPS")],
        min_header_len: 12,
        priority: 90,
        evidence_code: "sig.psd.magic",
        verification: Verification::Structure,
        validator: Some(validate_psd),
    },
    // 7. TIFF Little-Endian
    SignatureRule {
        format_id: "tiff",
        category: "image",
        container_id: None,
        recommended_ext: "tiff",
        accepted_exts: &["tiff", "tif"],
        patterns: &[MagicPattern::new(0, b"II*\0")],
        min_header_len: 8,
        priority: 85,
        evidence_code: "sig.tiff.magic",
        verification: Verification::Structure,
        validator: Some(validate_tiff_le),
    },
    // 8. TIFF Big-Endian
    SignatureRule {
        format_id: "tiff",
        category: "image",
        container_id: None,
        recommended_ext: "tiff",
        accepted_exts: &["tiff", "tif"],
        patterns: &[MagicPattern::new(0, b"MM\0*")],
        min_header_len: 8,
        priority: 85,
        evidence_code: "sig.tiff.magic",
        verification: Verification::Structure,
        validator: Some(validate_tiff_be),
    },
    // 9. Windows Icon (ICO)
    SignatureRule {
        format_id: "ico",
        category: "image",
        container_id: None,
        recommended_ext: "ico",
        accepted_exts: &["ico"],
        patterns: &[MagicPattern::new(0, &[0x00, 0x00, 0x01, 0x00])],
        min_header_len: 6,
        priority: 80,
        evidence_code: "sig.ico.magic",
        verification: Verification::Structure,
        validator: Some(validate_ico_or_cur),
    },
    // 10. Windows Cursor (CUR)
    SignatureRule {
        format_id: "cur",
        category: "image",
        container_id: None,
        recommended_ext: "cur",
        accepted_exts: &["cur"],
        patterns: &[MagicPattern::new(0, &[0x00, 0x00, 0x02, 0x00])],
        min_header_len: 6,
        priority: 80,
        evidence_code: "sig.cur.magic",
        verification: Verification::Structure,
        validator: Some(validate_ico_or_cur),
    },
    // 11. Advanced Audio Coding (AAC - ADTS syncword)
    SignatureRule {
        format_id: "aac",
        category: "audio",
        container_id: None,
        recommended_ext: "aac",
        accepted_exts: &["aac"],
        patterns: &[MagicPattern::new(0, &[0xFF, 0xF1])],
        min_header_len: 4,
        priority: 80,
        evidence_code: "sig.aac.magic",
        verification: Verification::Structure,
        validator: None,
    },
    SignatureRule {
        format_id: "aac",
        category: "audio",
        container_id: None,
        recommended_ext: "aac",
        accepted_exts: &["aac"],
        patterns: &[MagicPattern::new(0, &[0xFF, 0xF9])],
        min_header_len: 4,
        priority: 80,
        evidence_code: "sig.aac.magic",
        verification: Verification::Structure,
        validator: None,
    },
    // 12. Audio Interchange File Format (AIFF)
    SignatureRule {
        format_id: "aiff",
        category: "audio",
        container_id: None,
        recommended_ext: "aif",
        accepted_exts: &["aif", "aiff", "aifc"],
        patterns: &[MagicPattern::new(0, b"FORM"), MagicPattern::new(8, b"AIFF")],
        min_header_len: 12,
        priority: 85,
        evidence_code: "sig.aiff.magic",
        verification: Verification::Structure,
        validator: None,
    },
    SignatureRule {
        format_id: "aiff",
        category: "audio",
        container_id: None,
        recommended_ext: "aif",
        accepted_exts: &["aif", "aiff", "aifc"],
        patterns: &[MagicPattern::new(0, b"FORM"), MagicPattern::new(8, b"AIFC")],
        min_header_len: 12,
        priority: 85,
        evidence_code: "sig.aiff.magic",
        verification: Verification::Structure,
        validator: None,
    },
    // 13. MIDI Audio (MID)
    SignatureRule {
        format_id: "mid",
        category: "audio",
        container_id: None,
        recommended_ext: "mid",
        accepted_exts: &["mid", "midi"],
        patterns: &[MagicPattern::new(0, b"MThd")],
        min_header_len: 8,
        priority: 90,
        evidence_code: "sig.mid.magic",
        verification: Verification::Structure,
        validator: Some(validate_mid),
    },
    // 14. Compiled HTML Help (CHM)
    SignatureRule {
        format_id: "chm",
        category: "document",
        container_id: None,
        recommended_ext: "chm",
        accepted_exts: &["chm"],
        patterns: &[MagicPattern::new(0, b"ITSF")],
        min_header_len: 8,
        priority: 90,
        evidence_code: "sig.chm.magic",
        verification: Verification::Structure,
        validator: Some(validate_chm),
    },
];
