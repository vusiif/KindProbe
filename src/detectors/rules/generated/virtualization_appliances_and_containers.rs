use crate::detectors::rules::types::{MagicPattern, SignatureRule};
use crate::model::Verification;

pub static VIRTUALIZATION_APPLIANCES_AND_CONTAINERS_RULES: &[SignatureRule] = &[
    SignatureRule {
        format_id: "ovf_envelope", category: "virtualization", container_id: None, recommended_ext: "ovf", accepted_exts: &["ovf"],
        patterns: &[MagicPattern::new(0, b"<Envelope xmlns=\"http://schemas.dmtf.org/ovf/envelope/1\"")], min_header_len: 54, priority: 50,
        evidence_code: "rule.ovf_envelope.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "singularity_sif", category: "virtualization", container_id: None, recommended_ext: "sif", accepted_exts: &["sif"],
        patterns: &[MagicPattern::new(0, b"SIFIMAGE\x00\x00\x00\x00")], min_header_len: 12, priority: 50,
        evidence_code: "rule.singularity_sif.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "vagrant_box_json", category: "virtualization", container_id: None, recommended_ext: "boxjson", accepted_exts: &["boxjson"],
        patterns: &[MagicPattern::new(0, b"{\"provider\":\"virtualbox\"")], min_header_len: 25, priority: 50,
        evidence_code: "rule.vagrant_box_json.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "docker_image_manifest_v2", category: "virtualization", container_id: None, recommended_ext: "dockermanifest", accepted_exts: &["dockermanifest"],
        patterns: &[MagicPattern::new(0, b"{\"schemaVersion\":2,\"mediaType\":\"application/vnd.docker.distribution.manifest.v2+json\"")], min_header_len: 83, priority: 50,
        evidence_code: "rule.docker_image_manifest_v2.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "oci_image_manifest", category: "virtualization", container_id: None, recommended_ext: "ocimanifest", accepted_exts: &["ocimanifest"],
        patterns: &[MagicPattern::new(0, b"{\"schemaVersion\":2,\"mediaType\":\"application/vnd.oci.image.manifest.v1+json\"")], min_header_len: 77, priority: 50,
        evidence_code: "rule.oci_image_manifest.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "qemu_cow_v1", category: "virtualization", container_id: None, recommended_ext: "qcow1", accepted_exts: &["qcow1"],
        patterns: &[MagicPattern::new(0, b"OO-M\x00\x00\x00\x01")], min_header_len: 8, priority: 50,
        evidence_code: "rule.qemu_cow_v1.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "vmware_nvram_image", category: "virtualization", container_id: None, recommended_ext: "nvram", accepted_exts: &["nvram"],
        patterns: &[MagicPattern::new(0, b"NVRAM\x00\x00\x00")], min_header_len: 8, priority: 50,
        evidence_code: "rule.vmware_nvram_image.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "xen_save_record", category: "virtualization", container_id: None, recommended_ext: "xen", accepted_exts: &["xen"],
        patterns: &[MagicPattern::new(0, b"LinuxGuestRecord")], min_header_len: 16, priority: 50,
        evidence_code: "rule.xen_save_record.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "bhyve_nvram_image", category: "virtualization", container_id: None, recommended_ext: "bnvram", accepted_exts: &["bnvram"],
        patterns: &[MagicPattern::new(0, b"BHYVE_NVRAM_V1")], min_header_len: 14, priority: 50,
        evidence_code: "rule.bhyve_nvram_image.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "parallels_pvs_config", category: "virtualization", container_id: None, recommended_ext: "pvs", accepted_exts: &["pvs"],
        patterns: &[MagicPattern::new(0, b"<?xml version=\"1.0\"?>\n<ParallelsVirtualMachine")], min_header_len: 48, priority: 50,
        evidence_code: "rule.parallels_pvs_config.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "virtualbox_vbox_config", category: "virtualization", container_id: None, recommended_ext: "vbox", accepted_exts: &["vbox"],
        patterns: &[MagicPattern::new(0, b"<?xml version=\"1.0\"?>\n<VirtualBox xmlns=\"http://www.virtualbox.org/\"")], min_header_len: 68, priority: 50,
        evidence_code: "rule.virtualbox_vbox_config.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "cloud_init_userdata", category: "virtualization", container_id: None, recommended_ext: "cloudconfig", accepted_exts: &["cloudconfig"],
        patterns: &[MagicPattern::new(0, b"#cloud-config\n")], min_header_len: 14, priority: 50,
        evidence_code: "rule.cloud_init_userdata.magic", verification: Verification::Signature, validator: None,
    },
    SignatureRule {
        format_id: "systemd_nspawn_config", category: "virtualization", container_id: None, recommended_ext: "nspawn", accepted_exts: &["nspawn"],
        patterns: &[MagicPattern::new(0, b"[Files]\nAllocateAttribute=")], min_header_len: 26, priority: 50,
        evidence_code: "rule.systemd_nspawn_config.magic", verification: Verification::Signature, validator: None,
    },
];
