use std::io::{self, Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use crate::model::DetectionStats;

/// Detection budget and resource limits according to specification §9.
#[derive(Debug, Clone)]
pub struct DetectOptions {
    /// Initial header probe window (default 4 KiB)
    pub initial_window_size: usize,
    /// Total source bytes allowed to read (default 8 MiB)
    pub max_source_bytes_read: usize,
    /// Maximum byte window for ZIP central directory parsing (default 1 MiB)
    pub max_zip_directory_window: usize,
    /// Maximum container directory / stream entries to inspect (default 10,000)
    pub max_entries: usize,
    /// Maximum decompressed size per XML document (default 256 KiB)
    pub max_single_decompressed_bytes: usize,
    /// Maximum cumulative decompressed bytes (default 1 MiB)
    pub max_total_decompressed_bytes: usize,
    /// Maximum XML element depth (default 64)
    pub max_xml_depth: usize,
}

impl Default for DetectOptions {
    fn default() -> Self {
        Self {
            initial_window_size: 4 * 1024,
            max_source_bytes_read: 8 * 1024 * 1024,
            max_zip_directory_window: 1024 * 1024,
            max_entries: 10_000,
            max_single_decompressed_bytes: 256 * 1024,
            max_total_decompressed_bytes: 1024 * 1024,
            max_xml_depth: 64,
        }
    }
}

/// A counting, budget-enforcing wrapper around any Read + Seek stream.
pub struct BudgetReader<R> {
    inner: R,
    max_read_bytes: usize,
    total_read_bytes: usize,
    seek_count: usize,
    budget_exceeded: Arc<AtomicBool>,
    window_state: Option<(Arc<AtomicUsize>, Arc<AtomicBool>, Arc<AtomicBool>, usize)>,
}

impl<R> BudgetReader<R> {
    pub fn new(inner: R, max_read_bytes: usize) -> Self {
        Self {
            inner,
            max_read_bytes,
            total_read_bytes: 0,
            seek_count: 0,
            budget_exceeded: Arc::new(AtomicBool::new(false)),
            window_state: None,
        }
    }

    pub fn start_window(
        &mut self,
        window_budget: usize,
    ) -> (Arc<AtomicUsize>, Arc<AtomicBool>, Arc<AtomicBool>) {
        let counter = Arc::new(AtomicUsize::new(0));
        let exceeded = Arc::new(AtomicBool::new(false));
        let active = Arc::new(AtomicBool::new(true));
        self.window_state = Some((
            counter.clone(),
            exceeded.clone(),
            active.clone(),
            window_budget,
        ));
        (counter, exceeded, active)
    }

    pub fn budget_exceeded_handle(&self) -> Arc<AtomicBool> {
        self.budget_exceeded.clone()
    }

    pub fn total_read_bytes(&self) -> usize {
        self.total_read_bytes
    }

    pub fn seek_count(&self) -> usize {
        self.seek_count
    }

    pub fn is_budget_exceeded(&self) -> bool {
        self.budget_exceeded.load(Ordering::Relaxed)
    }

    pub fn populate_stats(&self, stats: &mut DetectionStats) {
        stats.source_bytes_read = self.total_read_bytes;
        stats.seek_count = self.seek_count;
    }

    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: Read> Read for BudgetReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.total_read_bytes >= self.max_read_bytes {
            self.budget_exceeded.store(true, Ordering::Relaxed);
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "budget.source_read_exceeded: total source read budget exhausted",
            ));
        }

        let mut remaining = self.max_read_bytes - self.total_read_bytes;

        if let Some((counter, exceeded, active, budget)) = &self.window_state {
            if active.load(Ordering::Relaxed) {
                let current_window = counter.load(Ordering::Relaxed);
                if current_window >= *budget {
                    exceeded.store(true, Ordering::Relaxed);
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "budget.zip_directory_window_exceeded: window budget exhausted",
                    ));
                }
                remaining = remaining.min(*budget - current_window);
            }
        }

        let to_read = buf.len().min(remaining);
        match self.inner.read(&mut buf[..to_read]) {
            Ok(n) => {
                self.total_read_bytes += n;
                if let Some((counter, _, active, _)) = &self.window_state {
                    if active.load(Ordering::Relaxed) {
                        counter.fetch_add(n, Ordering::Relaxed);
                    }
                }
                Ok(n)
            }
            Err(e) => Err(e),
        }
    }
}

impl<R: Seek> Seek for BudgetReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.seek_count += 1;
        self.inner.seek(pos)
    }
}

/// A bounded reader for decompressing streams with individual and total byte caps.
pub struct DecompressLimitReader<R> {
    inner: R,
    max_single_bytes: usize,
    current_read: usize,
    exceeded: bool,
}

impl<R: Read> DecompressLimitReader<R> {
    pub fn new(inner: R, max_single_bytes: usize) -> Self {
        Self {
            inner,
            max_single_bytes,
            current_read: 0,
            exceeded: false,
        }
    }

    pub fn bytes_read(&self) -> usize {
        self.current_read
    }

    pub fn is_exceeded(&self) -> bool {
        self.exceeded
    }
}

impl<R: Read> Read for DecompressLimitReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        if self.current_read >= self.max_single_bytes {
            // Buffer already reached the budget limit.
            // Probe if the underlying reader actually has more bytes (real overflow),
            // or if it has reached normal EOF.
            let mut probe = [0u8; 1];
            match self.inner.read(&mut probe) {
                Ok(0) => {
                    // Normal EOF: exactly reached limit, no overflow.
                    return Ok(0);
                }
                Ok(_) => {
                    // There are additional bytes beyond budget limit -> truly exceeded.
                    self.exceeded = true;
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "budget.xml_output_exceeded: decompressed output budget exceeded",
                    ));
                }
                Err(e) => return Err(e),
            }
        }

        let remaining = self.max_single_bytes - self.current_read;
        let to_read = buf.len().min(remaining);
        match self.inner.read(&mut buf[..to_read]) {
            Ok(n) => {
                self.current_read += n;
                Ok(n)
            }
            Err(e) => Err(e),
        }
    }
}
