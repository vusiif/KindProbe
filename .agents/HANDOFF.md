# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-23  
> **当前版本**：v0.1.0 (格式支持跨越式突破 1000+ 项大关！全量 59 个领域分卷模块、62 个实现文件严密收敛于 60~80 行黄金规范，全局扩展名注册表达到 1006 项，全套 49 个测试套件 187+ 用例全绿，Clippy 0 警告)  
> **远端分支**：`origin/main` (前次提交 `5b43e84` 810 项格式已推送)

---

## 1. [已完成] (Completed)

### 1.1 跨越 1000+ 项格式里程碑与架构持续收敛
KindProbe 在保持“单文件 50~80 行铁律”与“内容优先真实特征码”的前提下，成功实现 1000+ 项格式检测覆盖：
- [`src/detectors/rules/generated/mod.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/mod.rs) (68 行)：声明全部 59 个垂直领域分卷模块，导出汇总常量，严格稳定在 50~80 行区间；
- [`src/detectors/rules/generated/rule_tables.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/rule_tables.rs) (67 行)：集中装配 59 个分卷静态规则数组切片 `GENERATED_RULES`，对外类型 `&[&[SignatureRule]]` 保持零破坏；
- 全量 62 个实现文件（59 个规则分卷 + 3 个架构/模块文件）行数严格稳定在 **60~80 行**黄金规范区间（0 违规，平均 73.5 行）。

### 1.2 本次新增 15 个跨学科专业垂直领域分卷（共 195 项新规则）
1. **金融证券与国际银行清算** ([`finance_and_fintech.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/finance_and_fintech.rs), 70 行)：
   `fix_protocol` (FIX 4.x), `mt940_statement` (SWIFT MT940), `swift_mt_msg` (SWIFT MT Block 1), `fints_message` (FinTS/HBCI), `gnucash_xml` (GnuCash), `iso20022_pain001` (SEPA Credit Transfer), `iso20022_camt053` (Bank to Customer Statement), `fpml_financial` (FpML), `finvoice_xml` (Finvoice), `facturx_invoice` (Factur-X/ZUGFeRD), `ebics_request` (EBICS Banking), `bloomberg_data` (Bloomberg Data), `reuters_data` (Reuters RIC Data).
2. **网络管控、流采样与协议诊断** ([`network_management_and_snmp.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/network_management_and_snmp.rs), 70 行)：
   `sflow_v5_dump` (sFlow v5), `snmp_mib_definition` (SMIv2 MIB), `radius_dictionary` (FreeRADIUS Dict), `cisco_running_config` (Cisco IOS Config), `juniper_set_config` (Juniper Junos Config), `wireshark_k12_text` (Tektronix K12), `visual_networks_traffic` (Visual Networks), `coad_network_log` (COAD Log), `argus_audit_record` (Argus Network Audit), `ipfix_stream` (IPFIX Stream), `etherpeek_v7` (WildPackets EtherPeek), `airodump_ivs` (Aircrack-ng IVS), `netmon_2_capture` (Microsoft NetMon 2.x).
3. **虚拟化设备与容器运行时** ([`virtualization_appliances_and_containers.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/virtualization_appliances_and_containers.rs), 70 行)：
   `ovf_envelope` (DMTF OVF Appliance), `singularity_sif` (Singularity SIF Container), `vagrant_box_json` (Vagrant Box Metadata), `docker_image_manifest_v2` (Docker v2 Manifest), `oci_image_manifest` (OCI Image Manifest), `qemu_cow_v1` (QEMU COW v1), `vmware_nvram_image` (VMware NVRAM), `xen_save_record` (Xen Guest Save Record), `bhyve_nvram_image` (FreeBSD bhyve NVRAM), `parallels_pvs_config` (Parallels PVS VM), `virtualbox_vbox_config` (VirtualBox VBOX XML), `cloud_init_userdata` (cloud-config User Data), `systemd_nspawn_config` (systemd-nspawn Container).
4. **区块链、加密货币与硬件钱包** ([`cryptocurrency_and_blockchains.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/cryptocurrency_and_blockchains.rs), 70 行)：
   `bitcoin_blk_dat` (Bitcoin Mainnet BLK), `bitcoin_testnet_blk` (Bitcoin Testnet BLK), `litecoin_blk_dat` (Litecoin BLK), `dogecoin_blk_dat` (Dogecoin BLK), `monero_block_cache` (Monero Cache), `ethereum_keystore_v3` (Ethereum Keystore JSON), `solana_ledger_slice` (Solana Ledger Slice), `cardano_chunk` (Cardano Chunk), `bip39_seed_backup` (BIP-39 Mnemonic Backup), `electrum_wallet_v1` (Electrum Wallet), `trezor_metadata` (Trezor Hardware Backup), `ledger_backup_apdu` (Ledger APDU Dump), `chia_plot_v1` (Chia Proof of Space Plot).
