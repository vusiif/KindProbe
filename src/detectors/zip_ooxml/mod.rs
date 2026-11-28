pub mod cd_check;
pub mod epub;
pub mod opc;
pub mod packages;
pub mod schemas;
pub mod xml;

use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use zip::ZipArchive;

use crate::budget::{BudgetReader, DecompressLimitReader, DetectOptions};
use crate::model::{
    DetectError, DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
    Verification, WarningItem,
};

use cd_check::precheck_zip_central_directory;
use epub::check_epub_or_odf;
use packages::check_zip_packages;
use opc::{parse_content_types, parse_package_relationships, resolve_target_path};
use schemas::classify_ooxml_content_type;
use xml::{verify_main_part_xml, MainPartXmlOutcome};

struct XmlPartSpec<'a> {
    entry_name: &'a str,
    missing_code: &'static str,
    missing_msg: &'static str,
    xml_limit_msg: &'static str,
}

fn stream_zip_xml_bounded<R: Read + Seek, T, F>(
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
            report.warnings.push(WarningItem::new(spec.missing_code, spec.missing_msg));
            return None;
        }
    };

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

    let mut limit_reader = DecompressLimitReader::new(entry, allowed);
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
            report.warnings.push(WarningItem::new("budget.xml_output_exceeded", spec.xml_limit_msg));
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

fn finalize_zip_package(
    report: &mut DetectionReport,
    entries_inspected: usize,
    format_id: &str,
    category: &str,
    rec_ext: &str,
    accepted_exts: &[&str],
    evidence_code: &str,
) {
    report.stats.entries_inspected = entries_inspected;
    report.evidence.push(EvidenceItem::new("zip.central_directory"));
    report.identification = Identification::Format;
    report.format_id = Some(format_id.to_string());
    report.category = category.to_string();
    report.container_id = Some("zip".to_string());
    report.verification = Verification::Structure;
    report.inspection = Inspection::Complete;
    report.recommended_extension = Some(rec_ext.to_string());
    report.accepted_extensions = accepted_exts.iter().map(|s| s.to_string()).collect();
    report.evidence.push(EvidenceItem::new(evidence_code));
}

fn finalize_ooxml_outcome(
    outcome: MainPartXmlOutcome,
    options: &DetectOptions,
    report: &mut DetectionReport,
) {
    match outcome {
        MainPartXmlOutcome::Valid(kind) => {
            report.identification = Identification::Format;
            report.format_id = Some(kind.format_id().to_string());
            report.category = "document".to_string();
            report.container_id = Some("zip".to_string());
            report.verification = Verification::Structure;
            report.inspection = Inspection::Complete;
            report.recommended_extension = Some(kind.recommended_extension().to_string());
            report.accepted_extensions = vec![kind.recommended_extension().to_string()];
            report.evidence.push(EvidenceItem::new(kind.evidence_code()));
        }
        MainPartXmlOutcome::DepthExceededValidRoot(kind) => {
            report.identification = Identification::Format;
            report.format_id = Some(kind.format_id().to_string());
            report.category = "document".to_string();
            report.container_id = Some("zip".to_string());
            report.verification = Verification::Structure;
            report.inspection = Inspection::Limited;
            report.recommended_extension = Some(kind.recommended_extension().to_string());
            report.accepted_extensions = vec![kind.recommended_extension().to_string()];
            report.evidence.push(EvidenceItem::new(kind.evidence_code()));
            report.warnings.push(WarningItem::new(
                "budget.xml_depth_exceeded",
                format!(
                    "XML element depth ({}) exceeded allowed maximum ({})",
                    options.max_xml_depth + 1,
                    options.max_xml_depth
                ),
            ));
        }
        MainPartXmlOutcome::DepthExceededInvalidRoot => {
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "ooxml.invalid_root_element",
                "Main part XML does not have valid root element for the detected content type",
            ));
            report.warnings.push(WarningItem::new(
                "budget.xml_depth_exceeded",
                format!(
                    "XML element depth ({}) exceeded allowed maximum ({})",
                    options.max_xml_depth + 1,
                    options.max_xml_depth
                ),
            ));
        }
        MainPartXmlOutcome::InvalidRoot(msg) => {
            report.warnings.push(WarningItem::new("ooxml.invalid_root_element", msg));
        }
    }
}

