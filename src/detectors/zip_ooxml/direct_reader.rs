use flate2::read::DeflateDecoder;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use zip::ZipArchive;

use crate::budget::{BudgetReader, DecompressLimitReader, DetectOptions};
use crate::model::{DetectError, DetectionReport, EvidenceItem, Inspection, WarningItem};

use super::cd_check::{ZipCdPreCheckResult, ZipEntryMeta};
use super::finalize_ooxml_outcome;
use super::opc::{parse_content_types, parse_package_relationships, resolve_target_path};
use super::schemas::{self, classify_ooxml_content_type};
use super::xml::verify_main_part_xml;

pub(super) struct XmlPartSpec<'a> {
    pub(super) entry_name: &'a str,
    pub(super) missing_code: &'static str,
    pub(super) missing_msg: &'static str,
    pub(super) xml_limit_msg: &'static str,
}

fn run_bounded_xml_reader<S: Read, T, F>(
    stream: S,
    xml_limit_msg: &'static str,
    options: &DetectOptions,
    report: &mut DetectionReport,
    parse_fn: F,
) -> Option<T>
where
    F: FnOnce(&mut dyn BufRead) -> Result<T, WarningItem>,
{
    let remaining_total = options
        .max_total_decompressed_bytes
        .saturating_sub(report.stats.decompressed_bytes);
    let allowed = options.max_single_decompressed_bytes.min(remaining_total);
    if allowed == 0 {
        report.inspection = Inspection::Limited;
        report.warnings.push(WarningItem::new(
            "budget.total_decompressed_exceeded",
            "Cumulative decompressed byte budget exceeded",
        ));
        return None;
    }

    let mut limit_reader = DecompressLimitReader::new(stream, allowed);
    let result = {
        let mut buf_reader = BufReader::with_capacity(4096, &mut limit_reader);
        parse_fn(&mut buf_reader)
    };
    report.stats.decompressed_bytes += limit_reader.bytes_read();

    if limit_reader.is_exceeded() {
        report.inspection = Inspection::Limited;
        if report.stats.decompressed_bytes >= options.max_total_decompressed_bytes {
            report.warnings.push(WarningItem::new(
                "budget.total_decompressed_exceeded",
                "Cumulative decompressed byte budget exceeded",
            ));
        } else {
            report.warnings.push(WarningItem::new(
                "budget.xml_output_exceeded",
                xml_limit_msg,
            ));
        }
        return None;
    }

    match result {
        Ok(val) => Some(val),
        Err(warn) => {
            if warn.code == "ooxml.conflicting_relationships" {
                report.inspection = Inspection::Ambiguous;
            }
            report.warnings.push(warn);
            None
        }
    }
}

pub(super) fn stream_zip_xml_bounded<R: Read + Seek, T, F>(
    archive: &mut ZipArchive<&mut BudgetReader<R>>,
    spec: &XmlPartSpec<'_>,
    options: &DetectOptions,
    report: &mut DetectionReport,
    parse_fn: F,
) -> Option<T>
where
    F: FnOnce(&mut dyn BufRead) -> Result<T, WarningItem>,
{
    let entry = match archive.by_name(spec.entry_name) {
        Ok(e) => e,
        Err(_) => {
            report
                .warnings
                .push(WarningItem::new(spec.missing_code, spec.missing_msg));
            return None;
        }
    };

    run_bounded_xml_reader(entry, spec.xml_limit_msg, options, report, parse_fn)
}

enum DirectEntryOutcome<T> {
    Done(Option<T>),
    Fallback,
}

pub(super) enum DirectManifestOutcome {
    Done(super::packages::ApkManifestStatus),
    Fallback,
}

