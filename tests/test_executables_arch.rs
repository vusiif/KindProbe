use kindprobe::budget::DetectOptions;
use kindprobe::detect_bytes;
use kindprobe::model::{DetectionReport, Identification};
use std::process::Command;

const BIN_PATH: &str = env!("CARGO_BIN_EXE_kindprobe");

fn make_dummy_pe(machine: u16, opt_magic: u16, subsystem: u16, is_dll: bool) -> Vec<u8> {
    let mut pe = vec![0u8; 512];
    pe[0..2].copy_from_slice(b"MZ");
    let pe_offset = 0x80usize;
    pe[0x3C..0x40].copy_from_slice(&(pe_offset as u32).to_le_bytes());
    pe[pe_offset..pe_offset + 4].copy_from_slice(b"PE\0\0");
    // Machine at pe_offset + 4
    pe[pe_offset + 4..pe_offset + 6].copy_from_slice(&machine.to_le_bytes());
    // SizeOfOptionalHeader at pe_offset + 20
    pe[pe_offset + 20..pe_offset + 22].copy_from_slice(&224u16.to_le_bytes());
    // Characteristics at pe_offset + 22
    let charac = if is_dll { 0x2102u16 } else { 0x0102u16 };
    pe[pe_offset + 22..pe_offset + 24].copy_from_slice(&charac.to_le_bytes());
    // Optional Header Magic at pe_offset + 24
    pe[pe_offset + 24..pe_offset + 26].copy_from_slice(&opt_magic.to_le_bytes());
    // Subsystem at pe_offset + 24 + 68
    pe[pe_offset + 92..pe_offset + 94].copy_from_slice(&subsystem.to_le_bytes());
    pe
}

#[test]
fn test_detect_pe_cpu_architectures() {
    let opts = DetectOptions::default();

    // 1. ARM64 PE Executable (GUI)
    let pe_arm64 = make_dummy_pe(0xAA64, 0x020B, 2, false);
    let rep = detect_bytes(&pe_arm64, &opts).expect("detect pe arm64");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("pe"));
    assert_eq!(rep.recommended_extension.as_deref(), Some("exe"));
    assert!(rep.evidence.iter().any(|e| e.code == "sig.pe.arch.arm64"));
    assert!(rep.evidence.iter().any(|e| e.code == "sig.pe.opt.pe32plus"));
    assert!(rep
        .evidence
        .iter()
        .any(|e| e.code == "sig.pe.subsystem.gui"));

    // 2. x64 / AMD64 PE Executable (Console)
    let pe_x64 = make_dummy_pe(0x8664, 0x020B, 3, false);
    let rep = detect_bytes(&pe_x64, &opts).expect("detect pe x64");
    assert!(rep.evidence.iter().any(|e| e.code == "sig.pe.arch.x64"));
    assert!(rep
        .evidence
        .iter()
        .any(|e| e.code == "sig.pe.subsystem.console"));

    // 3. x86 PE DLL (32-bit)
    let pe_x86 = make_dummy_pe(0x014C, 0x010B, 2, true);
    let rep = detect_bytes(&pe_x86, &opts).expect("detect pe x86 dll");
    assert_eq!(rep.recommended_extension.as_deref(), Some("dll"));
    assert!(rep.evidence.iter().any(|e| e.code == "sig.pe.arch.x86"));
    assert!(rep.evidence.iter().any(|e| e.code == "sig.pe.opt.pe32"));

    // 4. ARM64EC (Emulation Compatible)
    let pe_arm64ec = make_dummy_pe(0xA641, 0x020B, 2, false);
    let rep = detect_bytes(&pe_arm64ec, &opts).expect("detect pe arm64ec");
    assert!(rep.evidence.iter().any(|e| e.code == "sig.pe.arch.arm64ec"));

    // 5. ARM Thumb-2 (Windows RT)
    let pe_thumb2 = make_dummy_pe(0x01C4, 0x010B, 2, false);
    let rep = detect_bytes(&pe_thumb2, &opts).expect("detect pe thumb2");
    assert!(rep
        .evidence
        .iter()
        .any(|e| e.code == "sig.pe.arch.arm_thumb2"));
}

#[test]
fn test_detect_elf_cpu_architectures() {
    let opts = DetectOptions::default();

    // 1. ELF ARM64 (AArch64) executable
    let mut elf_arm64 = vec![0u8; 64];
    elf_arm64[0..4].copy_from_slice(b"\x7FELF");
    elf_arm64[4] = 2; // 64-bit
    elf_arm64[5] = 1; // Little-endian
    elf_arm64[6] = 1; // Current version
    elf_arm64[16..18].copy_from_slice(&2u16.to_le_bytes()); // ET_EXEC
    elf_arm64[18..20].copy_from_slice(&0xB7u16.to_le_bytes()); // EM_AARCH64

    let rep = detect_bytes(&elf_arm64, &opts).expect("detect elf arm64");
    assert!(rep.evidence.iter().any(|e| e.code == "sig.elf.arch.arm64"));
    assert!(rep.evidence.iter().any(|e| e.code == "sig.elf.class.64bit"));
    assert!(rep
        .evidence
        .iter()
        .any(|e| e.code == "sig.elf.type.executable"));

    // 2. ELF RISC-V 64-bit shared object
    let mut elf_riscv = vec![0u8; 64];
    elf_riscv[0..4].copy_from_slice(b"\x7FELF");
    elf_riscv[4] = 2;
    elf_riscv[5] = 1;
    elf_riscv[6] = 1;
    elf_riscv[16..18].copy_from_slice(&3u16.to_le_bytes()); // ET_DYN
    elf_riscv[18..20].copy_from_slice(&0xF3u16.to_le_bytes()); // EM_RISCV

    let rep = detect_bytes(&elf_riscv, &opts).expect("detect elf riscv");
    assert_eq!(rep.recommended_extension.as_deref(), Some("so"));
    assert!(rep.evidence.iter().any(|e| e.code == "sig.elf.arch.riscv"));
}

