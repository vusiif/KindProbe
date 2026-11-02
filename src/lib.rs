pub mod budget;
pub mod detectors;
pub mod extension;
pub mod model;

use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::path::Path;
use std::time::Instant;

pub use budget::{BudgetReader, DetectOptions};
pub use extension::compare_extension;
pub use model::{
    DetectError, DetectionReport, DetectionStats, ErrorDetails, ErrorEnvelope, EvidenceItem,
    ExtensionStatus, Identification, Inspection, Verification, WarningItem,
};

/// Detects the format of a file at the given path.
///
/// Follows symlinks safely, verifies that the target is a regular file,
/// identifies the format via content evidence, and performs independent extension diagnosis.
pub fn detect_path(
    path: &Path,
    options: &DetectOptions,
) -> Result<DetectionReport, DetectError> {
    let start = Instant::now();

    let meta = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(e) => return Err(DetectError::Io(e)),
    };

    if !meta.is_file() {
        return Err(DetectError::NotRegularFile(path.to_string_lossy().to_string()));
    }

    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(DetectError::Io(e)),
    };

    let mut report = detect_reader(&mut file, options)?;

    // Perform independent extension diagnosis
    report.extension_status = compare_extension(&report, path.extension());
    report.stats.duration_micros = start.elapsed().as_micros() as u64;

    Ok(report)
}

/// Detects the format of an in-memory byte slice.
pub fn detect_bytes(
    bytes: &[u8],
    options: &DetectOptions,
) -> Result<DetectionReport, DetectError> {
    let start = Instant::now();
    let mut cursor = Cursor::new(bytes);
    let mut report = detect_reader(&mut cursor, options)?;
    report.stats.duration_micros = start.elapsed().as_micros() as u64;
    Ok(report)
}

/// Detects the format of a seekable reader using content-first structural analysis.
pub fn detect_reader<R: Read + Seek>(
    reader: &mut R,
    options: &DetectOptions,
) -> Result<DetectionReport, DetectError> {
    let start = Instant::now();
    let mut budget_reader = BudgetReader::new(reader, options.max_source_bytes_read);

    let mut report = detectors::detect_stream(&mut budget_reader, options)?;

    // Populate I/O stats
    budget_reader.populate_stats(&mut report.stats);

    // If budget limit was exhausted by the reader
    if budget_reader.is_budget_exceeded() {
        report.inspection = Inspection::Limited;
        if !report.warnings.iter().any(|w| w.code == "budget.source_read_exceeded") {
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
        return Err(DetectError::InvalidParameter(format!("Invariant violation: {}", inv)));
    }

    Ok(report)
}