pub(super) fn verify_apk_manifest_direct<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    meta: &ZipEntryMeta,
    cd_offset: u64,
    options: &DetectOptions,
    report: &mut DetectionReport,
) -> Result<DirectManifestOutcome, DetectError> {
    match position_at_direct_entry(reader, meta, cd_offset) {
        Ok(true) => {}
        Ok(false) => {
            if reader.is_budget_exceeded() {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.source_read_exceeded",
                    "Total source read budget exceeded during detection",
                ));
                return Ok(DirectManifestOutcome::Done(
                    super::packages::ApkManifestStatus::BudgetExceeded,
                ));
            }
            return Ok(DirectManifestOutcome::Fallback);
        }
        Err(e) => {
            if reader.is_budget_exceeded() {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.source_read_exceeded",
                    "Total source read budget exceeded during detection",
                ));
                return Ok(DirectManifestOutcome::Done(
                    super::packages::ApkManifestStatus::BudgetExceeded,
                ));
            }
            return Err(e);
        }
    }

    let compressed = (&mut *reader).take(meta.compressed_size as u64);
    let status = match meta.compression_method {
        0 => super::packages::verify_apk_manifest_stream(compressed, options, report),
        8 => super::packages::verify_apk_manifest_stream(
            DeflateDecoder::new(compressed),
            options,
            report,
        ),
        _ => return Ok(DirectManifestOutcome::Fallback),
    };

    if status != super::packages::ApkManifestStatus::Valid && reader.is_budget_exceeded() {
        report.inspection = Inspection::Limited;
        if !report
            .warnings
            .iter()
            .any(|w| w.code == "budget.source_read_exceeded")
        {
            report.warnings.push(WarningItem::new(
                "budget.source_read_exceeded",
                "Total source read budget exceeded during detection",
            ));
        }
        return Ok(DirectManifestOutcome::Done(
            super::packages::ApkManifestStatus::BudgetExceeded,
        ));
    }

    Ok(DirectManifestOutcome::Done(status))
}

fn position_at_direct_entry<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    meta: &ZipEntryMeta,
    cd_offset: u64,
) -> Result<bool, DetectError> {
    let lfh_off = meta.local_header_offset as u64;
    reader.seek(SeekFrom::Start(lfh_off))?;
    let mut lfh = [0u8; 30];
    if reader.read_exact(&mut lfh).is_err() {
        return Ok(false);
    }
    if &lfh[0..4] != b"PK\x03\x04" {
        return Ok(false);
    }
    let lfh_fn_len = u16::from_le_bytes([lfh[26], lfh[27]]) as u64;
    let lfh_extra_len = u16::from_le_bytes([lfh[28], lfh[29]]) as u64;
    let Some(data_offset) = lfh_off
        .checked_add(30)
        .and_then(|v| v.checked_add(lfh_fn_len))
        .and_then(|v| v.checked_add(lfh_extra_len))
    else {
        return Ok(false);
    };
    if cd_offset > 0 && data_offset.saturating_add(meta.compressed_size as u64) > cd_offset {
        return Ok(false);
    }
    reader.seek(SeekFrom::Start(data_offset))?;
    Ok(true)
}

fn stream_direct_xml_bounded<R: Read + Seek, T, F>(
    reader: &mut BudgetReader<R>,
    meta: &ZipEntryMeta,
    cd_offset: u64,
    xml_limit_msg: &'static str,
    options: &DetectOptions,
    report: &mut DetectionReport,
    parse_fn: F,
) -> Result<DirectEntryOutcome<T>, DetectError>
where
    F: FnOnce(&mut dyn BufRead) -> Result<T, WarningItem>,
{
    match position_at_direct_entry(reader, meta, cd_offset) {
        Ok(true) => {}
        Ok(false) => {
            if reader.is_budget_exceeded() {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.source_read_exceeded",
                    "Total source read budget exceeded during detection",
                ));
                return Ok(DirectEntryOutcome::Done(None));
            }
            return Ok(DirectEntryOutcome::Fallback);
        }
        Err(e) => {
            if reader.is_budget_exceeded() {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.source_read_exceeded",
                    "Total source read budget exceeded during detection",
                ));
                return Ok(DirectEntryOutcome::Done(None));
            }
            return Err(e);
        }
    }

    let compressed = (&mut *reader).take(meta.compressed_size as u64);
    let parsed = match meta.compression_method {
        0 => run_bounded_xml_reader(compressed, xml_limit_msg, options, report, parse_fn),
        8 => run_bounded_xml_reader(
            DeflateDecoder::new(compressed),
            xml_limit_msg,
            options,
            report,
            parse_fn,
        ),
        _ => return Ok(DirectEntryOutcome::Fallback),
    };

    if parsed.is_none() && reader.is_budget_exceeded() {
        report.inspection = Inspection::Limited;
        if !report
            .warnings
            .iter()
            .any(|w| w.code == "budget.source_read_exceeded")
        {
            report.warnings.push(WarningItem::new(
                "budget.source_read_exceeded",
                "Total source read budget exceeded during detection",
            ));
        }
    }

    Ok(DirectEntryOutcome::Done(parsed))
}