5. **编译器IR、字节码与符号表** ([`compiler_ir_and_assemblies.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/compiler_ir_and_assemblies.rs), 70 行)：
   `llvm_mir_yaml` (LLVM Machine IR YAML), `swift_module_sib` (Swift Intermediate SIL), `graal_truffle_cache` (GraalVM Truffle Cache), `dalvik_odex` (Android Optimized DEX), `dalvik_vdex` (Android Verified DEX), `dalvik_oat` (Android ART OAT), `cranelift_clif` (Wasmtime Cranelift IR), `gimple_dump` (GCC GIMPLE IR), `spirv_assembly` (Khronos SPIR-V Assembly), `dotnet_cil_text` (ECMA-335 CIL Assembly), `elixir_beam_chunk` (Elixir EXIR Chunk), `v8_code_cache` (V8 JIT Code Cache), `mir_rust_dump` (Rustc Mid-level IR).
6. **计算生物学、基因组与序列对齐** ([`computational_biology_and_genetics.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/computational_biology_and_genetics.rs), 70 行)：
   `pizzly_fusion_json` (Pizzly RNA Fusion JSON), `plink_bed_binary` (PLINK Biallelic Genotype BED), `genbank_xml` (NCBI GenBank INSDSet XML), `embl_flat_file` (EMBL Nucleotide Flatfile), `gff3_annotation` (Generic Feature Format v3), `gtf2_annotation` (Gene Transfer Format v2), `clustal_alignment` (ClustalW Multiple Alignment), `stockholm_alignment` (Pfam/Rfam Stockholm Alignment), `nexus_phylogenetic` (NEXUS Phylogenetic), `newick_tree_file` (Newick Phylogenetic Tree), `two_bit_dna` (UCSC 2bit Genome Sequence), `bigbed_annotation` (UCSC BigBed Binary Annotation), `vcf_variant_text` (VCF v4.x Variant Call).
7. **地理信息栅格与数字高程模型** ([`geographic_information_rasters.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/geographic_information_rasters.rs), 70 行)：
   `dted_elevation` (NGA DTED Digital Terrain Elevation), `erdas_imagine_raw` (ERDAS Imagine LAN/GIS), `bip_binary_raster` (BIP Band Interleaved by Pixel), `ecw_compressed` (ER Mapper ECW Wavelet), `grass_gis_raster` (GRASS GIS Raster Header), `surfer_grid_ascii` (Golden Software Surfer DSAA), `surfer_grid_bin6` (Surfer v6 Binary DSBB), `surfer_grid_bin7` (Surfer v7 Binary BRSG), `gdal_vrt_xml` (GDAL Virtual Raster VRT), `pcidsk_database` (PCI Geomatics PCIDSK), `idrisi_raster_doc` (Clark Labs Idrisi Raster Doc), `esri_ascii_grid` (ESRI ASCII Raster Grid), `terragen_terrain` (Planetside Terragen Terrain).
8. **专业音频DSP、宿主预设与乐器采样** ([`audio_production_and_dsp.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/audio_production_and_dsp.rs), 70 行)：
   `soundfont_3` (MuseScore SoundFont 3 Vorbis), `korg_ksc_script` (KORG Script File), `yamaha_tyros_voice` (Yamaha Tyros Voice UVN), `soundfont_sfz` (Cakewalk SFZ Instrument), `ableton_preset_adv` (Ableton Live Preset ADV), `steinberg_vstpreset` (Steinberg VST3 Preset), `audiomulch_document` (AudioMulch Document AMP), `logic_pro_project_legacy` (Apple Logic Pro LOGX), `pro_tools_session_pts` (Avid Pro Tools Session PTS), `reaper_peak_file` (Cockos REAPER Peak File), `audacity_aup_classic` (Audacity Classic Project AUP), `renoise_song_xml` (Renoise Song XML), `bitwig_preset_bw` (Bitwig Studio Preset BWPRESET).
