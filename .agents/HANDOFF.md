# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-23  
> **当前版本**：v0.1.0 (格式支持跨越式突破 810 项大关！架构解耦升级为模块注册表+静态切片总装表双层体系，全量 46 个规则文件严密收敛于 52~80 行黄金规范，45 大测试套件 183 用例全绿，Clippy 0 警告)  
> **远端分支**：`origin/main` (前次提交 `3969d90` 672 项格式已推送)

---

## 1. [已完成] (Completed)

### 1.1 架构分层解耦与突破 810 项格式里程碑
为彻底消除行数超标瓶颈并维持 50~80 行铁律，将 `src/detectors/rules/generated/` 演进为“模块注册中心 + 静态规则汇总装配表”的双层架构：
- [`src/detectors/rules/generated/mod.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/mod.rs) (53 行)：声明全部 44 个领域分卷模块，导出汇总常量，稳定收敛；
- [`src/detectors/rules/generated/rule_tables.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/rule_tables.rs) (52 行)：统一装配 44 个分卷静态规则数组切片 `GENERATED_RULES`，对外类型 `&[&[SignatureRule]]` 保持零破坏；
- 全量 46 个实现文件（44 个规则分卷 + 2 个架构文件）行数严格稳定在 **52~80 行**黄金规范区间。

### 1.2 本次新增 11 个跨学科垂直领域分卷（共 143 项新规则）
1. **工业自动化、PLC与组态协议** ([`industrial_plc_and_scada.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/industrial_plc_and_scada.rs), 70 行)：
   `s7p` (Siemens Step 7), `pro_codesys` (CoDeSys), `mer` (Rockwell FT Machine Edition), `apa` (Rockwell FT Archive), `cxp` (Omron CX-Programmer), `cxd` (Omron CX-Designer), `mwp` (Siemens S7-200), `eds` (CANopen/CIP EDS), `gsd` (Profibus/Profinet GSD), `xcos` (Scilab Xcos), `s7opt` (Step 7 Options), `smc` (Schneider SoMachine), `pcu` (Sinumerik CNC).
2. **大型机磁带与经典微机软盘镜像** ([`mainframe_and_legacy_fs.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/mainframe_and_legacy_fs.rs), 70 行)：
   `het` (Hercules Emulated Tape), `atr` (Atari 8-bit SIO), `crt_c64` (Commodore 64 Cartridge), `xdf` (Sharp X68000 XDF), `dim` (Sharp X68000 DIM), `nfd` (PC-98 T98 FDI), `bkd` (Soviet BK-0010 Disk), `dsk_cpc` (Amstrad CPC DSK), `ipf` (Amiga/Atari CAPS SPS), `stx` (Pasti Atari ST ProCopy), `sad` (SAM Coupe Disk), `cpm_imd` (ImageDisk IMD CP/M/DOS), `hdi` (PC-98 Anex86 HDI).
3. **高能物理、质谱与光谱分析** ([`physics_and_spectroscopy.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/physics_and_spectroscopy.rs), 70 行)：
   `root` (CERN ROOT Framework), `mzml` (Proteomics mzML), `mzxml` (Legacy mzXML), `jcamp` (JCAMP-DX Spectroscopy), `shelx` (SHELX Crystallography), `baf` (Bruker Daltonics MS), `mgf` (Mascot Generic MS/MS), `mca` (Canberra MCA Spectrum), `nmredata` (NMR Data Exchange), `nexus` (NeXus Synchrotron/Neutron XML), `orca` (ORCA Quantum Chemistry), `gaussian` (Gaussian 09/16 Input), `gamess` (GAMESS Log).
4. **专业非编工程、LUT与调色剪辑表** ([`video_editing_and_presets.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/video_editing_and_presets.rs), 70 行)：
   `cube` (Iridas/Adobe 3D LUT), `look` (SpeedGrade Look), `edl` (CMX 3600 Edit List), `ale` (Avid Log Exchange), `cdl` (ASC Color Decision List), `aep` (Adobe After Effects RIFX Egg!), `c4d` (Cinema 4D), `otio` (OpenTimelineIO), `fcpxml` (Final Cut Pro XML), `fcp` (Final Cut Pro 7 KeyG), `veg` (Sony Vegas Pro vegs), `vpj` (VideoPad vprj), `tvp` (TVPaint Animation).
