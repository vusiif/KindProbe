use std::io::BufRead;
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
    let trimmed = raw.trim();
    let stripped = trimmed.trim_start_matches('/');
    if is_normalized_opc_path(stripped) {
        return stripped.to_string();
    }
    let s = trimmed.replace('\\', "/");
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
    let trimmed = raw.trim();
    let stripped = trimmed.trim_start_matches('/');
    if is_normalized_opc_path(stripped) {
        std::borrow::Cow::Borrowed(stripped)
    } else {
        std::borrow::Cow::Owned(normalize_opc_path(raw))
    }
}

/// Resolves a target relative to a base directory within an OPC package.
pub fn resolve_target_path(base_dir: &str, target: &str) -> Option<String> {
    let trimmed = target.trim();
    let has_backslash = trimmed.as_bytes().contains(&b'\\');
    let t_cow = if has_backslash {
        std::borrow::Cow::Owned(trimmed.replace('\\', "/"))
    } else {
        std::borrow::Cow::Borrowed(trimmed)
    };
    let t = t_cow.as_ref();
    if t.starts_with('/') {
        let norm = normalize_opc_path(t);
        if norm.is_empty() {
            None
        } else {
            Some(norm)
        }
    } else {
        let combined = if base_dir.is_empty() {
            t.to_string()
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
pub fn parse_package_relationships<R: BufRead>(reader: R) -> Result<Option<String>, WarningItem> {
    let mut reader = XmlReader::from_reader(reader);
    reader.config_mut().trim_text(false);
    reader.config_mut().check_comments = false;

    let mut target_val: Option<String> = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                if e.name().as_ref() == "Relationship" {
                    let mut is_office_doc = false;
                    let mut rel_target = None;

                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == "Type" {
                            let val = attr.value.as_ref();
                            if val == TRANSITIONAL_REL_OFFICE_DOC || val == STRICT_REL_OFFICE_DOC {
                                is_office_doc = true;
                            }
                        } else if attr.key.as_ref() == "Target" {
                            rel_target = Some(attr.value);
                        }
                    }

                    if is_office_doc {
                        if let Some(target_str) = rel_target {
                            if !target_str.is_empty() {
                                target_val = Some(target_str.into_owned());
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
pub fn parse_content_types<R: BufRead>(
    reader: R,
    normalized_target: &str,
) -> Result<Option<String>, WarningItem> {
    let mut reader = XmlReader::from_reader(reader);
    reader.config_mut().trim_text(false);
    reader.config_mut().check_comments = false;

    let target_ext = normalized_target.rsplit('.').next().unwrap_or("");

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
                if name.as_ref() == "Override" {
                    let mut matches = false;
                    let mut ct_val = None;
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == "PartName" {
                            let val = attr.value.as_ref();
                            let trimmed_val = val.strip_prefix('/').unwrap_or(val);
                            if trimmed_val.eq_ignore_ascii_case(normalized_target) {
                                matches = true;
                            }
                        } else if attr.key.as_ref() == "ContentType" {
                            ct_val = Some(attr.value);
                        }
                    }
                    if matches && matched_override.is_none() {
                        if let Some(val) = ct_val {
                            matched_override = Some(val.into_owned());
                        }
                    }
                } else if name.as_ref() == "Default" {
                    let mut matches = false;
                    let mut ct_val = None;
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == "Extension" {
                            if attr.value.eq_ignore_ascii_case(target_ext) {
                                matches = true;
                            }
                        } else if attr.key.as_ref() == "ContentType" {
                            ct_val = Some(attr.value);
                        }
                    }
                    if matches && matched_default.is_none() {
                        if let Some(val) = ct_val {
                            matched_default = Some(val.into_owned());
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