9. **经典办公套件与文档表单** ([`office_productivity_and_forms.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/office_productivity_and_forms.rs), 70 行)：
   `lotus_123_wk4` (Lotus 1-2-3 WK4), `lotus_123_wk1` (Lotus 1-2-3 WK1), `quattro_pro_wq1` (Borland Quattro Pro WQ1), `corel_presentations_shw` (Corel Presentations SHW), `lotus_word_pro` (Lotus Word Pro LWP), `staroffice_calc_sdc` (StarOffice Calc SDC), `microsoft_works_wps` (Microsoft Works Spreadsheet), `adobe_xd_manifest` (Adobe XD Project Manifest), `scribus_document` (Scribus Desktop Publishing SLA), `adobe_framemaker_fm` (Adobe FrameMaker FM), `adobe_indesign_idml_manifest` (Adobe InDesign IDML Package), `quark_xpress_qxd` (QuarkXPress QXD), `claris_works_cwk` (Apple/ClarisWorks CWK).
10. **操作系统内核转储与崩溃报告** ([`operating_system_crash_dumps.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/operating_system_crash_dumps.rs), 70 行)：
    `macos_panic_report` (macOS Kernel Panic JSON), `linux_vmcore_elf_note` (Linux Kexec/Kdump Vmcoreinfo), `solaris_savecore_dump` (Oracle/Sun Solaris Savecore Dump), `aix_core_dump` (IBM AIX Core Dump), `freebsd_minidump` (FreeBSD Kernel Minidump), `netbsd_core_dump` (NetBSD Core Dump), `hpux_crash_dump` (HP-UX Crash Dump), `vxworks_core_dump` (Wind River VxWorks Core), `qnx_core_dump` (BlackBerry QNX Neutrino Core), `openvms_crash_dump` (VSI OpenVMS Crash Dump), `apple_spin_dump` (Apple SpinDump Report), `android_tombstone` (Android Native Crash Tombstone), `systemd_coredump` (systemd coredump Metadata).
11. **光盘刻录映像与母盘工程** ([`optical_disc_mastering.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/optical_disc_mastering.rs), 70 行)：
    `clonecd_ccd` (SlySoft CloneCD CCD), `alcohol_mds` (Alcohol 120% Media Descriptor MDS), `nero_nrg2` (Nero Burning ROM NRG2), `blindwrite_b5t` (VSO BlindWrite 5 B5T), `blindwrite_b6t` (VSO BlindWrite 6 B6T), `discjuggler_cdi` (Padus DiscJuggler CDI v2), `cdrwin_cue` (Golden Hawk CDRWIN CUE Sheet), `cdrdao_toc` (cdrdao CD-DA Table of Contents), `gear_mastering_p01` (GEAR Optical Mastering P01), `plex_tools_pxi` (Plextor PlexTools PXI), `virtual_cd_vc4` (Virtual CD v4 VC4), `gamejack_xmd` (GameJack XMD), `padus_cdi_v3` (Padus DiscJuggler CDI v3).
12. **现代与嵌入式数据库存储引擎** ([`database_storage_engines.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/database_storage_engines.rs), 70 行)：
    `mysql_innodb_ibd` (MySQL InnoDB Tablespace IBD), `rocksdb_sst_file` (RocksDB SSTable File), `postgresql_base_block` (PostgreSQL Base Block), `firebird_fdb` (Firebird Relational Database), `redis_dump_rdb9` (Redis RDB v9 Dump), `etcd_wal_segment` (CoreOS etcd Raft WAL), `cassandra_sstable_toc` (Apache Cassandra SSTable TOC), `couchdb_couch_file` (Apache CouchDB Storage File), `neo4j_store_db` (Neo4j Graph Record Store), `clickhouse_bin_column` (ClickHouse Compressed Column), `scylladb_sstable` (ScyllaDB SSTable Storage), `lmdb_data_mdb` (Symas LMDB Memory Database), `leveldb_log` (Google LevelDB Record Log).