5. **气象水文雷达与高精度定位遥感** ([`sensors_and_meteorology.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/sensors_and_meteorology.rs), 70 行)：
   `bufr` (WMO BUFR), `mcl` (MicroWave Radar MCL1), `nexrad` (NOAA WSR-88D Level II), `iris` (Vaisala Sigmet IRIS), `cinrad` (China New Gen Weather Radar), `e57` (ASTM E57 3D LiDAR), `cwls_las` (CWLS LAS Well Log), `mseed` (IRIS Mini-SEED), `ubx` (u-blox GNSS Binary Protocol), `nmea` (NMEA 0183 GNSS Sentence), `novatel_gps` (NovAtel OEM GNSS), `dorade` (Super-SWEEP Radar), `fpa` (CMC FPA Weather Data).
6. **电子设计自动化与集成电路版图** ([`eda_and_hardware_layout.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/eda_and_hardware_layout.rs), 70 行)：
   `gdsii` (Calma GDSII Stream), `oasis` (SEMI P39 OASIS), `vcd_waveform` (IEEE 1364 Verilog VCD), `ghdl_ghw` (GHDL Waveform), `lef` (Cadence LEF), `def_layout` (Cadence DEF), `liberty_timing` (Synopsys Liberty), `gerber_x2` (RS-274X Gerber X2), `excellon_drl` (Excellon Drill), `ipc2581` (IPC-2581 PCB Data), `odb_step` (ODB++ Step), `spef` (IEEE 1481 Parasitic SPEF), `wgl` (Waveform Generation Language).
7. **神经电生理、核磁共振与生物信号** ([`neuro_imaging_and_signals.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/neuro_imaging_and_signals.rs), 70 行)：
   `tmsi_poly5` (TMSi Poly5 Biosignals), `bdf_biosemi` (BioSemi 24-bit EEG), `cnt_neuroscan` (Neuroscan Continuous EEG), `brainvision_vhdr` (BrainVision Header), `brainvision_vmrk` (BrainVision Markers), `afni_head` (AFNI Header), `nifti2` (NIfTI-2 Neuroimaging), `freesurfer_surf` (FreeSurfer Triangle Surface), `eeglab_set` (EEGLAB Dataset), `micromed_trc` (Micromed SystemPLUS TRC), `xdf_biosignals` (LabStreamingLayer XDF), `spike2_smr` (CED Spike2 Data), `ni_tdms` (NI LabVIEW TDMS).
8. **通信协议、无线电采样与总线捕获** ([`telecom_protocols_and_sdr.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/telecom_protocols_and_sdr.rs), 70 行)：
   `sigmf_meta` (SigMF Metadata JSON), `hdlc_stream` (HDLC Bit Stream Capture), `mpt1327` (MPT1327 Trunked Radio), `dmr_capture` (ETSI DMR Packet Capture), `tetra_capture` (TETRA Packet Capture), `dstar_voice` (D-STAR Digital Voice), `yaesu_fusion` (Yaesu C4FM Stream), `aprs_packet` (APRS Raw TNC Frame), `pcapng` (PCAPNG Enhanced Packet Capture), `btsnoop` (Bluetooth HCI Snoop), `kismet_log` (Kismet Wireless Sniffer), `rfcat_capture` (RFCat Sub-GHz Capture), `gnuradio_grc` (GNU Radio Flowgraph).