pub(super) fn try_detect_ooxml_from_cd_cache<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    cd_check: &ZipCdPreCheckResult,
    options: &DetectOptions,
    report: &mut DetectionReport,
) -> Result<Option<()>, DetectError> {
    if cd_check.is_zip64 || cd_check.has_mimetype || !cd_check.has_content_types {
        return Ok(None);
    }

    // If _rels/.rels is absent from the verified Central Directory, no need to reopen ZipArchive.
    if !cd_check.has_rels {
        report.stats.entries_inspected = cd_check.cd_total_entries;
        report
            .evidence
            .push(EvidenceItem::new("zip.central_directory"));
        report.warnings.push(WarningItem::new(
            "ooxml.required_relationship_missing",
            "Cannot open _rels/.rels entry in package",
        ));
        return Ok(Some(()));
    }

    let (Some(rels_meta), Some(ct_meta)) = (
        cd_check.rels_entry.as_ref(),
        cd_check.content_types_entry.as_ref(),
    ) else {
        return Ok(None);
    };

    // Validate LFH signatures upfront so any corrupted LFH cleanly falls back without partial state.
    let rels_parsed = match stream_direct_xml_bounded(
        reader,
        rels_meta,
        cd_check.cd_offset,
        "Decompressed size of _rels/.rels exceeded single XML limit",
        options,
        report,
        |r| parse_package_relationships(r),
    )? {
        DirectEntryOutcome::Fallback => return Ok(None),
        DirectEntryOutcome::Done(res) => res,
    };

    report.stats.entries_inspected = cd_check.cd_total_entries;
    report
        .evidence
        .push(EvidenceItem::new("zip.central_directory"));

    let Some(parsed_rel) = rels_parsed else {
        return Ok(Some(()));
    };
    let Some(main_rel_target) = parsed_rel else {
        report.warnings.push(WarningItem::new(
            "ooxml.required_relationship_missing",
            "No officeDocument core relationship found in _rels/.rels",
        ));
        return Ok(Some(()));
    };

    report
        .evidence
        .push(EvidenceItem::new("ooxml.package_relationships"));

    let normalized_target = match resolve_target_path("", &main_rel_target) {
        Some(t) => t,
        None => {
            report.warnings.push(WarningItem::new(
                "ooxml.invalid_target_path",
                format!(
                    "Relationship target path '{}' resolves outside package root",
                    main_rel_target
                ),
            ));
            return Ok(Some(()));
        }
    };

    let ct_parsed = match stream_direct_xml_bounded(
        reader,
        ct_meta,
        cd_check.cd_offset,
        "Decompressed size of [Content_Types].xml exceeded single XML limit",
        options,
        report,
        |r| parse_content_types(r, &normalized_target),
    )? {
        DirectEntryOutcome::Fallback => {
            return open_zip_archive_and_finish_main(
                reader,
                &normalized_target,
                None,
                options,
                report,
            )
            .map(Some);
        }
        DirectEntryOutcome::Done(res) => res,
    };

    let Some(parsed_ct) = ct_parsed else {
        return Ok(Some(()));
    };
    let Some(content_type) = parsed_ct else {
        report.warnings.push(WarningItem::new(
            "ooxml.content_type_not_found",
            format!(
                "No ContentType entry found for part '/{}'",
                normalized_target
            ),
        ));
        return Ok(Some(()));
    };

    report
        .evidence
        .push(EvidenceItem::new("ooxml.content_types"));

    let format_kind = match classify_ooxml_content_type(&content_type) {
        Some(k) => k,
        None => {
            report.warnings.push(WarningItem::new(
                "ooxml.unsupported_content_type",
                format!(
                    "Package target has unsupported OOXML content type: {}",
                    content_type
                ),
            ));
            return Ok(Some(()));
        }
    };

    if let Some(main_meta) = cd_check
        .xml_entries
        .iter()
        .find(|entry| entry.name == normalized_target)
    {
        if let DirectEntryOutcome::Done(outcome_opt) = stream_direct_xml_bounded(
            reader,
            main_meta,
            cd_check.cd_offset,
            "Decompressed size of main part exceeded single XML limit",
            options,
            report,
            |r| verify_main_part_xml(r, format_kind, options.max_xml_depth),
        )? {
            if let Some(outcome) = outcome_opt {
                finalize_ooxml_outcome(outcome, options, report);
            }
            return Ok(Some(()));
        }
    } else if !cd_check.xml_entries_capped
        && normalized_target.len() > 4
        && normalized_target.as_bytes()[normalized_target.len() - 4..].eq_ignore_ascii_case(b".xml")
    {
        report.warnings.push(WarningItem::new(
            "ooxml.main_part_missing",
            "Failed to open main document part in package",
        ));
        return Ok(Some(()));
    }

    open_zip_archive_and_finish_main(
        reader,
        &normalized_target,
        Some(format_kind),
        options,
        report,
    )
    .map(Some)
}

