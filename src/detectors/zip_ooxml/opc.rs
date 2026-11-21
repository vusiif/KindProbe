use std::collections::HashMap;
use quick_xml::events::Event;
use quick_xml::reader::Reader as XmlReader;
use crate::model::WarningItem;

pub const TRANSITIONAL_REL_OFFICE_DOC: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
pub const STRICT_REL_OFFICE_DOC: &str =
    "http://purl.oclc.org/ooxml/officeDocument/relationships/officeDocument";

/// Normalizes an OPC relative or absolute path relative to package root.
/// Returns normalized path without leading slash, e.g. "word/document.xml".
pub fn normalize_opc_path(raw: &str) -> String {
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
                    let mut rel_type = String::new();
                    let mut rel_target = String::new();

                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"Type" {
                            rel_type = String::from_utf8_lossy(&attr.value).to_string();
                        } else if attr.key.as_ref() == b"Target" {
                            rel_target = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }

                    if (rel_type == TRANSITIONAL_REL_OFFICE_DOC || rel_type == STRICT_REL_OFFICE_DOC)
                        && !rel_target.is_empty()
                    {
                        target_val = Some(rel_target);
                        break;
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
    let target_ext = normalized_target.rsplit('.').next().unwrap_or("").to_ascii_lowercase();

    let mut overrides = HashMap::new();
    let mut defaults = HashMap::new();

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"Override" {
                    let mut part_name = String::new();
                    let mut content_type = String::new();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"PartName" {
                            part_name = String::from_utf8_lossy(&attr.value).to_ascii_lowercase();
                        } else if attr.key.as_ref() == b"ContentType" {
                            content_type = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                    if !part_name.is_empty() && !content_type.is_empty() {
                        overrides.insert(part_name, content_type);
                    }
                } else if name.as_ref() == b"Default" {
                    let mut ext = String::new();
                    let mut content_type = String::new();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"Extension" {
                            ext = String::from_utf8_lossy(&attr.value).to_ascii_lowercase();
                        } else if attr.key.as_ref() == b"ContentType" {
                            content_type = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                    if !ext.is_empty() && !content_type.is_empty() {
                        defaults.insert(ext, content_type);
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

    if let Some(ct) = overrides.get(&target_part_name) {
        return Ok(Some(ct.clone()));
    }
    if let Some(ct) = defaults.get(&target_ext) {
        return Ok(Some(ct.clone()));
    }

    Ok(None)
}