pub fn detect_zip_ooxml<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    options: &DetectOptions,
) -> Result<DetectionReport, DetectError> {
    let mut report = DetectionReport {
        schema_version: 1,
        format_id: None,
        category: "container".to_string(),
        container_id: Some("zip".to_string()),
        identification: Identification::ContainerOnly,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: None,
        accepted_extensions: Vec::new(),
        extension_status: ExtensionStatus::Undetermined,
        evidence: Vec::new(),
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    };

    // Pre-check ZIP End of Central Directory and Central Directory Headers
    let cd_check = match precheck_zip_central_directory(reader, options, &mut report)? {
        Some(res) => res,
        None => return Ok(report),
    };

    // Check for OOXML indicators (already collected during central directory pre-check)
    let has_content_types = cd_check.has_content_types;
    let has_rels = cd_check.has_rels;
    let has_ooxml_subdirs = cd_check.has_ooxml_subdirs;

    // OPC-based ZIP packages that legitimately contain [Content_Types].xml / _rels/.rels
    if cd_check.has_msix_manifest {
        finalize_zip_package(
            &mut report,
            cd_check.cd_total_entries,
            "msix",
            "package",
            "msix",
            &["msix", "appx"],
            "zip.msix.manifest",
        );
        return Ok(report);
    }
    if cd_check.has_vsix_manifest && !has_ooxml_subdirs {
        finalize_zip_package(
            &mut report,
            cd_check.cd_total_entries,
            "vsix",
            "package",
            "vsix",
            &["vsix"],
            "zip.vsix.manifest",
        );
        return Ok(report);
    }
    if cd_check.has_nupkg_nuspec && !has_ooxml_subdirs {
        finalize_zip_package(
            &mut report,
            cd_check.cd_total_entries,
            "nupkg",
            "package",
            "nupkg",
            &["nupkg"],
            "zip.nupkg.nuspec",
        );
        return Ok(report);
    }

    // Fast short-circuit: non-OOXML and non-EPUB/ODF archives (APK, IPA, JAR, KMZ, plain ZIP)
    // do not need ZipArchive instantiation, second EOCD search, or stream rewinding.
    if !has_content_types && !has_rels && !has_ooxml_subdirs && !cd_check.has_mimetype {
        report.stats.entries_inspected = cd_check.cd_total_entries;
        report.evidence.push(EvidenceItem::new("zip.central_directory"));

        // 1. Android Package (APK): AndroidManifest.xml and/or classes.dex
        if cd_check.has_apk_manifest || cd_check.has_apk_dex {
            report.identification = Identification::Format;
            report.format_id = Some("apk".to_string());
            report.category = "package".to_string();
            report.container_id = Some("zip".to_string());
            report.verification = Verification::Structure;
            report.inspection = Inspection::Complete;
            report.recommended_extension = Some("apk".to_string());
            report.accepted_extensions = vec!["apk".to_string(), "xapk".to_string()];
            if cd_check.has_apk_manifest {
                report.evidence.push(EvidenceItem::new("zip.apk.manifest"));
            }
            if cd_check.has_apk_dex {
                report.evidence.push(EvidenceItem::new("zip.apk.dex"));
            }
            return Ok(report);
        }

        // 1b. Android App Bundle (AAB): BundleConfig.pb
        if cd_check.has_aab_bundle {
            report.identification = Identification::Format;
            report.format_id = Some("aab".to_string());
            report.category = "package".to_string();
            report.container_id = Some("zip".to_string());
            report.verification = Verification::Structure;
            report.inspection = Inspection::Complete;
            report.recommended_extension = Some("aab".to_string());
            report.accepted_extensions = vec!["aab".to_string()];
            report.evidence.push(EvidenceItem::new("zip.aab.bundle"));
            return Ok(report);
        }

        // 2. iOS Application Archive (IPA): Payload/*.app/Info.plist
        if cd_check.has_ipa_payload {
            report.identification = Identification::Format;
            report.format_id = Some("ipa".to_string());
            report.category = "package".to_string();
            report.container_id = Some("zip".to_string());
            report.verification = Verification::Structure;
            report.inspection = Inspection::Complete;
            report.recommended_extension = Some("ipa".to_string());
            report.accepted_extensions = vec!["ipa".to_string()];
            report.evidence.push(EvidenceItem::new("zip.ipa.payload"));
            return Ok(report);
        }

        // 3. Java Archive (JAR): META-INF/MANIFEST.MF
        if cd_check.has_jar_manifest {
            report.identification = Identification::Format;
            report.format_id = Some("jar".to_string());
            report.category = "package".to_string();
            report.container_id = Some("zip".to_string());
            report.verification = Verification::Structure;
            report.inspection = Inspection::Complete;
            report.recommended_extension = Some("jar".to_string());
            report.accepted_extensions = vec!["jar".to_string(), "war".to_string(), "ear".to_string()];
            report.evidence.push(EvidenceItem::new("zip.jar.manifest"));
            return Ok(report);
        }

        // 4. Keyhole Markup Zip (KMZ): doc.kml
        if cd_check.has_kmz_kml {
            report.identification = Identification::Format;
            report.format_id = Some("kmz".to_string());
            report.category = "geospatial".to_string();
            report.container_id = Some("zip".to_string());
            report.verification = Verification::Structure;
            report.inspection = Inspection::Complete;
            report.recommended_extension = Some("kmz".to_string());
            report.accepted_extensions = vec!["kmz".to_string(), "kml".to_string()];
            report.evidence.push(EvidenceItem::new("zip.kmz.kml"));
            return Ok(report);
        }

        // 4b. NumPy NPZ archive: contains *.npy arrays
        if cd_check.has_npz_array {
            report.identification = Identification::Format;
            report.format_id = Some("npz".to_string());
            report.category = "scientific".to_string();
            report.container_id = Some("zip".to_string());
            report.verification = Verification::Structure;
            report.inspection = Inspection::Complete;
            report.recommended_extension = Some("npz".to_string());
            report.accepted_extensions = vec!["npz".to_string()];
            report.evidence.push(EvidenceItem::new("zip.npz.array"));
            return Ok(report);
        }

        // 4c. Python Wheel (.whl), VS Code Extension (.vsix), NuGet (.nupkg), Firefox Add-on (.xpi)
        if cd_check.has_whl_dist_info {
            report.identification = Identification::Format;
            report.format_id = Some("whl".to_string());
            report.category = "package".to_string();
            report.container_id = Some("zip".to_string());
            report.verification = Verification::Structure;
            report.inspection = Inspection::Complete;
            report.recommended_extension = Some("whl".to_string());
            report.accepted_extensions = vec!["whl".to_string()];
            report.evidence.push(EvidenceItem::new("zip.whl.wheel"));
            return Ok(report);
        }
        if cd_check.has_vsix_manifest {
            report.identification = Identification::Format;
            report.format_id = Some("vsix".to_string());
            report.category = "package".to_string();
            report.container_id = Some("zip".to_string());
            report.verification = Verification::Structure;
            report.inspection = Inspection::Complete;
            report.recommended_extension = Some("vsix".to_string());
            report.accepted_extensions = vec!["vsix".to_string()];
            report.evidence.push(EvidenceItem::new("zip.vsix.manifest"));
            return Ok(report);
        }
        if cd_check.has_nupkg_nuspec {
            report.identification = Identification::Format;
            report.format_id = Some("nupkg".to_string());
            report.category = "package".to_string();
            report.container_id = Some("zip".to_string());
            report.verification = Verification::Structure;
            report.inspection = Inspection::Complete;
            report.recommended_extension = Some("nupkg".to_string());
            report.accepted_extensions = vec!["nupkg".to_string()];
            report.evidence.push(EvidenceItem::new("zip.nupkg.nuspec"));
            return Ok(report);
        }
        if cd_check.has_xpi_manifest {
            report.identification = Identification::Format;
            report.format_id = Some("xpi".to_string());
            report.category = "package".to_string();
            report.container_id = Some("zip".to_string());
            report.verification = Verification::Structure;
            report.inspection = Inspection::Complete;
            report.recommended_extension = Some("xpi".to_string());
            report.accepted_extensions = vec!["xpi".to_string()];
            report.evidence.push(EvidenceItem::new("zip.xpi.manifest"));
            return Ok(report);
        }

        // 5. Plain ZIP archive
        report.identification = Identification::Format;
        report.format_id = Some("zip".to_string());
        report.category = "archive".to_string();
        report.container_id = Some("zip".to_string());
        report.recommended_extension = Some("zip".to_string());
        report.accepted_extensions = vec!["zip".to_string()];
        return Ok(report);
    }

    // Incomplete OOXML short-circuit: OOXML indicators exist but [Content_Types].xml is absent
    if !has_content_types && (has_rels || has_ooxml_subdirs) && !cd_check.has_mimetype {
        report.stats.entries_inspected = cd_check.cd_total_entries;
        report.evidence.push(EvidenceItem::new("zip.central_directory"));
        report.warnings.push(WarningItem::new(
            "ooxml.content_types_missing",
            "OOXML package structure incomplete: [Content_Types].xml missing",
        ));
        return Ok(report);
    }

    // Rewind back to start before calling ZipArchive
    if let Err(e) = reader.seek(SeekFrom::Start(0)) {
        if reader.is_budget_exceeded() {
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "budget.source_read_exceeded",
                "Total source read budget exceeded during detection",
            ));
            return Ok(report);
        }
        return Err(DetectError::Io(e));
    }

    let budget_handle = reader.budget_exceeded_handle();
    let (window_read, window_exceeded, window_active) =
        reader.start_window(options.max_zip_directory_window);

    let mut archive = match ZipArchive::new(&mut *reader) {
        Ok(a) => {
            window_active.store(false, std::sync::atomic::Ordering::Relaxed);
            let bytes_read = window_read.load(std::sync::atomic::Ordering::Relaxed);
            if window_exceeded.load(std::sync::atomic::Ordering::Relaxed)
                || bytes_read > options.max_zip_directory_window
            {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.zip_directory_window_exceeded",
                    format!(
                        "ZIP central directory read window exceeded budget limit ({})",
                        options.max_zip_directory_window
                    ),
                ));
                return Ok(report);
            }
            a
        }
        Err(e) => {
            window_active.store(false, std::sync::atomic::Ordering::Relaxed);
            if window_exceeded.load(std::sync::atomic::Ordering::Relaxed) {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.zip_directory_window_exceeded",
                    format!(
                        "ZIP central directory read window exceeded budget limit ({})",
                        options.max_zip_directory_window
                    ),
                ));
                return Ok(report);
            }
            if budget_handle.load(std::sync::atomic::Ordering::Relaxed) {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.source_read_exceeded",
                    "Total source read budget exceeded during detection",
                ));
                return Ok(report);
            }
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "zip.truncated_or_corrupted",
                format!("Failed to parse ZIP archive: {}", e),
            ));
            return Ok(report);
        }
    };

    report.stats.entries_inspected = archive.len();
    report.evidence.push(EvidenceItem::new("zip.central_directory"));

    // Check for OOXML indicators (already collected during central directory pre-check)
    let has_content_types = cd_check.has_content_types;
    let has_rels = cd_check.has_rels;
    let has_ooxml_subdirs = cd_check.has_ooxml_subdirs;

    // Pure ZIP confirmation: no OOXML characteristics
    if !has_content_types && !has_rels && !has_ooxml_subdirs {
        if check_epub_or_odf(&mut archive, &mut report) {
            return Ok(report);
        }
        if check_zip_packages(&mut archive, &mut report) {
            return Ok(report);
        }

        report.identification = Identification::Format;
        report.format_id = Some("zip".to_string());
        report.category = "archive".to_string();
        report.container_id = Some("zip".to_string());
        report.recommended_extension = Some("zip".to_string());
        report.accepted_extensions = vec!["zip".to_string()];
        return Ok(report);
    }

    // If OOXML indicators exist but [Content_Types].xml is absent -> incomplete OOXML
    if !has_content_types {
        report.warnings.push(WarningItem::new(
            "ooxml.content_types_missing",
            "OOXML package structure incomplete: [Content_Types].xml missing",
        ));
        return Ok(report);
    }

    // Parse _rels/.rels to locate main officeDocument part (streaming directly from compressed entry)
    let main_rel_target = {
        let rels_spec = XmlPartSpec {
            entry_name: "_rels/.rels",
            missing_code: "ooxml.required_relationship_missing",
            missing_msg: "Cannot open _rels/.rels entry in package",
            xml_limit_msg: "Decompressed size of _rels/.rels exceeded single XML limit",
        };
        let parsed = match stream_zip_xml_bounded(
            &mut archive,
            &rels_spec,
            options,
            &mut report,
            |r| parse_package_relationships(r),
        ) {
            Some(p) => p,
            None => return Ok(report),
        };
        match parsed {
            Some(target) => target,
            None => {
                report.warnings.push(WarningItem::new(
                    "ooxml.required_relationship_missing",
                    "No officeDocument core relationship found in _rels/.rels",
                ));
                return Ok(report);
            }
        }
    };

    report.evidence.push(EvidenceItem::new("ooxml.package_relationships"));

    let normalized_target = match resolve_target_path("", &main_rel_target) {
        Some(t) => t,
        None => {
            report.warnings.push(WarningItem::new(
                "ooxml.invalid_target_path",
                format!("Relationship target path '{}' resolves outside package root", main_rel_target),
            ));
            return Ok(report);
        }
    };

    // Parse [Content_Types].xml to verify target part content type (streaming)
    let content_type = {
        let ct_spec = XmlPartSpec {
            entry_name: "[Content_Types].xml",
            missing_code: "ooxml.content_types_missing",
            missing_msg: "Cannot open [Content_Types].xml entry",
            xml_limit_msg: "Decompressed size of [Content_Types].xml exceeded single XML limit",
        };
        let parsed = match stream_zip_xml_bounded(
            &mut archive,
            &ct_spec,
            options,
            &mut report,
            |r| parse_content_types(r, &normalized_target),
        ) {
            Some(p) => p,
            None => return Ok(report),
        };
        match parsed {
            Some(ct) => ct,
            None => {
                report.warnings.push(WarningItem::new(
                    "ooxml.content_type_not_found",
                    format!("No ContentType entry found for part '/{}'", normalized_target),
                ));
                return Ok(report);
            }
        }
    };

    report.evidence.push(EvidenceItem::new("ooxml.content_types"));

    // Classify supported OOXML family
    let format_kind = match classify_ooxml_content_type(&content_type) {
        Some(k) => k,
        None => {
            report.warnings.push(WarningItem::new(
                "ooxml.unsupported_content_type",
                format!("Package target has unsupported OOXML content type: {}", content_type),
            ));
            return Ok(report);
        }
    };

    // Stream-decompress and parse main part XML, verifying root tag and depth budget
    let main_spec = XmlPartSpec {
        entry_name: &normalized_target,
        missing_code: "ooxml.main_part_missing",
        missing_msg: "Failed to open main document part in package",
        xml_limit_msg: "Decompressed size of main part exceeded single XML limit",
    };
    let outcome = match stream_zip_xml_bounded(
        &mut archive,
        &main_spec,
        options,
        &mut report,
        |r| verify_main_part_xml(r, format_kind, options.max_xml_depth),
    ) {
        Some(o) => o,
        None => return Ok(report),
    };

    finalize_ooxml_outcome(outcome, options, &mut report);

    Ok(report)
}
