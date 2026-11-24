use quick_xml::events::Event;
use quick_xml::reader::Reader as XmlReader;
use crate::model::WarningItem;

pub const TRANSITIONAL_REL_OFFICE_DOC: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
pub const STRICT_REL_OFFICE_DOC: &str =
    "http://purl.oclc.org/ooxml/officeDocument/relationships/officeDocument";

fn is_normalized_opc_path(raw: &str) -> bool {
    if raw.is_empty() {
        return true;
    }
    let b = raw.as_bytes();
    if b[0] == b'/' || b[0] == b'\\' || b[0].is_ascii_whitespace() {
        return false;
    }
    let last = b[b.len() - 1];
    if last == b'/' || last == b'\\' || last.is_ascii_whitespace() {
        return false;
    }
    let mut prev = 0u8;
    for &c in b {
        if c == b'\\' || c.is_ascii_whitespace() {
            return false;
        }
        if c == b'/' && prev == b'/' {
            return false;
        }
        prev = c;
    }
    for seg in raw.split('/') {
        if seg == "." || seg == ".." {
            return false;
        }
    }
    true
}

/// Normalizes an OPC relative or absolute path relative to package root.
/// Returns normalized path without leading slash, e.g. "word/document.xml".
pub fn normalize_opc_path(raw: &str) -> String {
    if is_normalized_opc_path(raw) {
        return raw.to_string();
    }
    let s = raw.trim().replace('\\', "/");
    let parts = s.split('/');
    let mut stack = Vec::new();

    for p in parts {
        let seg = p.trim();
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            stack.pop();
        } else {
            stack.push(seg);
        }
    }
    stack.join("/")
}

/// Normalizes an OPC relative or absolute path, returning a borrowed `Cow`
/// if already clean to avoid heap allocations.
pub fn normalize_opc_path_cow<'a>(raw: &'a str) -> std::borrow::Cow<'a, str> {
    if is_normalized_opc_path(raw) {
        std::borrow::Cow::Borrowed(raw)
    } else {
        std::borrow::Cow::Owned(normalize_opc_path(raw))
    }
}

/// Resolves a target relative to a base directory within an OPC package.
pub fn resolve_target_path(base_dir: &str, target: &str) -> Option<String> {
    let t = target.trim().replace('\\', "/");
    if t.starts_with('/') {
        let norm = normalize_opc_path(&t);
        if norm.is_empty() {
            None
        } else {
            Some(norm)
        }
    } else {
        let combined = if base_dir.is_empty() {
            t
        } else {
            format!("{}/{}", base_dir, t)
        };
        let norm = normalize_opc_path(&combined);
        if norm.is_empty() {
            None
        } else {
            Some(norm)
        }
    }
}

/// Parses `_rels/.rels` XML to find the officeDocument core relationship target.
pub fn parse_package_relationships(xml_bytes: &[u8]) -> Result<Option<String>, WarningItem> {
    let mut reader = XmlReader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);

    let mut target_val: Option<String> = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                if e.name().as_ref() == b"Relationship" {
                    let mut is_office_doc = false;
                    let mut rel_target = None;

                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"Type" {
                            let val = attr.value.as_ref();
                            if val == TRANSITIONAL_REL_OFFICE_DOC.as_bytes()
                                || val == STRICT_REL_OFFICE_DOC.as_bytes()
                            {
                                is_office_doc = true;
                            }
                        } else if attr.key.as_ref() == b"Target" {
                            rel_target = Some(attr.value);
                        }
                    }

                    if is_office_doc {
                        if let Some(target_bytes) = rel_target {
                            if !target_bytes.is_empty() {
                                target_val = Some(String::from_utf8_lossy(&target_bytes).to_string());
                                break;
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(WarningItem::new(
                    "ooxml.rels_xml_error",
                    format!("XML error parsing _rels/.rels: {}", e),
                ));
            }
            _ => (),
        }
        buf.clear();
    }

    Ok(target_val)
}

/// Parses `[Content_Types].xml` to locate the content type of the target part.
pub fn parse_content_types(
    xml_bytes: &[u8],
    normalized_target: &str,
) -> Result<Option<String>, WarningItem> {
    let mut reader = XmlReader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);

    let target_part_name = format!("/{}", normalized_target).to_ascii_lowercase();
    let target_part_bytes = target_part_name.as_bytes();
    let target_ext = normalized_target.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    let target_ext_bytes = target_ext.as_bytes();

    let mut matched_override: Option<String> = None;
    let mut matched_default: Option<String> = None;

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                if matched_override.is_some() {
                    buf.clear();
                    continue;
                }
                let name = e.name();
                if name.as_ref() == b"Override" {
                    let mut matches = false;
                    let mut ct_bytes = None;
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"PartName" {
                            if attr.value.eq_ignore_ascii_case(target_part_bytes) {
                                matches = true;
                            }
                        } else if attr.key.as_ref() == b"ContentType" {
                            ct_bytes = Some(attr.value);
                        }
                    }
                    if matches && matched_override.is_none() {
                        if let Some(val) = ct_bytes {
                            matched_override = Some(String::from_utf8_lossy(&val).to_string());
                        }
                    }
                } else if name.as_ref() == b"Default" {
                    let mut matches = false;
                    let mut ct_bytes = None;
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"Extension" {
                            if attr.value.eq_ignore_ascii_case(target_ext_bytes) {
                                matches = true;
                            }
                        } else if attr.key.as_ref() == b"ContentType" {
                            ct_bytes = Some(attr.value);
                        }
                    }
                    if matches && matched_default.is_none() {
                        if let Some(val) = ct_bytes {
                            matched_default = Some(String::from_utf8_lossy(&val).to_string());
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(WarningItem::new(
                    "ooxml.content_types_xml_error",
                    format!("XML error parsing [Content_Types].xml: {}", e),
                ));
            }
            _ => (),
        }
        buf.clear();
    }

    if let Some(ct) = matched_override {
        return Ok(Some(ct));
    }
    if let Some(ct) = matched_default {
        return Ok(Some(ct));
    }

    Ok(None)
}
