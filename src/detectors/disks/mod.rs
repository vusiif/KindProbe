pub mod iso;
pub mod qcow;
pub mod vhd;
pub mod vmdk;

use crate::model::{DetectionCore, DetectionReport};
pub use iso::{probe_iso_buffer, probe_iso_buffer_core, probe_iso_stream, IsoMatch};
use qcow::{probe_qcow, probe_qcow_core, QcowMatch};
use vhd::{probe_vhd, probe_vhd_core, VhdMatch};
use vmdk::{probe_vmdk, probe_vmdk_core, VmdkMatch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskMatchCore {
    Confirmed(DetectionCore),
    Truncated(&'static str),
    None,
}

pub enum DiskMatch {
    Confirmed(Box<DetectionReport>),
    Truncated(String),
    None,
}

/// Probes virtual disk image and container signatures returning zero-allocation DetectionCore.
pub fn probe_disk_signatures_core(header: &[u8]) -> DiskMatchCore {
    // 1. QCOW / QCOW2
    if header.len() < 4 && !header.is_empty() && header.starts_with(b"QF") {
        return DiskMatchCore::Truncated("qcow2");
    }
    if let Some(core) = probe_qcow_core(header) {
        return DiskMatchCore::Confirmed(core);
    }

    // 2. VMDK
    if header.len() < 4 && !header.is_empty() && (header.starts_with(b"KD") || header.starts_with(b"# ")) {
        return DiskMatchCore::Truncated("vmdk");
    }
    if let Some(core) = probe_vmdk_core(header) {
        return DiskMatchCore::Confirmed(core);
    }

    // 3. VHD / VHDX
    if header.len() < 8 && !header.is_empty() {
        if header.starts_with(b"vhdx") {
            return DiskMatchCore::Truncated("vhdx");
        } else if header.starts_with(b"cone") {
            return DiskMatchCore::Truncated("vhd");
        }
    }
    if let Some(core) = probe_vhd_core(header) {
        return DiskMatchCore::Confirmed(core);
    }

    // 4. ISO 9660
    if let Some(core) = probe_iso_buffer_core(header) {
        return DiskMatchCore::Confirmed(core);
    }

    DiskMatchCore::None
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
