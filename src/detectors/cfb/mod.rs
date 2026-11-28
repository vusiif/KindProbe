pub mod doc;
pub mod ppt;
pub mod xls;

use cfb::CompoundFile;
use std::collections::HashSet;
use std::io::{self, Read, Seek, SeekFrom};

use crate::budget::{BudgetReader, DetectOptions};
use crate::model::{
    DetectError, DetectionReport, EvidenceItem, ExtensionStatus, Identification, Inspection,
    Verification, WarningItem,
};
pub use doc::verify_cfb_doc;
pub use ppt::verify_cfb_ppt;
pub use xls::verify_cfb_xls;

pub const CFB_MAGIC: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
const CFB_MAX_REG_SECTOR: u32 = 0xFFFFFFFA;

enum CfbPrecheckOutcome {
    Proceed,
    EntriesExceeded(usize),
    SourceBudgetExceeded,
}

fn read_cfb_sector<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    sector_id: u32,
    buf: &mut [u8],
) -> Result<bool, DetectError> {
    let sector_size = buf.len() as u64;
    let Some(offset) = (sector_id as u64)
        .checked_add(1)
        .and_then(|idx| idx.checked_mul(sector_size))
    else {
        return Ok(false);
    };
    if let Err(e) = reader.seek(SeekFrom::Start(offset)) {
        if reader.is_budget_exceeded() {
            return Ok(false);
        }
        return Err(DetectError::Io(e));
    }
    if let Err(e) = reader.read_exact(buf) {
        if reader.is_budget_exceeded()
            || e.kind() == io::ErrorKind::UnexpectedEof
            || e.kind() == io::ErrorKind::InvalidData
        {
            return Ok(false);
        }
        return Err(DetectError::Io(e));
    }
    Ok(true)
}

fn resolve_cfb_fat_sector<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    header: &[u8; 512],
    sector_size: usize,
    fat_idx: usize,
    difat_buf: &mut [u8],
) -> Result<Option<u32>, DetectError> {
    if fat_idx < 109 {
        let off = 76 + fat_idx * 4;
        return Ok(Some(u32::from_le_bytes([
            header[off],
            header[off + 1],
            header[off + 2],
            header[off + 3],
        ])));
    }

    let entries_per_difat = (sector_size / 4).saturating_sub(1);
    if entries_per_difat == 0 {
        return Ok(None);
    }
    let mut rem = fat_idx - 109;
    let mut difat_sector = u32::from_le_bytes([header[68], header[69], header[70], header[71]]);
    let num_difat = u32::from_le_bytes([header[72], header[73], header[74], header[75]]) as usize;
    let max_hops = num_difat.clamp(1, 256);

    for _ in 0..max_hops {
        if difat_sector > CFB_MAX_REG_SECTOR {
            return Ok(None);
        }
        if !read_cfb_sector(reader, difat_sector, difat_buf)? {
            return Ok(None);
        }
        if rem < entries_per_difat {
            let off = rem * 4;
            return Ok(Some(u32::from_le_bytes([
                difat_buf[off],
                difat_buf[off + 1],
                difat_buf[off + 2],
                difat_buf[off + 3],
            ])));
        }
        rem -= entries_per_difat;
        let next_off = entries_per_difat * 4;
        difat_sector = u32::from_le_bytes([
            difat_buf[next_off],
            difat_buf[next_off + 1],
            difat_buf[next_off + 2],
            difat_buf[next_off + 3],
        ]);
    }

    Ok(None)
}

