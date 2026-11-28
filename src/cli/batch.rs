use std::collections::{BTreeMap, VecDeque};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Condvar, Mutex};
use std::time::Instant;

use kindprobe::budget::DetectOptions;
use kindprobe::model::DetectError;
use kindprobe::{detect_path_core, DetectionCore, Identification};

use super::i18n::{format_category_cn, AppLanguage};

#[derive(serde::Serialize)]
struct BatchResultJson<'a> {
    path: &'a str,
    status: &'static str,
    format_id: Option<&'static str>,
    category: &'static str,
    container_id: Option<&'static str>,
    identification: Identification,
    recommended_extension: Option<&'static str>,
}

#[derive(serde::Serialize)]
struct BatchErrorJson<'a> {
    path: &'a str,
    status: &'static str,
    error_code: &'static str,
    message: String,
}

pub(crate) fn error_code_for_detect_error(err: &DetectError) -> &'static str {
    match err {
        DetectError::NotRegularFile(_) => "io.not_regular_file",
        DetectError::InvalidParameter(_) => "arg.invalid_parameter",
        DetectError::Io(io_err) => match io_err.kind() {
            std::io::ErrorKind::NotFound => "io.not_found",
            std::io::ErrorKind::PermissionDenied => "io.permission_denied",
            _ => "io.read_error",
        },
    }
}

fn error_code_for_io_error(err: &io::Error) -> &'static str {
    match err.kind() {
        io::ErrorKind::NotFound => "io.not_found",
        io::ErrorKind::PermissionDenied => "io.permission_denied",
        _ => "io.read_error",
    }
}

fn format_detect_result(
    path: &Path,
    result: Result<DetectionCore, DetectError>,
    jsonl_mode: bool,
    lang: AppLanguage,
    out_buf: &mut Vec<u8>,
    err_buf: &mut Vec<u8>,
) -> io::Result<(usize, usize, usize)> {
    let path_str = path.to_string_lossy();
    match result {
        Ok(core) => {
            if jsonl_mode {
                serde_json::to_writer(
                    &mut *out_buf,
                    &BatchResultJson {
                        path: &path_str,
                        status: "ok",
                        format_id: core.format_id,
                        category: core.category,
                        container_id: core.container_id,
                        identification: core.identification,
                        recommended_extension: core.recommended_extension,
                    },
                )
                .map_err(io::Error::other)?;
                out_buf.write_all(b"\n")?;
            } else {
                let fmt = core.format_id.unwrap_or("unknown");
                match lang {
                    AppLanguage::Chinese => writeln!(
                        out_buf,
                        "{} -> {} ({})",
                        path_str,
                        fmt,
                        format_category_cn(core.category)
                    )?,
                    AppLanguage::English => {
                        writeln!(out_buf, "{} -> {} ({})", path_str, fmt, core.category)?;
                    }
                }
            }
            if core.identification == Identification::Format {
                Ok((1, 0, 0))
            } else {
                Ok((0, 1, 0))
            }
        }
        Err(err) => {
            let code = error_code_for_detect_error(&err);
            format_error_record(
                &path_str,
                code,
                &err.to_string(),
                jsonl_mode,
                lang,
                out_buf,
                err_buf,
            )?;
            Ok((0, 0, 1))
        }
    }
}

fn format_error_record(
    path_str: &str,
    code: &'static str,
    message: &str,
    jsonl_mode: bool,
    lang: AppLanguage,
    out_buf: &mut Vec<u8>,
    err_buf: &mut Vec<u8>,
) -> io::Result<()> {
    if jsonl_mode {
        serde_json::to_writer(
            &mut *out_buf,
            &BatchErrorJson {
                path: path_str,
                status: "error",
                error_code: code,
                message: message.to_string(),
            },
        )
        .map_err(io::Error::other)?;
        out_buf.write_all(b"\n")?;
    } else {
        match lang {
            AppLanguage::Chinese => {
                writeln!(err_buf, "错误 [{}]: {} ({})", code, path_str, message)?
            }
            AppLanguage::English => {
                writeln!(err_buf, "Error [{}]: {} ({})", code, path_str, message)?
            }
        }
    }
    Ok(())
}

struct QueueState<T> {
    items: VecDeque<T>,
    closed: bool,
    capacity: usize,
}

struct BoundedWorkQueue<T> {
    state: Mutex<QueueState<T>>,
    not_empty: Condvar,
    not_full: Condvar,
}

