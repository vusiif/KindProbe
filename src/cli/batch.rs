use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

use kindprobe::budget::DetectOptions;
use kindprobe::model::DetectError;
use kindprobe::{detect_path_core, Identification};

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

fn process_batch_file(
    path: &std::path::Path,
    options: &DetectOptions,
    jsonl_mode: bool,
    lang: AppLanguage,
    out: &mut impl Write,
    err_out: &mut impl Write,
) -> (usize, usize, usize) {
    let path_str = path.to_string_lossy();
    match detect_path_core(path, options) {
        Ok(core) => {
            if jsonl_mode {
                let _ = serde_json::to_writer(
                    &mut *out,
                    &BatchResultJson {
                        path: &path_str,
                        status: "ok",
                        format_id: core.format_id,
                        category: core.category,
                        container_id: core.container_id,
                        identification: core.identification,
                        recommended_extension: core.recommended_extension,
                    },
                );
                let _ = out.write_all(b"\n");
            } else {
                let fmt = core.format_id.unwrap_or("unknown");
                match lang {
                    AppLanguage::Chinese => {
                        let _ = writeln!(
                            out,
                            "{} -> {} ({})",
                            path_str,
                            fmt,
                            format_category_cn(core.category)
                        );
                    }
                    AppLanguage::English => {
                        let _ = writeln!(out, "{} -> {} ({})", path_str, fmt, core.category);
                    }
                }
            }
            if core.identification == Identification::Format {
                (1, 0, 0)
            } else {
                (0, 1, 0)
            }
        }
        Err(err) => {
            let code = error_code_for_detect_error(&err);
            if jsonl_mode {
                let _ = serde_json::to_writer(
                    &mut *out,
                    &BatchErrorJson {
                        path: &path_str,
                        status: "error",
                        error_code: code,
                        message: err.to_string(),
                    },
                );
                let _ = out.write_all(b"\n");
            } else {
                match lang {
                    AppLanguage::Chinese => {
                        let _ = writeln!(err_out, "错误 [{}]: {} ({})", code, path_str, err);
                    }
                    AppLanguage::English => {
                        let _ = writeln!(err_out, "Error [{}]: {} ({})", code, path_str, err);
                    }
                }
            }
            (0, 0, 1)
        }
    }
}

