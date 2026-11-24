/// Gateway for validating text buffers, stripping UTF-8 BOM, and filtering binary noise.
pub fn examine_text_gateway(bytes: &[u8]) -> Option<&str> {
    if bytes.is_empty() {
        return None;
    }

    // 1. Strip UTF-8 BOM if present
    let raw = if bytes.starts_with(b"\xEF\xBB\xBF") {
        &bytes[3..]
    } else {
        bytes
    };

    if raw.is_empty() {
        return None;
    }

    // Fast bailout: First byte must be valid text starter
    // Valid starters: whitespace (0x09, 0x0A, 0x0D, 0x20), printable ASCII (0x21..=0x7E), or UTF-8 lead (0xC2..=0xF4)
    let b0 = raw[0];
    if (b0 < 0x20 && b0 != b'\t' && b0 != b'\n' && b0 != b'\r') || (b0 >= 0x7F && (b0 <= 0xC1 || b0 >= 0xF5)) {
        return None;
    }

    // 2. Reject immediately if null bytes exist
    if raw.contains(&0) {
        return None;
    }

    // 3. UTF-8 conversion
    let s = match std::str::from_utf8(raw) {
        Ok(valid) => valid,
        Err(e) => {
            // Allow truncated last UTF-8 character at window boundary
            let valid_len = e.valid_up_to();
            if valid_len > 0 && e.error_len().is_none() {
                std::str::from_utf8(&raw[..valid_len]).unwrap_or("")
            } else {
                return None;
            }
        }
    };

    if s.is_empty() {
        return None;
    }

    // 4. Control character ratio check (reject binary data masquerading as text)
    // Fast path: if string is ASCII (the vast majority of source code and configs),
    // we can avoid UTF-8 code point decoding entirely and operate directly on bytes.
    if s.is_ascii() {
        let b = s.as_bytes();
        let total_chars = b.len();
        let max_controls = total_chars / 100;
        let mut control_count = 0usize;

        for &byte in b {
            if (byte < 0x20 && byte != b'\t' && byte != b'\n' && byte != b'\r') || byte == 0x7F {
                control_count += 1;
                if control_count > max_controls {
                    return None;
                }
            }
        }
    } else {
        let mut control_count = 0usize;
        let mut total_chars = 0usize;

        for ch in s.chars() {
            total_chars += 1;
            if ch.is_control() && ch != '\t' && ch != '\n' && ch != '\r' {
                control_count += 1;
            }
        }

        // If control characters exceed 1%, reject as binary
        if control_count * 100 > total_chars {
            return None;
        }
    }

    Some(s)
}
