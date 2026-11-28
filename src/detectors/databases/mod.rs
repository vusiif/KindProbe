pub mod pcap;
pub mod parquet_avro;
pub mod sqlite;

use crate::model::{DetectionCore, DetectionReport};
use pcap::{probe_pcap, probe_pcap_core, PcapMatch};
use parquet_avro::{probe_parquet_avro, probe_parquet_avro_core, ColumnarMatch};
use sqlite::{probe_sqlite, probe_sqlite_core, SqliteMatch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseMatchCore {
    Confirmed(DetectionCore),
    Truncated(&'static str),
    None,
}

pub enum DatabaseMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes database, columnar and network capture signatures returning zero-allocation DetectionCore.
pub fn probe_database_signatures_core(header: &[u8]) -> DatabaseMatchCore {
    // 1. SQLite 3
    if header.len() < 16 && !header.is_empty() && sqlite::SQLITE3_MAGIC.starts_with(header) {
        return DatabaseMatchCore::Truncated("sqlite3");
    }
    if let Some(core) = probe_sqlite_core(header) {
        return DatabaseMatchCore::Confirmed(core);
    }

    // 2. PCAP / PCAPNG
    if header.len() < 4
        && header.len() >= 2
        && (header.starts_with(&[0xD4, 0xC3])
            || header.starts_with(&[0xA1, 0xB2])
            || header.starts_with(&[0x4D, 0x3C])
            || header.starts_with(&[0x0A, 0x0D]))
    {
        return DatabaseMatchCore::Truncated("pcap");
    }
    if let Some(core) = probe_pcap_core(header) {
        return DatabaseMatchCore::Confirmed(core);
    }

    // 3. Parquet / Avro
    if header.len() < 4 && !header.is_empty() {
        if header.starts_with(b"PAR") {
            return DatabaseMatchCore::Truncated("parquet");
        } else if header.starts_with(b"Obj") {
            return DatabaseMatchCore::Truncated("avro");
        }
    }
    if let Some(core) = probe_parquet_avro_core(header) {
        return DatabaseMatchCore::Confirmed(core);
    }

    DatabaseMatchCore::None
}

/// Probes database, columnar and network capture signatures (SQLite3, PCAP, PCAPNG, Parquet, Avro).
pub fn probe_database_signatures(header: &[u8]) -> DatabaseMatch {
    // 1. SQLite 3
    match probe_sqlite(header) {
        SqliteMatch::Confirmed(rep) => return DatabaseMatch::Confirmed(rep),
        SqliteMatch::Truncated(fmt) => return DatabaseMatch::Truncated(fmt),
        SqliteMatch::None => (),
    }

    // 2. PCAP / PCAPNG
    match probe_pcap(header) {
        PcapMatch::Confirmed(rep) => return DatabaseMatch::Confirmed(rep),
        PcapMatch::Truncated(fmt) => return DatabaseMatch::Truncated(fmt),
        PcapMatch::None => (),
    }

    // 3. Parquet / Avro
    match probe_parquet_avro(header) {
        ColumnarMatch::Confirmed(rep) => return DatabaseMatch::Confirmed(rep),
        ColumnarMatch::Truncated(fmt) => return DatabaseMatch::Truncated(fmt),
        ColumnarMatch::None => (),
    }

    DatabaseMatch::None
}