pub(crate) fn run_batch(
    paths: &[PathBuf],
    recursive: bool,
    jsonl_mode: bool,
    jobs: Option<usize>,
    lang: AppLanguage,
) -> i32 {
    let start = Instant::now();
    let options = DetectOptions::default();
    let stdout = std::io::stdout();
    let mut out = BufWriter::with_capacity(64 * 1024, stdout.lock());

    let mut total_files: usize = 0;
    let mut identified: usize = 0;
    let mut unknown: usize = 0;
    let mut errors: usize = 0;
    let mut file_paths: Vec<PathBuf> = Vec::new();

    // Explicit stack of (path, is_root_arg) so `--batch <dir>` scans 1 level even when !recursive
    let mut work_stack: Vec<(PathBuf, bool)> =
        paths.iter().rev().map(|p| (p.clone(), true)).collect();

    while let Some((path, is_root)) = work_stack.pop() {
        let meta = match std::fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                total_files += 1;
                errors += 1;
                let path_str = path.to_string_lossy();
                let code = error_code_for_detect_error(&DetectError::Io(std::io::Error::new(
                    e.kind(),
                    e.to_string(),
                )));
                if jsonl_mode {
                    let _ = serde_json::to_writer(
                        &mut out,
                        &BatchErrorJson {
                            path: &path_str,
                            status: "error",
                            error_code: code,
                            message: e.to_string(),
                        },
                    );
                    let _ = out.write_all(b"\n");
                } else {
                    match lang {
                        AppLanguage::Chinese => {
                            eprintln!("错误 [{}]: {} ({})", code, path_str, e);
                        }
                        AppLanguage::English => {
                            eprintln!("Error [{}]: {} ({})", code, path_str, e);
                        }
                    }
                }
                continue;
            }
        };

        let is_symlink_dir = meta.file_type().is_symlink() && path.is_dir();
        if is_symlink_dir {
            continue;
        }

        if meta.is_dir() && (recursive || is_root) {
            match std::fs::read_dir(&path) {
                Ok(rd) => {
                    let mut children: Vec<PathBuf> = rd
                        .filter_map(|res| res.ok())
                        .filter_map(|entry| {
                            let ft = entry.file_type().ok()?;
                            let child_path = entry.path();
                            // Skip symlinked directories to avoid traversal cycles
                            if ft.is_symlink() && child_path.is_dir() {
                                return None;
                            }
                            if !recursive && ft.is_dir() {
                                return None;
                            }
                            Some(child_path)
                        })
                        .collect();
                    children.sort();
                    for child in children.into_iter().rev() {
                        work_stack.push((child, false));
                    }
                }
                Err(e) => {
                    total_files += 1;
                    errors += 1;
                    let path_str = path.to_string_lossy();
                    if jsonl_mode {
                        let _ = serde_json::to_writer(
                            &mut out,
                            &BatchErrorJson {
                                path: &path_str,
                                status: "error",
                                error_code: "io.read_error",
                                message: e.to_string(),
                            },
                        );
                        let _ = out.write_all(b"\n");
                    } else {
                        match lang {
                            AppLanguage::Chinese => {
                                eprintln!("错误 [io.read_error]: {} ({})", path_str, e);
                            }
                            AppLanguage::English => {
                                eprintln!("Error [io.read_error]: {} ({})", path_str, e);
                            }
                        }
                    }
                }
            }
            continue;
        }

        file_paths.push(path);
    }

    total_files += file_paths.len();
    let default_jobs = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .clamp(2, 8);
    let num_threads = jobs
        .unwrap_or(default_jobs)
        .clamp(1, file_paths.len().max(1))
        .min(16);

    if file_paths.len() < 32 || num_threads <= 1 {
        let mut stderr = std::io::stderr();
        for path in &file_paths {
            let (id_inc, unk_inc, err_inc) =
                process_batch_file(path, &options, jsonl_mode, lang, &mut out, &mut stderr);
            identified += id_inc;
            unknown += unk_inc;
            errors += err_inc;
        }
    } else {
        #[derive(Default)]
        struct BatchChunkResult {
            id_cnt: usize,
            unk_cnt: usize,
            err_cnt: usize,
            local_out: Vec<u8>,
            local_err: Vec<u8>,
        }

        let chunk_size = file_paths.len().div_ceil(num_threads);
        let chunk_results: Vec<Result<BatchChunkResult, usize>> = std::thread::scope(|s| {
            let handles: Vec<_> = file_paths
                .chunks(chunk_size)
                .map(|chunk| {
                    let opts_ref = &options;
                    let chunk_len = chunk.len();
                    s.spawn(move || {
                        let mut res = BatchChunkResult {
                            local_out: Vec::with_capacity(chunk.len() * 128),
                            ..BatchChunkResult::default()
                        };
                        for path in chunk {
                            let (i, u, e) = process_batch_file(
                                path,
                                opts_ref,
                                jsonl_mode,
                                lang,
                                &mut res.local_out,
                                &mut res.local_err,
                            );
                            res.id_cnt += i;
                            res.unk_cnt += u;
                            res.err_cnt += e;
                        }
                        res
                    })
                    .join()
                    .map_err(|_| chunk_len)
                })
                .collect();
            handles
        });

        let mut stderr = std::io::stderr();
        for res in chunk_results {
            match res {
                Ok(chunk_res) => {
                    identified += chunk_res.id_cnt;
                    unknown += chunk_res.unk_cnt;
                    errors += chunk_res.err_cnt;
                    let _ = out.write_all(&chunk_res.local_out);
                    if !chunk_res.local_err.is_empty() {
                        let _ = stderr.write_all(&chunk_res.local_err);
                    }
                }
                Err(lost_count) => {
                    errors += lost_count;
                    match lang {
                        AppLanguage::Chinese => {
                            let _ = writeln!(stderr, "错误 [batch.worker_panic]: 批处理工作线程异常终止（{} 个文件未完成）", lost_count);
                        }
                        AppLanguage::English => {
                            let _ = writeln!(stderr, "Error [batch.worker_panic]: batch worker thread panicked ({} files failed)", lost_count);
                        }
                    }
                }
            }
        }
    }

    let _ = out.flush();
    if !jsonl_mode {
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        let rate = if elapsed_ms > 0.0 {
            (total_files as f64) / (elapsed_ms / 1000.0)
        } else {
            0.0
        };
        match lang {
            AppLanguage::Chinese => eprintln!(
                "已扫描 {} 个文件，耗时 {:.2} ms ({:.0} files/s)：已识别 {}，未知 {}，错误 {}",
                total_files, elapsed_ms, rate, identified, unknown, errors
            ),
            AppLanguage::English => eprintln!(
                "Scanned {} files in {:.2} ms ({:.0} files/s): {} identified, {} unknown, {} errors",
                total_files, elapsed_ms, rate, identified, unknown, errors
            ),
        }
    }

    if errors > 0 && identified == 0 && unknown == 0 {
        4
    } else {
        0
    }
}
