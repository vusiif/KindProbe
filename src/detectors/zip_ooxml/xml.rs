use std::io::BufRead;
use quick_xml::events::Event;
use quick_xml::reader::NsReader;
use crate::model::WarningItem;
use super::schemas::OoxmlFormatKind;

#[derive(Debug, PartialEq, Eq)]
pub enum MainPartXmlOutcome {
    Valid(OoxmlFormatKind),
    DepthExceededValidRoot(OoxmlFormatKind),
    DepthExceededInvalidRoot,
    InvalidRoot(String),
}

/// Verifies that the main document/workbook/presentation part XML has a valid
/// root element corresponding to the expected OOXML format kind and adheres to max_depth.
pub fn verify_main_part_xml<R: BufRead>(
    reader: R,
    expected_kind: OoxmlFormatKind,
    max_depth: usize,
) -> Result<MainPartXmlOutcome, WarningItem> {
    let mut reader = NsReader::from_reader(reader);
    reader.config_mut().trim_text(false);
    reader.config_mut().check_comments = false;

    let mut current_depth: usize = 0;
    let mut root_seen = false;
    let mut root_is_valid = false;
    let mut root_closed = false;
    let mut has_multiple_roots = false;
    let mut depth_exceeded = false;

    let expected_local = expected_kind.expected_root_local();

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                current_depth += 1;
                if current_depth > max_depth {
                    depth_exceeded = true;
                    break;
                }

                if !root_seen {
                    root_seen = true;
                    let (resolve, local) = reader.resolve_element(e.name());
                    if local.as_ref() == expected_local {
                        if let quick_xml::name::ResolveResult::Bound(quick_xml::name::Namespace(ns)) = resolve {
                            if expected_kind.matches_namespace(ns) {
                                root_is_valid = true;
                            }
                        }
                    }
                } else if root_closed {
                    has_multiple_roots = true;
                    break;
                }
            }
            Ok(Event::Empty(ref e)) => {
                if !root_seen {
                    root_seen = true;
                    let (resolve, local) = reader.resolve_element(e.name());
                    if local.as_ref() == expected_local {
                        if let quick_xml::name::ResolveResult::Bound(quick_xml::name::Namespace(ns)) = resolve {
                            if expected_kind.matches_namespace(ns) {
                                root_is_valid = true;
                            }
                        }
                    }
                    root_closed = true;
                } else if root_closed {
                    has_multiple_roots = true;
                    break;
                }
            }
            Ok(Event::End(_)) => {
                if current_depth > 0 {
                    current_depth -= 1;
                    if current_depth == 0 {
                        root_closed = true;
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(WarningItem::new(
                    "ooxml.main_part_xml_error",
                    format!("XML error parsing main part: {}", e),
                ));
            }
            _ => (),
        }
        buf.clear();
    }

    if has_multiple_roots {
        return Ok(MainPartXmlOutcome::InvalidRoot(
            "Main part XML contains multiple root elements".to_string(),
        ));
    }

    if depth_exceeded {
        if root_is_valid {
            return Ok(MainPartXmlOutcome::DepthExceededValidRoot(expected_kind));
        } else {
            return Ok(MainPartXmlOutcome::DepthExceededInvalidRoot);
        }
    }

    if !root_seen || !root_is_valid {
        return Ok(MainPartXmlOutcome::InvalidRoot(format!(
            "Main part XML does not have valid <{}> root element with expected namespace",
            String::from_utf8_lossy(expected_local)
        )));
    }

    if !root_closed {
        return Ok(MainPartXmlOutcome::InvalidRoot(
            "Main part XML root element was not closed".to_string(),
        ));
    }

    Ok(MainPartXmlOutcome::Valid(expected_kind))
}