impl<T> BoundedWorkQueue<T> {
    fn new(capacity: usize) -> Self {
        Self {
            state: Mutex::new(QueueState {
                items: VecDeque::with_capacity(capacity.max(1)),
                closed: false,
                capacity: capacity.max(1),
            }),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
        }
    }

    fn push(&self, item: T) -> bool {
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        while guard.items.len() >= guard.capacity && !guard.closed {
            guard = self.not_full.wait(guard).unwrap_or_else(|e| e.into_inner());
        }
        if guard.closed {
            return false;
        }
        guard.items.push_back(item);
        self.not_empty.notify_one();
        true
    }

    fn pop(&self) -> Option<T> {
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        while guard.items.is_empty() && !guard.closed {
            guard = self
                .not_empty
                .wait(guard)
                .unwrap_or_else(|e| e.into_inner());
        }
        let item = guard.items.pop_front();
        if item.is_some() {
            self.not_full.notify_one();
        }
        item
    }

    fn close(&self) {
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        guard.closed = true;
        self.not_empty.notify_all();
        self.not_full.notify_all();
    }
}

struct WindowState {
    in_flight: usize,
    max_in_flight: usize,
    closed: bool,
}

struct InFlightWindow {
    state: Mutex<WindowState>,
    available: Condvar,
}

impl InFlightWindow {
    fn new(max_in_flight: usize) -> Self {
        Self {
            state: Mutex::new(WindowState {
                in_flight: 0,
                max_in_flight: max_in_flight.max(1),
                closed: false,
            }),
            available: Condvar::new(),
        }
    }

    fn acquire(&self) -> bool {
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        while guard.in_flight >= guard.max_in_flight && !guard.closed {
            guard = self
                .available
                .wait(guard)
                .unwrap_or_else(|e| e.into_inner());
        }
        if guard.closed {
            return false;
        }
        guard.in_flight += 1;
        true
    }

    fn release(&self) {
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        guard.in_flight = guard.in_flight.saturating_sub(1);
        self.available.notify_one();
    }

    fn close(&self) {
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        guard.closed = true;
        self.available.notify_all();
    }
}

struct BatchTask {
    seq: usize,
    path: PathBuf,
}

struct ProcessedItem {
    seq: usize,
    id_inc: usize,
    unk_inc: usize,
    err_inc: usize,
    out_bytes: Vec<u8>,
    err_bytes: Vec<u8>,
}

enum WalkEvent {
    File(PathBuf),
    Error {
        path: PathBuf,
        code: &'static str,
        message: String,
    },
}

fn next_walk_event(
    work_stack: &mut Vec<(PathBuf, bool)>,
    pending_events: &mut VecDeque<WalkEvent>,
    recursive: bool,
) -> Option<WalkEvent> {
    loop {
        if let Some(ev) = pending_events.pop_front() {
            return Some(ev);
        }

        let (path, is_root) = work_stack.pop()?;
        let meta = match std::fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                return Some(WalkEvent::Error {
                    code: error_code_for_io_error(&e),
                    message: e.to_string(),
                    path,
                });
            }
        };

        let is_symlink_dir = meta.file_type().is_symlink() && path.is_dir();
        if is_symlink_dir {
            continue;
        }

        if meta.is_dir() && (recursive || is_root) {
            match std::fs::read_dir(&path) {
                Ok(rd) => {
                    let mut children: Vec<PathBuf> = Vec::new();
                    let mut dir_errors: Vec<WalkEvent> = Vec::new();
                    for res in rd {
                        match res {
                            Ok(entry) => {
                                let child_path = entry.path();
                                match entry.file_type() {
                                    Ok(ft) => {
                                        // Skip symlinked directories to avoid traversal cycles
                                        if ft.is_symlink() && child_path.is_dir() {
                                            continue;
                                        }
                                        if !recursive && ft.is_dir() {
                                            continue;
                                        }
                                        children.push(child_path);
                                    }
                                    Err(e) => {
                                        dir_errors.push(WalkEvent::Error {
                                            code: error_code_for_io_error(&e),
                                            message: e.to_string(),
                                            path: child_path,
                                        });
                                    }
                                }
                            }
                            Err(e) => {
                                dir_errors.push(WalkEvent::Error {
                                    code: error_code_for_io_error(&e),
                                    message: e.to_string(),
                                    path: path.clone(),
                                });
                            }
                        }
                    }
                    children.sort();
                    for child in children.into_iter().rev() {
                        work_stack.push((child, false));
                    }
                    for err_ev in dir_errors {
                        pending_events.push_back(err_ev);
                    }
                }
                Err(e) => {
                    return Some(WalkEvent::Error {
                        code: error_code_for_io_error(&e),
                        message: e.to_string(),
                        path,
                    });
                }
            }
            continue;
        }

        return Some(WalkEvent::File(path));
    }
}

