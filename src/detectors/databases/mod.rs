pub mod pcap;
pub mod parquet_avro;
pub mod sqlite;

use crate::model::DetectionReport;
use pcap::{probe_pcap, PcapMatch};
use parquet_avro::{probe_parquet_avro, ColumnarMatch};
use sqlite::{probe_sqlite, SqliteMatch};

pub enum DatabaseMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
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