9. **3D游戏引擎资源与场景关卡资产** ([`game_engines_and_virtual_worlds.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/game_engines_and_virtual_worlds.rs), 70 行)：
   `unity_scene` (Unity 3D YAML Scene), `godot_scene` (Godot Text Scene), `godot_resource` (Godot Text Resource), `unreal_uasset` (Unreal Engine Asset), `unreal_upk` (Unreal Engine 3 Package), `cryengine_chunk` (Crytek CryEngine Chunk), `source_bsp` (Valve Source BSP Map), `source_mdl` (Valve Source 3D Model), `source_vmt` (Valve Material Type), `rockstar_rpf` (Rockstar Games RPF7 Archive), `infinity_ward_ff` (Call of Duty FastFile), `ogre3d_mesh` (OGRE 3D Mesh), `nintendo_yaz0` (Nintendo Yaz0 Compressed Archive).
10. **安全凭证、数字信封与密码令牌** ([`cryptographic_tokens_and_security.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/cryptographic_tokens_and_security.rs), 70 行)：
    `pkcs7_pem` (PKCS #7 PEM), `spkac` (Netscape SPKAC), `tss2_key` (TPM2 TSS FAPI Key), `pkcs15_profile` (PKCS #15 Smartcard Profile), `wireguard_conf` (WireGuard Interface Config), `openvpn_conf` (OpenVPN Client Config), `clamav_cvd` (ClamAV Virus Database Container), `tpm2_key` (TCG TPM 2.0 Key), `yubikey_oath` (YubiKey OATH Backup), `age_identity` (Age Secret Key), `bitwarden_export` (Bitwarden Encrypted Vault), `keepassx_xml` (KeePassX XML Database), `android_adb_backup` (Android ADB Backup).
11. **高级专用编码、归档与地理容器** ([`advanced_codecs_and_containers.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/advanced_codecs_and_containers.rs), 70 行)：
    `cuneiform_fed` (Cuneiform OCR Project), `nvidia_ptx` (NVIDIA PTX Assembly), `spirv_shader` (Khronos SPIR-V Shader Binary), `dvd_bup` (DVD Backup Navigation IFO), `canon_cr3` (Canon CR3 Raw Image), `panasonic_rw2` (Panasonic RW2 Raw Image), `olympus_orf` (Olympus ORF Raw Image), `geopackage` (OGC GeoPackage SQLite Container), `internet_archive_arc` (Internet Archive ARC Format), `mobipocket` (Mobipocket PalmDOC eBook), `zstd_dict` (Zstandard Compression Dictionary), `cramfs_img` (Linux CramFS ROM File System), `bzip3_archive` (Bzip3 Compressed Archive).

### 1.3 全局扩展名索引与测试套件
- **全局扩展名索引表** ([`src/extension/registry.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/extension/registry.rs))：
  - 条目数由 672 项一举跃升至 **811 项**；
  - 严格保持单调递增 ASCII 字典排序，0 重复项；
- **全套自动化测试套件** ([`tests/test_batch_800_milestone.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/tests/test_batch_800_milestone.rs))：
  - 新增专用端到端回归测试，包含工业、物理、遥感、EDA、神经电生理、游戏引擎、加密安全等 4 组 20 个代表性用例全部通过。

---

## 2. [当前状态] (Current State)

- **本地测试状态**：
  - 全套 45 个测试套件，**183 个测试用例全部通过 (183/183 PASS)**；
  - `cargo clippy --all-targets -- -D warnings` **0 警告，0 错误**；
  - `src/detectors/rules/generated/` 下全量 46 个文件行数严格分布在 **52~80 行**区间（100% 严守规范）；
- **远端 CI 状态**：
  - 前序提交 `3969d90`（672 项）已推送至远程 `origin/main`。

---

## 3. [待办与下一步] (Next Steps)

1. **原子提交并推送到远端主线**：
   - 提交信息：`feat(detectors): expand format coverage to 810+ formats across 44 partitions`；
   - 推送至 `origin/main`，监控 GitHub Actions CI 三端测试通过。
2. **向用户汇报达成 800+ 格式里程碑**。