fn open_zip_archive_and_finish_main<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    normalized_target: &str,
    format_kind: Option<schemas::OoxmlFormatKind>,
    options: &DetectOptions,
    report: &mut DetectionReport,
) -> Result<(), DetectError> {
    if let Err(e) = reader.seek(SeekFrom::Start(0)) {
        if reader.is_budget_exceeded() {
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "budget.source_read_exceeded",
                "Total source read budget exceeded during detection",
            ));
            return Ok(());
        }
        return Err(DetectError::Io(e));
    }
    let mut archive = match ZipArchive::new(&mut *reader) {
        Ok(a) => a,
        Err(e) => {
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "zip.truncated_or_corrupted",
                format!("Failed to parse ZIP archive: {}", e),
            ));
            return Ok(());
        }
    };

    let format_kind = match format_kind {
        Some(k) => k,
        None => {
            let ct_spec = XmlPartSpec {
                entry_name: "[Content_Types].xml",
                missing_code: "ooxml.content_types_missing",
                missing_msg: "Cannot open [Content_Types].xml entry",
                xml_limit_msg: "Decompressed size of [Content_Types].xml exceeded single XML limit",
            };
            let Some(Some(content_type)) =
                stream_zip_xml_bounded(&mut archive, &ct_spec, options, report, |r| {
                    parse_content_types(r, normalized_target)
                })
            else {
                return Ok(());
            };
            report
                .evidence
                .push(EvidenceItem::new("ooxml.content_types"));
            match classify_ooxml_content_type(&content_type) {
                Some(k) => k,
                None => {
                    report.warnings.push(WarningItem::new(
                        "ooxml.unsupported_content_type",
                        format!(
                            "Package target has unsupported OOXML content type: {}",
                            content_type
                        ),
                    ));
                    return Ok(());
                }
            }
        }
    };

    let main_spec = XmlPartSpec {
        entry_name: normalized_target,
        missing_code: "ooxml.main_part_missing",
        missing_msg: "Failed to open main document part in package",
        xml_limit_msg: "Decompressed size of main part exceeded single XML limit",
    };
    if let Some(outcome) = stream_zip_xml_bounded(&mut archive, &main_spec, options, report, |r| {
        verify_main_part_xml(r, format_kind, options.max_xml_depth)
    }) {
        finalize_ooxml_outcome(outcome, options, report);
    }
    Ok(())
}
