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

    // 2. Single-pass unified ASCII / NUL / control character scan
    let total_len = raw.len();
    let max_controls = total_len / 100;
    let mut control_count = 0usize;
    let mut ascii_end = raw.len();

    for (i, &b) in raw.iter().enumerate() {
        if b == 0 {
            return None;
        }
        if b >= 0x80 {
            ascii_end = i;
            break;
        }
        if (b < 0x20 && b != b'\t' && b != b'\n' && b != b'\r') || b == 0x7F {
            control_count += 1;
            if control_count > max_controls {
                return None;
            }
        }
    }

    // Pure ASCII fast path: guaranteed 100% valid UTF-8, no NULs, and valid control ratio
    if ascii_end == raw.len() {
        return std::str::from_utf8(raw).ok();
    }

    // 3. Fallback for non-ASCII UTF-8: check remaining slice for NUL bytes
    if raw[ascii_end..].contains(&0) {
        return None;
    }

    // 4. UTF-8 conversion with window-boundary truncation tolerance
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

    // 5. Control character ratio check for non-ASCII text
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

    Some(s)
}