fn report_write_error(err: &io::Error, lang: AppLanguage, err_out: &mut impl Write) {
    let _ = match lang {
        AppLanguage::Chinese => {
            writeln!(err_out, "错误 [io.write_error]: 写入批量输出失败 ({})", err)
        }
        AppLanguage::English => writeln!(
            err_out,
            "Error [io.write_error]: failed to write batch output ({})",
            err
        ),
    };
}

fn emit_processed_item(
    item: ProcessedItem,
    out: &mut impl Write,
    err_out: &mut impl Write,
    identified: &mut usize,
    unknown: &mut usize,
    errors: &mut usize,
    total_files: &mut usize,
) -> io::Result<()> {
    *total_files += 1;
    *identified += item.id_inc;
    *unknown += item.unk_inc;
    *errors += item.err_inc;

    if !item.out_bytes.is_empty() {
        out.write_all(&item.out_bytes)?;
    }
    if !item.err_bytes.is_empty() {
        err_out.write_all(&item.err_bytes)?;
    }
    Ok(())
}

pub(crate) fn run_batch(
    paths: &[PathBuf],
    recursive: bool,
    jsonl_mode: bool,
    jobs: Option<usize>,
    lang: AppLanguage,
) -> i32 {
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let out = BufWriter::with_capacity(64 * 1024, stdout.lock());
    let mut err_out = stderr.lock();
    run_batch_with_io(
        paths,
        recursive,
        jsonl_mode,
        jobs,
        lang,
        out,
        &mut err_out,
        detect_path_core,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_batch_with_io<F, WOut, WErr>(
    paths: &[PathBuf],
    recursive: bool,
    jsonl_mode: bool,
    jobs: Option<usize>,
    lang: AppLanguage,
    mut out: WOut,
    err_out: &mut WErr,
    detector: F,
) -> i32
where
    F: Fn(&Path, &DetectOptions) -> Result<DetectionCore, DetectError> + Sync,
    WOut: Write,
    WErr: Write,
{
    let start = Instant::now();
    let options = DetectOptions::default();

    let mut total_files: usize = 0;
    let mut identified: usize = 0;
    let mut unknown: usize = 0;
    let mut errors: usize = 0;
    let mut write_failed = false;

    let default_jobs = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .clamp(2, 8);
    let requested_jobs = jobs.unwrap_or(default_jobs).clamp(1, 16);
    let single_regular_file_fast = paths.len() == 1 && !paths[0].is_dir();

    if requested_jobs <= 1 || single_regular_file_fast {
        let mut work_stack: Vec<(PathBuf, bool)> =
            paths.iter().rev().map(|p| (p.clone(), true)).collect();
        let mut pending_events = VecDeque::new();
        let mut line_out = Vec::with_capacity(256);
        let mut line_err = Vec::with_capacity(256);

        while let Some(ev) = next_walk_event(&mut work_stack, &mut pending_events, recursive) {
            line_out.clear();
            line_err.clear();
            let (id_inc, unk_inc, err_inc, format_res) = match ev {
                WalkEvent::File(path) => {
                    let res = detector(&path, &options);
                    match format_detect_result(
                        &path,
                        res,
                        jsonl_mode,
                        lang,
                        &mut line_out,
                        &mut line_err,
                    ) {
                        Ok((i, u, e)) => (i, u, e, Ok(())),
                        Err(e) => (0, 0, 1, Err(e)),
                    }
                }
                WalkEvent::Error {
                    path,
                    code,
                    message,
                } => {
                    let path_str = path.to_string_lossy();
                    let fmt_res = format_error_record(
                        &path_str,
                        code,
                        &message,
                        jsonl_mode,
                        lang,
                        &mut line_out,
                        &mut line_err,
                    );
                    (0, 0, 1, fmt_res)
                }
            };

            if let Err(e) = format_res {
                report_write_error(&e, lang, err_out);
                write_failed = true;
                break;
            }

            let item = ProcessedItem {
                seq: total_files,
                id_inc,
                unk_inc,
                err_inc,
                out_bytes: std::mem::take(&mut line_out),
                err_bytes: std::mem::take(&mut line_err),
            };
            if let Err(e) = emit_processed_item(
                item,
                &mut out,
                err_out,
                &mut identified,
                &mut unknown,
                &mut errors,
                &mut total_files,
            ) {
                report_write_error(&e, lang, err_out);
                write_failed = true;
                break;
            }
            if jsonl_mode && total_files == 1 {
                if let Err(e) = out.flush() {
                    report_write_error(&e, lang, err_out);
                    write_failed = true;
                    break;
                }
            }
        }
    } else {
        let num_threads = requested_jobs;
        let max_in_flight = (num_threads * 8).max(32);
        let work_queue = BoundedWorkQueue::<BatchTask>::new((num_threads * 4).max(16));
        let in_flight = InFlightWindow::new(max_in_flight);
        let (result_tx, result_rx) = mpsc::channel::<ProcessedItem>();

        let work_queue_ref = &work_queue;
        let in_flight_ref = &in_flight;
        let options_ref = &options;
        let detector_ref = &detector;

        let worker_panics = std::thread::scope(|s| {
            let producer_tx = result_tx.clone();
            let producer_handle = s.spawn(move || {
                let mut work_stack: Vec<(PathBuf, bool)> =
                    paths.iter().rev().map(|p| (p.clone(), true)).collect();
                let mut pending_events = VecDeque::new();
                let mut next_seq = 0usize;

                while let Some(ev) =
                    next_walk_event(&mut work_stack, &mut pending_events, recursive)
                {
                    if !in_flight_ref.acquire() {
                        break;
                    }
                    let seq = next_seq;
                    next_seq += 1;
                    match ev {
                        WalkEvent::File(path) => {
                            if !work_queue_ref.push(BatchTask { seq, path }) {
                                in_flight_ref.release();
                                break;
                            }
                        }
                        WalkEvent::Error {
                            path,
                            code,
                            message,
                        } => {
                            let mut out_bytes = Vec::new();
                            let mut err_bytes = Vec::new();
                            let path_str = path.to_string_lossy();
                            let _ = format_error_record(
                                &path_str,
                                code,
                                &message,
                                jsonl_mode,
                                lang,
                                &mut out_bytes,
                                &mut err_bytes,
                            );
                            if producer_tx
                                .send(ProcessedItem {
                                    seq,
                                    id_inc: 0,
                                    unk_inc: 0,
                                    err_inc: 1,
                                    out_bytes,
                                    err_bytes,
                                })
                                .is_err()
                            {
                                in_flight_ref.release();
                                break;
                            }
                        }
                    }
                }
                work_queue_ref.close();
            });

            let mut worker_handles = Vec::with_capacity(num_threads);
            for _ in 0..num_threads {
                let worker_tx = result_tx.clone();
                worker_handles.push(s.spawn(move || {
                    while let Some(task) = work_queue_ref.pop() {
                        let mut out_bytes = Vec::with_capacity(160);
                        let mut err_bytes = Vec::new();
                        let res = detector_ref(&task.path, options_ref);
                        let (id_inc, unk_inc, err_inc) = match format_detect_result(
                            &task.path,
                            res,
                            jsonl_mode,
                            lang,
                            &mut out_bytes,
                            &mut err_bytes,
                        ) {
                            Ok(counts) => counts,
                            Err(e) => {
                                let _ = writeln!(err_bytes, "Error [io.write_error]: {}", e);
                                (0, 0, 1)
                            }
                        };
                        if worker_tx
                            .send(ProcessedItem {
                                seq: task.seq,
                                id_inc,
                                unk_inc,
                                err_inc,
                                out_bytes,
                                err_bytes,
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                }));
            }
            drop(result_tx);

            let mut next_expected_seq = 0usize;
            let mut pending: BTreeMap<usize, ProcessedItem> = BTreeMap::new();
            let mut flushed_first = false;

            while let Ok(item) = result_rx.recv() {
                pending.insert(item.seq, item);
                while let Some(ready) = pending.remove(&next_expected_seq) {
                    in_flight_ref.release();
                    if let Err(e) = emit_processed_item(
                        ready,
                        &mut out,
                        err_out,
                        &mut identified,
                        &mut unknown,
                        &mut errors,
                        &mut total_files,
                    ) {
                        report_write_error(&e, lang, err_out);
                        write_failed = true;
                        work_queue_ref.close();
                        in_flight_ref.close();
                        break;
                    }
                    next_expected_seq += 1;
                }

                if write_failed {
                    break;
                }

                // Drain any already-queued items before flushing so live consumers see the first
                // line immediately without starving under a burst.
                while let Ok(more) = result_rx.try_recv() {
                    pending.insert(more.seq, more);
                    while let Some(ready) = pending.remove(&next_expected_seq) {
                        in_flight_ref.release();
                        if let Err(e) = emit_processed_item(
                            ready,
                            &mut out,
                            err_out,
                            &mut identified,
                            &mut unknown,
                            &mut errors,
                            &mut total_files,
                        ) {
                            report_write_error(&e, lang, err_out);
                            write_failed = true;
                            work_queue_ref.close();
                            in_flight_ref.close();
                            break;
                        }
                        next_expected_seq += 1;
                    }
                    if write_failed {
                        break;
                    }
                }
                if write_failed {
                    break;
                }

                if jsonl_mode && (!flushed_first || next_expected_seq > 0) {
                    if !flushed_first && next_expected_seq > 0 {
                        flushed_first = true;
                        if let Err(e) = out.flush() {
                            report_write_error(&e, lang, err_out);
                            write_failed = true;
                            work_queue_ref.close();
                            in_flight_ref.close();
                            break;
                        }
                    } else if next_expected_seq > 0 && pending.is_empty() {
                        if let Err(e) = out.flush() {
                            report_write_error(&e, lang, err_out);
                            write_failed = true;
                            work_queue_ref.close();
                            in_flight_ref.close();
                            break;
                        }
                    }
                }
            }

            if write_failed {
                drop(result_rx);
            }

            let mut panics = 0usize;
            if producer_handle.join().is_err() {
                panics += 1;
            }
            for handle in worker_handles {
                if handle.join().is_err() {
                    panics += 1;
                }
            }
            panics
        });

        if worker_panics > 0 {
            errors += worker_panics;
            let _ = match lang {
                AppLanguage::Chinese => writeln!(
                    err_out,
                    "错误 [batch.worker_panic]: {} 个批处理工作线程异常终止",
                    worker_panics
                ),
                AppLanguage::English => writeln!(
                    err_out,
                    "Error [batch.worker_panic]: {} batch worker thread(s) panicked",
                    worker_panics
                ),
            };
        }
    }

    if !write_failed {
        if let Err(e) = out.flush() {
            report_write_error(&e, lang, err_out);
            write_failed = true;
        }
    }

    if !jsonl_mode {
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        let rate = if elapsed_ms > 0.0 {
            (total_files as f64) / (elapsed_ms / 1000.0)
        } else {
            0.0
        };
        let _ = match lang {
            AppLanguage::Chinese => writeln!(
                err_out,
                "已扫描 {} 个文件，耗时 {:.2} ms ({:.0} files/s)：已识别 {}，未知 {}，错误 {}",
                total_files, elapsed_ms, rate, identified, unknown, errors
            ),
            AppLanguage::English => writeln!(
                err_out,
                "Scanned {} files in {:.2} ms ({:.0} files/s): {} identified, {} unknown, {} errors",
                total_files, elapsed_ms, rate, identified, unknown, errors
            ),
        };
    }

    if write_failed || (errors > 0 && identified == 0 && unknown == 0) {
        4
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Condvar, Mutex};
    use std::time::Duration;

    fn make_unit_temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "kindprobe_batch_unit_{}_{}",
            name,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn test_batch_parallel_workers_overlap_with_sync_barrier() {
        let root = make_unit_temp_dir("overlap");
        let files: Vec<PathBuf> = (0..4)
            .map(|i| {
                let p = root.join(format!("item_{:02}.txt", i));
                std::fs::write(&p, b"hello").unwrap();
                p
            })
            .collect();

        // Synchronization gate: at least 2 distinct worker threads MUST enter the detector
        // simultaneously before either can proceed; a serial loop would time out and fail.
        let arrived = Arc::new((Mutex::new(0usize), Condvar::new()));
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));

        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = run_batch_with_io(
            &files,
            false,
            true,
            Some(2),
            AppLanguage::English,
            &mut out,
            &mut err,
            |_path, _opts| {
                let now = active.fetch_add(1, Ordering::SeqCst) + 1;
                max_active.fetch_max(now, Ordering::SeqCst);

                let (lock, cv) = &*arrived;
                let mut cnt = lock.lock().unwrap();
                *cnt += 1;
                if *cnt < 2 {
                    let (guard, timeout) = cv.wait_timeout(cnt, Duration::from_secs(3)).unwrap();
                    cnt = guard;
                    assert!(
                        !timeout.timed_out() || *cnt >= 2,
                        "workers did not execute concurrently within timeout"
                    );
                } else {
                    cv.notify_all();
                }
                drop(cnt);

                active.fetch_sub(1, Ordering::SeqCst);
                Ok(DetectionCore {
                    format_id: Some("txt"),
                    category: "text",
                    container_id: None,
                    identification: Identification::Format,
                    verification: kindprobe::Verification::Structure,
                    inspection: kindprobe::Inspection::Complete,
                    recommended_extension: Some("txt"),
                    accepted_extensions: &["txt"],
                    primary_evidence: None,
                    secondary_evidence: None,
                    warning: None,
                    candidate: None,
                })
            },
        );

        assert_eq!(code, 0);
        assert!(
            max_active.load(Ordering::SeqCst) >= 2,
            "expected >= 2 concurrent workers"
        );
        let lines: Vec<serde_json::Value> = String::from_utf8(out)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(lines.len(), 4);
        for (idx, line) in lines.iter().enumerate() {
            assert!(line["path"]
                .as_str()
                .unwrap()
                .ends_with(&format!("item_{:02}.txt", idx)));
        }

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_batch_preserves_order_even_when_first_item_finishes_last() {
        let root = make_unit_temp_dir("order");
        let files: Vec<PathBuf> = (0..3)
            .map(|i| {
                let p = root.join(format!("seq_{:02}.txt", i));
                std::fs::write(&p, b"data").unwrap();
                p
            })
            .collect();

        // seq_01 and seq_02 must finish BEFORE seq_00 is released, yet output order stays [0, 1, 2].
        let gate = Arc::new((Mutex::new(0usize), Condvar::new()));
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = run_batch_with_io(
            &files,
            false,
            true,
            Some(3),
            AppLanguage::English,
            &mut out,
            &mut err,
            |path, _opts| {
                let name = path.file_name().unwrap().to_string_lossy();
                let (lock, cv) = &*gate;
                if name == "seq_00.txt" {
                    let mut fast_done = lock.lock().unwrap();
                    while *fast_done < 2 {
                        let (g, timeout) =
                            cv.wait_timeout(fast_done, Duration::from_secs(3)).unwrap();
                        fast_done = g;
                        assert!(
                            !timeout.timed_out(),
                            "other workers failed to complete first"
                        );
                    }
                } else {
                    let mut fast_done = lock.lock().unwrap();
                    *fast_done += 1;
                    cv.notify_all();
                }
                Ok(DetectionCore {
                    format_id: Some("txt"),
                    category: "text",
                    container_id: None,
                    identification: Identification::Format,
                    verification: kindprobe::Verification::Structure,
                    inspection: kindprobe::Inspection::Complete,
                    recommended_extension: Some("txt"),
                    accepted_extensions: &["txt"],
                    primary_evidence: None,
                    secondary_evidence: None,
                    warning: None,
                    candidate: None,
                })
            },
        );

        assert_eq!(code, 0);
        let emitted_names: Vec<String> = String::from_utf8(out)
            .unwrap()
            .lines()
            .map(|l| {
                let v: serde_json::Value = serde_json::from_str(l).unwrap();
                Path::new(v["path"].as_str().unwrap())
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        assert_eq!(
            emitted_names,
            vec!["seq_00.txt", "seq_01.txt", "seq_02.txt"]
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_batch_write_failure_returns_exit_4() {
        struct FailingWriter;
        impl Write for FailingWriter {
            fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "simulated pipe fail",
                ))
            }
            fn flush(&mut self) -> io::Result<()> {
                Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "simulated flush fail",
                ))
            }
        }

        let root = make_unit_temp_dir("write_fail");
        let files: Vec<PathBuf> = (0..4)
            .map(|i| {
                let p = root.join(format!("w_{}.png", i));
                std::fs::write(&p, b"\x89PNG\r\n\x1a\n").unwrap();
                p
            })
            .collect();

        let mut err = Vec::new();
        let code = run_batch_with_io(
            &files,
            false,
            true,
            Some(2),
            AppLanguage::English,
            FailingWriter,
            &mut err,
            detect_path_core,
        );
        assert_eq!(code, 4);
        assert!(String::from_utf8_lossy(&err).contains("io.write_error"));

        let _ = std::fs::remove_dir_all(&root);
    }
}