/// Pre-validates MS-CFB directory entry count and sector chain capacity before `CompoundFile::open`
/// materializes the entire directory tree in memory.
fn precheck_cfb_directory_budget<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    max_entries: usize,
) -> Result<CfbPrecheckOutcome, DetectError> {
    if let Err(e) = reader.seek(SeekFrom::Start(0)) {
        if reader.is_budget_exceeded() {
            return Ok(CfbPrecheckOutcome::SourceBudgetExceeded);
        }
        return Err(DetectError::Io(e));
    }

    let mut header = [0u8; 512];
    if let Err(e) = reader.read_exact(&mut header) {
        if reader.is_budget_exceeded() {
            return Ok(CfbPrecheckOutcome::SourceBudgetExceeded);
        }
        if e.kind() == io::ErrorKind::UnexpectedEof {
            let _ = reader.seek(SeekFrom::Start(0));
            return Ok(CfbPrecheckOutcome::Proceed);
        }
        return Err(DetectError::Io(e));
    }

    let sector_shift = u16::from_le_bytes([header[30], header[31]]);
    let (sector_size, entries_per_sector) = match sector_shift {
        9 => (512usize, 4usize),
        12 => (4096usize, 32usize),
        _ => {
            reader.seek(SeekFrom::Start(0)).map_err(DetectError::Io)?;
            return Ok(CfbPrecheckOutcome::Proceed);
        }
    };

    let max_dir_sectors = max_entries
        .div_ceil(entries_per_sector)
        .saturating_mul(4)
        .max(4);
    let num_dir_sectors =
        u32::from_le_bytes([header[40], header[41], header[42], header[43]]) as usize;
    if sector_shift == 12 && num_dir_sectors > max_dir_sectors {
        return Ok(CfbPrecheckOutcome::EntriesExceeded(max_entries + 1));
    }

    let mut cur_dir_sector = u32::from_le_bytes([header[48], header[49], header[50], header[51]]);
    let mut active_entries = 0usize;
    let mut visited_dir_sectors = HashSet::new();
    let mut dir_buf = vec![0u8; sector_size];
    let mut fat_buf = vec![0u8; sector_size];
    let mut difat_buf = vec![0u8; sector_size];
    let mut cached_fat_sector: Option<u32> = None;
    let fat_entries_per_sector = sector_size / 4;

    for sector_step in 0..=max_dir_sectors {
        if cur_dir_sector > CFB_MAX_REG_SECTOR {
            break;
        }
        if sector_step == max_dir_sectors {
            return Ok(CfbPrecheckOutcome::EntriesExceeded(
                active_entries.max(max_entries + 1),
            ));
        }
        if !visited_dir_sectors.insert(cur_dir_sector) {
            break;
        }
        if !read_cfb_sector(reader, cur_dir_sector, &mut dir_buf)? {
            if reader.is_budget_exceeded() {
                return Ok(CfbPrecheckOutcome::SourceBudgetExceeded);
            }
            break;
        }

        for chunk in dir_buf.as_chunks::<128>().0 {
            // MS-CFB 2.6.1: Object Type at offset 66 (1=Storage, 2=Stream, 5=Root Storage)
            if matches!(chunk[66], 1 | 2 | 5) {
                active_entries += 1;
                if active_entries > max_entries {
                    return Ok(CfbPrecheckOutcome::EntriesExceeded(active_entries));
                }
            }
        }

        let fat_idx = (cur_dir_sector as usize) / fat_entries_per_sector;
        let slot_idx = (cur_dir_sector as usize) % fat_entries_per_sector;
        let Some(fat_sector_id) =
            resolve_cfb_fat_sector(reader, &header, sector_size, fat_idx, &mut difat_buf)?
        else {
            if reader.is_budget_exceeded() {
                return Ok(CfbPrecheckOutcome::SourceBudgetExceeded);
            }
            break;
        };
        if fat_sector_id > CFB_MAX_REG_SECTOR {
            break;
        }
        if cached_fat_sector != Some(fat_sector_id) {
            if !read_cfb_sector(reader, fat_sector_id, &mut fat_buf)? {
                if reader.is_budget_exceeded() {
                    return Ok(CfbPrecheckOutcome::SourceBudgetExceeded);
                }
                break;
            }
            cached_fat_sector = Some(fat_sector_id);
        }
        let off = slot_idx * 4;
        cur_dir_sector = u32::from_le_bytes([
            fat_buf[off],
            fat_buf[off + 1],
            fat_buf[off + 2],
            fat_buf[off + 3],
        ]);
    }

    if let Err(e) = reader.seek(SeekFrom::Start(0)) {
        if reader.is_budget_exceeded() {
            return Ok(CfbPrecheckOutcome::SourceBudgetExceeded);
        }
        return Err(DetectError::Io(e));
    }

    Ok(CfbPrecheckOutcome::Proceed)
}

