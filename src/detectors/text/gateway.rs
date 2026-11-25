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

    #[inline]
fn scan_ascii_slice(
    slice: &[u8],
    base_idx: usize,
    control_count: &mut usize,
    max_controls: usize,
    ascii_end: &mut usize,
) -> bool {
    for (offset, &b) in slice.iter().enumerate() {
        if b == 0 {
            return false;
        }
        if b >= 0x80 {
            *ascii_end = base_idx + offset;
            return true;
        }
        if (b < 0x20 && b != b'\t' && b != b'\n' && b != b'\r') || b == 0x7F {
            *control_count += 1;
            if *control_count > max_controls {
                return false;
            }
        }
    }
    true
}
    // 2. Single-pass unified ASCII / NUL / control character scan (SWAR 8-byte fast path)
    let total_len = raw.len();
    let max_controls = total_len / 100;
    let mut control_count = 0usize;
    let mut ascii_end = raw.len();

    let mut idx = 0usize;
    let (chunks, remainder) = raw.as_chunks::<8>();
    for chunk in chunks {
        let w = u64::from_ne_bytes(*chunk);
        let has_non_ascii = (w & 0x8080_8080_8080_8080) != 0;
        let has_lt_0x20 =
            (w.wrapping_sub(0x2020_2020_2020_2020) & !w & 0x8080_8080_8080_8080) != 0;
        let x7f = w ^ 0x7F7F_7F7F_7F7F_7F7F;
        let has_0x7f =
            (x7f.wrapping_sub(0x0101_0101_0101_0101) & !x7f & 0x8080_8080_8080_8080) != 0;

        if (has_non_ascii || has_lt_0x20 || has_0x7f)
            && !scan_ascii_slice(chunk, idx, &mut control_count, max_controls, &mut ascii_end)
        {
            return None;
        }
        if ascii_end != raw.len() {
            break;
        }
        idx += 8;
    }

    if ascii_end == raw.len()
        && !scan_ascii_slice(remainder, idx, &mut control_count, max_controls, &mut ascii_end)
    {
        return None;
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