#[test]
fn test_detect_macho_cpu_architectures() {
    let opts = DetectOptions::default();

    // 1. Mach-O ARM64 (Apple Silicon, 64-bit LE)
    let mut macho_arm64 = vec![0u8; 64];
    macho_arm64[0..4].copy_from_slice(&[0xCF, 0xFA, 0xED, 0xFE]); // MH_MAGIC_64 LE
    macho_arm64[4..8].copy_from_slice(&0x0100000Cu32.to_le_bytes()); // CPU_TYPE_ARM64

    let rep = detect_bytes(&macho_arm64, &opts).expect("detect macho arm64");
    assert_eq!(rep.format_id.as_deref(), Some("macho"));
    assert!(rep
        .evidence
        .iter()
        .any(|e| e.code == "sig.macho.arch.arm64"));

    // 2. Mach-O x86-64 (64-bit LE)
    let mut macho_x64 = vec![0u8; 64];
    macho_x64[0..4].copy_from_slice(&[0xCF, 0xFA, 0xED, 0xFE]);
    macho_x64[4..8].copy_from_slice(&0x01000007u32.to_le_bytes()); // CPU_TYPE_X86_64

    let rep = detect_bytes(&macho_x64, &opts).expect("detect macho x64");
    assert!(rep
        .evidence
        .iter()
        .any(|e| e.code == "sig.macho.arch.x86_64"));
}

#[test]
fn test_cli_architecture_display_e2e() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let pe_arm64_path = std::env::temp_dir().join(format!("test_sample_arm64_{}.exe", nonce));
    let pe_bytes = make_dummy_pe(0xAA64, 0x020B, 2, false);
    std::fs::write(&pe_arm64_path, pe_bytes).expect("write sample pe");

    // Chinese text display (with KINDPROBE_TEST_WIN_LANGID=2052 for deterministic Windows CI)
    let out_zh = Command::new(BIN_PATH)
        .env("LC_ALL", "zh_CN.UTF-8")
        .env("KINDPROBE_TEST_WIN_LANGID", "2052")
        .arg(&pe_arm64_path)
        .output()
        .expect("run cli zh");
    assert_eq!(out_zh.status.code(), Some(0));
    let stdout_zh = String::from_utf8_lossy(&out_zh.stdout);
    assert!(stdout_zh.contains("识别结果: 格式已确认 (pe)"));
    assert!(stdout_zh.contains("类别: 可执行文件"));
    assert!(stdout_zh.contains("目标架构: ARM64 (AArch64 / 64位)"));
    assert!(stdout_zh.contains("sig.pe.arch.arm64"));

    // English text display (with KINDPROBE_TEST_WIN_LANGID=1033 for deterministic Windows CI)
    let out_en = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .env("KINDPROBE_TEST_WIN_LANGID", "1033")
        .arg(&pe_arm64_path)
        .output()
        .expect("run cli en");
    assert_eq!(out_en.status.code(), Some(0));
    let stdout_en = String::from_utf8_lossy(&out_en.stdout);
    assert!(stdout_en.contains("Identification: Format (pe)"));
    assert!(stdout_en.contains("Category: executable"));
    assert!(stdout_en.contains("Target Architecture: ARM64 (AArch64 / 64-bit)"));
    assert!(stdout_en.contains("sig.pe.arch.arm64"));

    // JSON output
    let out_json = Command::new(BIN_PATH)
        .arg("--json")
        .arg(&pe_arm64_path)
        .output()
        .expect("run cli json");
    assert_eq!(out_json.status.code(), Some(0));
    let stdout_json = String::from_utf8_lossy(&out_json.stdout);
    let report: DetectionReport =
        serde_json::from_str(stdout_json.trim()).expect("deserialize json");
    assert_eq!(report.format_id.as_deref(), Some("pe"));
    let arch_ev = report
        .evidence
        .iter()
        .find(|e| e.code == "sig.pe.arch.arm64")
        .expect("arm64 evidence");
    assert_eq!(
        arch_ev.description.as_deref(),
        Some("Target Architecture: ARM64 (AArch64 / 64-bit)")
    );

    // Clean up
    let _ = std::fs::remove_file(&pe_arm64_path);
}