/// Detects CFB compound file structure and specific formats (DOC, XLS, PPT).
pub fn detect_cfb<R: Read + Seek>(
    reader: &mut BudgetReader<R>,
    options: &DetectOptions,
) -> Result<DetectionReport, DetectError> {
    let mut report = DetectionReport {
        schema_version: 1,
        format_id: None,
        category: "container".to_string(),
        container_id: Some("cfb".to_string()),
        identification: Identification::ContainerOnly,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: None,
        accepted_extensions: Vec::new(),
        extension_status: ExtensionStatus::Undetermined,
        evidence: vec![EvidenceItem::new("cfb.header")],
        warnings: Vec::new(),
        stats: Default::default(),
        candidates: None,
    };

    match precheck_cfb_directory_budget(reader, options.max_entries)? {
        CfbPrecheckOutcome::Proceed => {}
        CfbPrecheckOutcome::EntriesExceeded(entries_count) => {
            report.stats.entries_inspected = entries_count;
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "budget.entries_exceeded",
                format!(
                    "CFB entry count ({}) exceeded budget limit ({})",
                    entries_count, options.max_entries
                ),
            ));
            return Ok(report);
        }
        CfbPrecheckOutcome::SourceBudgetExceeded => {
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "budget.source_read_exceeded",
                "Total source read budget exceeded during detection",
            ));
            return Ok(report);
        }
    }

    let budget_handle = reader.budget_exceeded_handle();
    let mut comp = match CompoundFile::open(&mut *reader) {
        Ok(c) => c,
        Err(e) => {
            if budget_handle.load(std::sync::atomic::Ordering::Relaxed) {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "budget.source_read_exceeded",
                    "Total source read budget exceeded during detection",
                ));
                return Ok(report);
            }
            if e.kind() == io::ErrorKind::InvalidData || e.kind() == io::ErrorKind::UnexpectedEof {
                report.inspection = Inspection::Limited;
                report.warnings.push(WarningItem::new(
                    "cfb.corrupted_or_invalid_chain",
                    format!("Failed to open compound document: {}", e),
                ));
                return Ok(report);
            }
            return Err(DetectError::Io(e));
        }
    };

    report.evidence.push(EvidenceItem::new("cfb.directory"));

    // Check CFB entries budget and discover root streams in a single walk pass
    let mut entries_count = 0;
    let mut word_doc = None::<(std::borrow::Cow<'static, std::path::Path>, u64)>;
    let mut xls_doc = None::<(std::borrow::Cow<'static, std::path::Path>, u64)>;
    let mut ppt_doc = None::<(std::borrow::Cow<'static, std::path::Path>, u64)>;
    let mut table_0 = None::<std::borrow::Cow<'static, std::path::Path>>;
    let mut table_1 = None::<std::borrow::Cow<'static, std::path::Path>>;
    let mut has_current_user = false;

    for entry in comp.walk() {
        entries_count += 1;
        if entries_count > options.max_entries {
            report.stats.entries_inspected = entries_count;
            report.inspection = Inspection::Limited;
            report.warnings.push(WarningItem::new(
                "budget.entries_exceeded",
                format!(
                    "CFB entry count ({}) exceeded budget limit ({})",
                    entries_count, options.max_entries
                ),
            ));
            return Ok(report);
        }

        if entry.is_stream() && entry.path().parent() == Some(std::path::Path::new("/")) {
            let name = entry.name();
            if name == "WordDocument" {
                word_doc = Some((
                    std::borrow::Cow::Borrowed(std::path::Path::new("/WordDocument")),
                    entry.len(),
                ));
            } else if name.eq_ignore_ascii_case("WordDocument") {
                word_doc = Some((
                    std::borrow::Cow::Owned(entry.path().to_path_buf()),
                    entry.len(),
                ));
            } else if name == "Workbook" {
                xls_doc = Some((
                    std::borrow::Cow::Borrowed(std::path::Path::new("/Workbook")),
                    entry.len(),
                ));
            } else if name == "Book" {
                xls_doc = Some((
                    std::borrow::Cow::Borrowed(std::path::Path::new("/Book")),
                    entry.len(),
                ));
            } else if name.eq_ignore_ascii_case("Workbook") || name.eq_ignore_ascii_case("Book") {
                xls_doc = Some((
                    std::borrow::Cow::Owned(entry.path().to_path_buf()),
                    entry.len(),
                ));
            } else if name == "PowerPoint Document" {
                ppt_doc = Some((
                    std::borrow::Cow::Borrowed(std::path::Path::new("/PowerPoint Document")),
                    entry.len(),
                ));
            } else if name.eq_ignore_ascii_case("PowerPoint Document") {
                ppt_doc = Some((
                    std::borrow::Cow::Owned(entry.path().to_path_buf()),
                    entry.len(),
                ));
            } else if name.eq_ignore_ascii_case("Current User") {
                has_current_user = true;
            } else if name == "0Table" {
                table_0 = Some(std::borrow::Cow::Borrowed(std::path::Path::new("/0Table")));
            } else if name.eq_ignore_ascii_case("0Table") {
                table_0 = Some(std::borrow::Cow::Owned(entry.path().to_path_buf()));
            } else if name == "1Table" {
                table_1 = Some(std::borrow::Cow::Borrowed(std::path::Path::new("/1Table")));
            } else if name.eq_ignore_ascii_case("1Table") {
                table_1 = Some(std::borrow::Cow::Owned(entry.path().to_path_buf()));
            }
        }
    }
    report.stats.entries_inspected = entries_count;

    // 1. Try Word Document verification if WordDocument stream exists
    if let Some((path, len)) = word_doc {
        if verify_cfb_doc(
            &mut comp,
            &path,
            len,
            table_0.as_deref(),
            table_1.as_deref(),
            &budget_handle,
            &mut report,
        )? {
            return Ok(report);
        }
    }

    // 2. Try Excel Workbook verification if Workbook/Book stream exists
    if let Some((path, len)) = xls_doc {
        if verify_cfb_xls(&mut comp, &path, len, &budget_handle, &mut report)? {
            return Ok(report);
        }
    }

    // 3. Try PowerPoint verification if PowerPoint Document stream exists
    if let Some((path, len)) = ppt_doc {
        if verify_cfb_ppt(
            &mut comp,
            &path,
            len,
            has_current_user,
            &budget_handle,
            &mut report,
        )? {
            return Ok(report);
        }
    }

    // If no specific office format confirmed, report remains ContainerOnly (CFB)
    Ok(report)
}

/// Backwards compatible alias for detect_cfb
pub use detect_cfb as detect_cfb_doc;
