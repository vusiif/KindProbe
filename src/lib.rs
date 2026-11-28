#![forbid(unsafe_code)]

pub mod budget;
pub mod detectors;
pub mod extension;
pub mod model;

use std::ffi::OsStr;
use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::path::Path;
use std::time::Instant;

pub use budget::{BudgetReader, DetectOptions};
pub use extension::{compare_extension, compare_extension_core};
pub use model::{
    DetectError, DetectionCore, DetectionReport, DetectionStats, ErrorDetails, ErrorEnvelope,
    EvidenceItem, ExtensionStatus, Identification, Inspection, Verification, WarningItem,
};

fn resolve_path_extension(path: &Path) -> Option<&OsStr> {
    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
        let b = file_name.as_bytes();
        if b.len() >= 7 && b[b.len() - 7..].eq_ignore_ascii_case(b".tar.gz") {
            return Some(OsStr::new("tar.gz"));
        } else if b.len() >= 8 && b[b.len() - 8..].eq_ignore_ascii_case(b".tar.bz2") {
            return Some(OsStr::new("tar.bz2"));
        } else if b.len() >= 7 && b[b.len() - 7..].eq_ignore_ascii_case(b".tar.xz") {
            return Some(OsStr::new("tar.xz"));
        } else if b.len() >= 8 && b[b.len() - 8..].eq_ignore_ascii_case(b".tar.zst") {
            return Some(OsStr::new("tar.zst"));
        }
    }
    path.extension()
}

/// Detects the format of a file at the given path.
///
/// Follows symlinks safely, verifies that the target is a regular file,
/// identifies the format via content evidence, and performs independent extension diagnosis.
pub fn detect_path(path: &Path, options: &DetectOptions) -> Result<DetectionReport, DetectError> {
    let start = Instant::now();

    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            // On platforms like Windows, opening a directory returns PermissionDenied.
            // Check if the path actually exists and is a directory / non-regular file.
            if let Ok(meta) = path.metadata() {
                if !meta.is_file() {
                    return Err(DetectError::NotRegularFile(
                        path.to_string_lossy().to_string(),
                    ));
                }
            }
            return Err(DetectError::Io(e));
        }
    };

    let meta = match file.metadata() {
        Ok(m) => m,
        Err(e) => return Err(DetectError::Io(e)),
    };

    if !meta.is_file() {
        return Err(DetectError::NotRegularFile(
            path.to_string_lossy().to_string(),
        ));
    }

    let path_ext = resolve_path_extension(path);
    let mut local_opts = options.clone();
    if local_opts.extension_hint.is_none() {
        local_opts.extension_hint = path_ext.and_then(|s| s.to_str()).map(|s| s.to_string());
    }

    let mut report = detect_reader(&mut file, &local_opts)?;

    // Perform independent extension diagnosis
    report.extension_status = compare_extension(&report, path_ext);
    report.stats.duration_micros = start.elapsed().as_micros() as u64;

    Ok(report)
}

/// Detects the format of an in-memory byte slice returning a lightweight, zero-allocation DetectionCore.
pub fn detect_bytes_core(
    bytes: &[u8],
    options: &DetectOptions,
) -> Result<DetectionCore, DetectError> {
    if options.max_source_bytes_read < options.initial_window_size
        && options.max_source_bytes_read < bytes.len()
    {
        let core = DetectionCore::budget_exceeded();
        if let Err(inv) = core.validate_invariants() {
            return Err(DetectError::InvalidParameter(format!(
                "Invariant violation: {}",
                inv
            )));
        }
        return Ok(core);
    }

    match detectors::detect_bytes_non_container_core(bytes, options)? {
        Some(core) => {
            if let Err(inv) = core.validate_invariants() {
                return Err(DetectError::InvalidParameter(format!(
                    "Invariant violation: {}",
                    inv
                )));
            }
            Ok(core)
        }
        None => {
            let mut cursor = Cursor::new(bytes);
            detect_reader_core(&mut cursor, options)
        }
    }
}