13. **汽车电子与车载总线协议** ([`embedded_automotive_buses.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/embedded_automotive_buses.rs), 70 行)：
    `can_candbc_v2` (Vector CANdb++ v2.0), `lin_protocol_cfg` (LIN Protocol Cluster Config), `fibex_xml` (ASAM FIBEX Field Bus Exchange), `autosar_arxml` (AUTOSAR System Description ARXML), `vector_asc_log` (Vector CANalyzer ASC ASCII Log), `kvaser_memorator_kme` (Kvaser Memorator Binary Log), `peak_can_trc` (PEAK-System PCAN Trace File), `can_open_eds` (CiA CANopen Electronic Data Sheet), `can_open_dcf` (CiA CANopen Device Configuration File), `canmatrix_sym` (PEAK PCAN Symbol File), `candump_log` (Linux SocketCAN candump Log), `asam_mdf2` (ASAM MDF 2.0 Measurement File), `asam_mdf3` (ASAM MDF 3.3 Measurement File).
14. **计算机代数、符号计算与数理统计** ([`mathematics_and_computer_algebra.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/mathematics_and_computer_algebra.rs), 70 行)：
    `mathematica_notebook` (Wolfram Mathematica Notebook), `maple_worksheet_mw` (Maplesoft Maple Worksheet), `maxima_wxmx_manifest` (wxMaxima Document Header), `mathcad_xmcd` (PTC Mathcad XML Worksheet), `r_workspace_rdata` (R Project Workspace RDA7), `stata_dta_118` (Stata v118 Dataset DTA), `spss_sav_system` (IBM SPSS Statistics SAV), `minitab_mtw` (Minitab Statistical Worksheet MTW), `geogebra_ggb_xml` (GeoGebra Interactive Math XML), `sage_math_sobj` (SageMath Serialized Object), `pari_gp_script` (PARI/GP Number Theory Script), `gap_groups_script` (GAP Computational Discrete Algebra), `coq_proof_v` (Coq Proof Assistant Script).
15. **经典复古游戏主机与模拟器** ([`retro_gaming_and_emulators.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/retro_gaming_and_emulators.rs), 70 行)：
    `amiga_hdf_hardfile` (Commodore Amiga Hardfile HDF), `vectrex_game_rom` (GCE Vectrex Vector ROM), `wonderswan_wsc_rom` (Bandai WonderSwan Color ROM), `neogeo_pocket_ngp` (SNK NeoGeo Pocket ROM), `dreamcast_gdi` (Sega Dreamcast Gigabyte Disc Image), `sega_saturn_cue` (Sega Saturn CD-ROM CUE), `gamecube_iso_gcm` (Nintendo GameCube GCM Image), `wii_iso_wbfs` (Nintendo Wii WBFS Image), `ps2_npo_memory_card` (Sony PlayStation 2 NPO Memory Card), `gamecube_gcm_raw` (GameCube Raw Disc Image), `msx_rom_cas` (MSX Cassette Tape CAS), `zx81_p_tape` (Sinclair ZX81 P File Tape), `sharp_x68000_xdf` (Sharp X68000 Floppy Disk XDF).

### 1.3 全局扩展名索引与测试套件
- **全局扩展名索引表** ([`src/extension/registry.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/extension/registry.rs))：
  - 条目数由 811 项一举跃升至 **1006 项**；
  - 严格保持单调递增 ASCII 字典排序，0 重复项；
- **全套自动化测试套件** ([`tests/test_batch_1000_milestone.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/tests/test_batch_1000_milestone.rs))：
  - 新增专用端到端回归测试，覆盖金融、网络、虚拟化、密码学、编译器、生物学、栅格、音频、办公、转储、母盘、数据库、汽车电子、数学、复古主机等 15 个垂直领域的 4 组 27 个代表性用例全部通过。

---

## 2. [当前状态] (Current State)

- **本地测试状态**：
  - 全套 49 个测试套件，**187+ 个测试用例全部通过 (187+/187+ PASS)**；
  - `cargo clippy --all-targets -- -D warnings` **0 警告，0 错误**；
  - `src/detectors/rules/generated/` 下全量 62 个文件行数严格分布在 **60~80 行**区间（100% 严守 50~80 行规范）；
- **远端 CI 状态**：
  - 前序提交 `5b43e84`（810 项）Ubuntu/macOS/Windows 三平台 CI 绿标全过。

---

## 3. [待办与下一步] (Next Steps)

1. **原子提交并推送到远端主线**：
   - 提交信息：`feat(detectors): expand format coverage to 1000+ formats across 59 partitions`；
   - 推送至 `origin/main`，监控 GitHub Actions CI 三端测试通过。
2. **向用户汇报达成 1000+ 格式里程碑**。
