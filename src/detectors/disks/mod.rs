pub mod iso;
pub mod qcow;
pub mod vhd;
pub mod vmdk;

use crate::model::DetectionReport;
pub use iso::{probe_iso_buffer, probe_iso_stream, IsoMatch};
use qcow::{probe_qcow, QcowMatch};
use vhd::{probe_vhd, VhdMatch};
use vmdk::{probe_vmdk, VmdkMatch};

pub enum DiskMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes virtual disk image and container signatures from header window.
pub fn probe_disk_signatures(header: &[u8]) -> DiskMatch {
    // 1. QCOW / QCOW2
    match probe_qcow(header) {
        QcowMatch::Confirmed(rep) => return DiskMatch::Confirmed(rep),
        QcowMatch::Truncated(fmt) => return DiskMatch::Truncated(fmt),
        QcowMatch::None => (),
    }

    // 2. VMDK (Sparse / Descriptor)
    match probe_vmdk(header) {
        VmdkMatch::Confirmed(rep) => return DiskMatch::Confirmed(rep),
        VmdkMatch::Truncated(fmt) => return DiskMatch::Truncated(fmt),
        VmdkMatch::None => (),
    }

    // 3. VHD / VHDX
    match probe_vhd(header) {
        VhdMatch::Confirmed(rep) => return DiskMatch::Confirmed(rep),
        VhdMatch::Truncated(fmt) => return DiskMatch::Truncated(fmt),
        VhdMatch::None => (),
    }

    // 4. ISO 9660 buffer probe (if header large enough)
    if let IsoMatch::Confirmed(rep) = probe_iso_buffer(header) {
        return DiskMatch::Confirmed(rep);
    }

    DiskMatch::None
}