/// Detects the format of an in-memory byte slice.
pub fn detect_bytes(bytes: &[u8], options: &DetectOptions) -> Result<DetectionReport, DetectError> {
    let start = Instant::now();

    // Check read budget consistency with detect_reader:
    // If the read budget is exhausted before the initial window can be read from non-empty input
    if options.max_source_bytes_read < options.initial_window_size
        && options.max_source_bytes_read < bytes.len()
    {
        let core = DetectionCore::budget_exceeded();
        let mut report = core.to_report(
            DetectionStats {
                source_bytes_read: options.max_source_bytes_read,
                seek_count: 0,
                entries_inspected: 0,
                decompressed_bytes: 0,
                duration_micros: start.elapsed().as_micros() as u64,
            },
            ExtensionStatus::Undetermined,
        );
        report.extension_status = compare_extension(&report, None);
        if let Err(inv) = report.validate_invariants() {
            return Err(DetectError::InvalidParameter(format!(
                "Invariant violation: {}",
                inv
            )));
        }
        return Ok(report);
    }

    let mut report = match detectors::detect_bytes_non_container(bytes, options)? {
        Some(rep) => rep,
        None => {
            let mut cursor = Cursor::new(bytes);
            let mut budget_reader = BudgetReader::new(&mut cursor, options.max_source_bytes_read)
                .with_max_seeks(options.max_entries.saturating_mul(8).max(4096));
            let mut rep = detectors::detect_stream(&mut budget_reader, options)?;
            budget_reader.populate_stats(&mut rep.stats);
            if budget_reader.is_budget_exceeded() {
                rep.inspection = Inspection::Limited;
                if !rep
                    .warnings
                    .iter()
                    .any(|w| w.code == "budget.source_read_exceeded")
                {
                    rep.warnings.push(WarningItem::new(
                        "budget.source_read_exceeded",
                        "Total source read budget exceeded during detection",
                    ));
                }
            }
            rep
        }
    };

    report.stats.duration_micros = start.elapsed().as_micros() as u64;

    // Independent extension diagnosis: readers and byte slices have no extension (None)
    report.extension_status = compare_extension(&report, None);

    // Verify invariants
    if let Err(inv) = report.validate_invariants() {
        return Err(DetectError::InvalidParameter(format!(
            "Invariant violation: {}",
            inv
        )));
    }

    Ok(report)
}

/// Lightweight detection for a seekable reader returning zero-allocation DetectionCore.
pub fn detect_reader_core<R: Read + Seek>(
    reader: &mut R,
    options: &DetectOptions,
) -> Result<DetectionCore, DetectError> {
    let mut budget_reader = BudgetReader::new(reader, options.max_source_bytes_read)
        .with_max_seeks(options.max_entries.saturating_mul(8).max(4096));
    let mut core = detectors::detect_stream_core(&mut budget_reader, options)?;

    if budget_reader.is_budget_exceeded() {
        core.inspection = Inspection::Limited;
        if core.warning.is_none() {
            core.warning = Some((
                "budget.source_read_exceeded",
                "Total source read budget exceeded during detection",
            ));
        }
    }

    if let Err(inv) = core.validate_invariants() {
        return Err(DetectError::InvalidParameter(format!(
            "Invariant violation: {}",
            inv
        )));
    }
    Ok(core)
}

/// Lightweight detection for a file path returning zero-allocation DetectionCore.
pub fn detect_path_core(
    path: &Path,
    options: &DetectOptions,
) -> Result<DetectionCore, DetectError> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            if let Ok(meta) = path.metadata() {
                if !meta.is_file() {
                    return Err(DetectError::NotRegularFile(
                        path.to_string_lossy().to_string(),
                    ));
                }
            }
            return Err(DetectError::Io(e));
        }
    };

    let meta = file.metadata().map_err(DetectError::Io)?;
    if !meta.is_file() {
        return Err(DetectError::NotRegularFile(
            path.to_string_lossy().to_string(),
        ));
    }

    if options.extension_hint.is_none() {
        if let Some(ext) = resolve_path_extension(path).and_then(|s| s.to_str()) {
            let mut local_opts = options.clone();
            local_opts.extension_hint = Some(ext.to_string());
            return detect_reader_core(&mut file, &local_opts);
        }
    }

    detect_reader_core(&mut file, options)
}

/// Detects the format of a seekable reader using content-first structural analysis.
pub fn detect_reader<R: Read + Seek>(
    reader: &mut R,
    options: &DetectOptions,
) -> Result<DetectionReport, DetectError> {
    let start = Instant::now();
    let mut budget_reader = BudgetReader::new(reader, options.max_source_bytes_read)
        .with_max_seeks(options.max_entries.saturating_mul(8).max(4096));

    let mut report = detectors::detect_stream(&mut budget_reader, options)?;

    // Populate I/O stats
    budget_reader.populate_stats(&mut report.stats);

    // If budget limit was exhausted by the reader
    if budget_reader.is_budget_exceeded() {
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

    report.stats.duration_micros = start.elapsed().as_micros() as u64;

    // Independent extension diagnosis: readers and byte slices have no extension (None)
    report.extension_status = compare_extension(&report, None);

    // Verify invariants
    if let Err(inv) = report.validate_invariants() {
        return Err(DetectError::InvalidParameter(format!(
            "Invariant violation: {}",
            inv
        )));
    }

    Ok(report)
}
