use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Identification, Inspection, Verification};
use std::ffi::OsStr;

#[test]
fn test_detect_elf64_and_elf32() {
    let opts = DetectOptions::default();

    // 1. ELF 64-bit Little-Endian executable
    let mut elf64 = vec![0u8; 64];
    elf64[0..4].copy_from_slice(b"\x7FELF");
    elf64[4] = 2; // 64-bit
    elf64[5] = 1; // Little-endian
    elf64[6] = 1; // Current version
    elf64[16..18].copy_from_slice(&2u16.to_le_bytes()); // ET_EXEC

    let rep64 = detect_bytes(&elf64, &opts).expect("detect elf64");
    assert_eq!(rep64.identification, Identification::Format);
    assert_eq!(rep64.format_id.as_deref(), Some("elf"));
    assert_eq!(rep64.category, "executable");
    assert_eq!(rep64.verification, Verification::Structure);
    assert_eq!(rep64.inspection, Inspection::Complete);
    assert_eq!(rep64.recommended_extension.as_deref(), Some("elf"));
    assert!(rep64.evidence.iter().any(|e| e.code == "sig.elf.magic"));
    assert!(rep64.evidence.iter().any(|e| e.code == "sig.elf.header"));

    // Extension matching
    assert_eq!(
        compare_extension(&rep64, Some(OsStr::new("elf"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep64, Some(OsStr::new("so"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep64, Some(OsStr::new("bin"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep64, Some(OsStr::new("exe"))),
        ExtensionStatus::Mismatch
    );

    // 2. ELF 32-bit Little-Endian shared library
    let mut elf32 = vec![0u8; 52];
    elf32[0..4].copy_from_slice(b"\x7FELF");
    elf32[4] = 1; // 32-bit
    elf32[5] = 1; // Little-endian
    elf32[6] = 1; // Current version
    elf32[16..18].copy_from_slice(&3u16.to_le_bytes()); // ET_DYN

    let rep32 = detect_bytes(&elf32, &opts).expect("detect elf32");
    assert_eq!(rep32.identification, Identification::Format);
    assert_eq!(rep32.format_id.as_deref(), Some("elf"));
    assert_eq!(rep32.verification, Verification::Structure);
}

#[test]
fn test_detect_pe_exe_and_dll() {
    let opts = DetectOptions::default();

    // 1. Windows PE .exe (Characteristics = 0x0102)
    let mut pe_exe = vec![0u8; 256];
    pe_exe[0..2].copy_from_slice(b"MZ");
    let pe_offset = 0x80usize;
    pe_exe[0x3C..0x40].copy_from_slice(&(pe_offset as u32).to_le_bytes());
    pe_exe[pe_offset..pe_offset + 4].copy_from_slice(b"PE\0\0");
    // Characteristics at pe_offset + 22
    pe_exe[pe_offset + 22..pe_offset + 24].copy_from_slice(&0x0102u16.to_le_bytes());

    let rep_exe = detect_bytes(&pe_exe, &opts).expect("detect pe exe");
    assert_eq!(rep_exe.identification, Identification::Format);
    assert_eq!(rep_exe.format_id.as_deref(), Some("pe"));
    assert_eq!(rep_exe.category, "executable");
    assert_eq!(rep_exe.verification, Verification::Structure);
    assert_eq!(rep_exe.recommended_extension.as_deref(), Some("exe"));
    assert_eq!(
        rep_exe.accepted_extensions,
        vec!["exe", "dll", "sys", "efi"]
    );
    assert!(rep_exe
        .evidence
        .iter()
        .any(|e| e.code == "sig.pe.mz_header"));
    assert!(rep_exe
        .evidence
        .iter()
        .any(|e| e.code == "sig.pe.signature"));

    assert_eq!(
        compare_extension(&rep_exe, Some(OsStr::new("exe"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_exe, Some(OsStr::new("dll"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_exe, Some(OsStr::new("elf"))),
        ExtensionStatus::Mismatch
    );

    // 2. Windows PE .dll (Characteristics = 0x2102: IMAGE_FILE_DLL)
    let mut pe_dll = pe_exe.clone();
    pe_dll[pe_offset + 22..pe_offset + 24].copy_from_slice(&0x2102u16.to_le_bytes());

    let rep_dll = detect_bytes(&pe_dll, &opts).expect("detect pe dll");
    assert_eq!(rep_dll.identification, Identification::Format);
    assert_eq!(rep_dll.format_id.as_deref(), Some("pe"));
    assert_eq!(rep_dll.recommended_extension.as_deref(), Some("dll"));

    // 3. DOS MZ executable without PE header
    let dos_mz = b"MZ\x90\x00\x03\x00\x00\x00\x04\x00\x00\x00\xFF\xFF\x00\x00";
    let rep_dos = detect_bytes(dos_mz, &opts).expect("detect dos mz");
    assert_eq!(rep_dos.identification, Identification::Format);
    assert_eq!(rep_dos.format_id.as_deref(), Some("pe"));
    assert_eq!(rep_dos.verification, Verification::Signature);
}

#[test]
fn test_detect_wasm_binary() {
    let opts = DetectOptions::default();

    let mut wasm = Vec::new();
    wasm.extend_from_slice(b"\x00asm"); // Magic
    wasm.extend_from_slice(&(1u32.to_le_bytes())); // Version 1

    let rep = detect_bytes(&wasm, &opts).expect("detect wasm");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("wasm"));
    assert_eq!(rep.category, "executable");
    assert_eq!(rep.verification, Verification::Structure);
    assert_eq!(rep.recommended_extension.as_deref(), Some("wasm"));
    assert_eq!(rep.accepted_extensions, vec!["wasm"]);
    assert!(rep.evidence.iter().any(|e| e.code == "sig.wasm.magic"));
    assert!(rep.evidence.iter().any(|e| e.code == "sig.wasm.version"));

    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("wasm"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("exe"))),
        ExtensionStatus::Mismatch
    );
}

#[test]
fn test_detect_macho_and_java_class() {
    let opts = DetectOptions::default();

    // 1. Mach-O 64-bit Little-Endian
    let macho64 = vec![0xCF, 0xFA, 0xED, 0xFE, 0x07, 0x00, 0x00, 0x01];
    let rep_macho = detect_bytes(&macho64, &opts).expect("detect macho");
    assert_eq!(rep_macho.identification, Identification::Format);
    assert_eq!(rep_macho.format_id.as_deref(), Some("macho"));
    assert_eq!(rep_macho.category, "executable");
    assert_eq!(rep_macho.verification, Verification::Structure);
    assert_eq!(rep_macho.recommended_extension.as_deref(), Some("dylib"));
    assert!(rep_macho
        .evidence
        .iter()
        .any(|e| e.code == "sig.macho.magic"));

    // 2. Java .class bytecode: 0xCAFEBABE, minor = 0, major = 52 (Java 8)
    let java_class = vec![0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x34];
    let rep_class = detect_bytes(&java_class, &opts).expect("detect java class");
    assert_eq!(rep_class.identification, Identification::Format);
    assert_eq!(rep_class.format_id.as_deref(), Some("class"));
    assert_eq!(rep_class.category, "executable");
    assert_eq!(rep_class.recommended_extension.as_deref(), Some("class"));
    assert!(rep_class
        .evidence
        .iter()
        .any(|e| e.code == "sig.java.class"));

    // 3. Mach-O Universal FAT Binary: 0xCAFEBABE, nfat_arch = 2
    let macho_fat = vec![0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x02];
    let rep_fat = detect_bytes(&macho_fat, &opts).expect("detect macho fat");
    assert_eq!(rep_fat.identification, Identification::Format);
    assert_eq!(rep_fat.format_id.as_deref(), Some("macho"));
    assert!(rep_fat
        .evidence
        .iter()
        .any(|e| e.code == "sig.macho.fat_magic"));
}

#[test]
fn test_detect_executables_truncated() {
    let opts = DetectOptions::default();

    // 1. Truncated ELF
    let rep_elf = detect_bytes(b"\x7FEL", &opts).expect("truncated elf");
    assert_eq!(rep_elf.identification, Identification::Unknown);
    assert_eq!(
        rep_elf.candidates.as_deref(),
        Some(&["elf".to_string()][..])
    );

    // 2. Truncated PE
    let rep_pe = detect_bytes(b"M", &opts).expect("truncated pe");
    assert_eq!(rep_pe.identification, Identification::Unknown);
    assert_eq!(rep_pe.candidates.as_deref(), Some(&["pe".to_string()][..]));

    // 3. Truncated WASM
    let rep_wasm = detect_bytes(b"\x00as", &opts).expect("truncated wasm");
    assert_eq!(rep_wasm.identification, Identification::Unknown);
    assert_eq!(
        rep_wasm.candidates.as_deref(),
        Some(&["wasm".to_string()][..])
    );

    // 4. Truncated Mach-O
    let rep_macho = detect_bytes(&[0xFE, 0xED], &opts).expect("truncated macho");
    assert_eq!(rep_macho.identification, Identification::Unknown);
    assert_eq!(
        rep_macho.candidates.as_deref(),
        Some(&["macho".to_string()][..])
    );
}
