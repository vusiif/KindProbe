# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-23  
> **当前版本**：v0.1.0 (格式支持跨越式突破 672 项大关！全量 35 个规则分卷严密收敛于 59~79 行黄金规范，44 大测试套件 179 用例全绿，Clippy 0 警告)  
> **远端分支**：`origin/main` (前次提交 `59c7e7b` GitHub Actions CI 全绿通过)

---

## 1. [已完成] (Completed)

### 1.1 扩展架构至 34 个垂直分卷与 672 项格式支持
全新增加 5 个专业领域分卷（深空天文学、超高清视音频裸流、现代地理空间CAD、专业排版字体、实时嵌入式固件镜像），将格式覆盖推向 **672 项**，`src/detectors/rules/generated/mod.rs` 紧凑维持在 72 行黄金规范区间：
- **深空天文学与轨道动力学分卷** ([`src/detectors/rules/generated/astronomy_and_space.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/astronomy_and_space.rs), 69 行)：
  1. `spk`: NASA SPICE 行星与航天器星历二进制 (`DAFT/SPK`, `.spk`, `.bsp`)
  2. `asdf`: 高级科学数据格式 (JWST 韦伯太空望远镜主格式, `#ASDF 1.0.0`, `.asdf`)
  3. `votable`: IVOA 国际虚拟天文台 XML (`<VOTABLE `, `.vot`, `.votable`)
  4. `tle`: NORAD 两行轨道要素星历 (`1 25544U 98067A`, `.tle`)
  5. `oem`: CCSDS 航天器轨道星历报文 (`CCSDS_OEM_VERS = 2.0`, `.oem`)
  6. `aem`: CCSDS 航天器姿态星历报文 (`CCSDS_AEM_VERS = 1.0`, `.aem`)
  7. `qso`: ADIF 业余无线电通联日志 (`<ADIF_VER:`, `.adi`)
  8. `rinex_obs`: 国际 GNSS 卫星导航观测数据 (`OBSERVATION DATA`, `.rnx`)
  9. `rinex_nav`: 国际 GNSS 卫星导航广播星历 (`NAVIGATION DATA`, `.nav`)
  10. `sp3`: IGS 精密卫星星历轨道格式 (`#cP`, `.sp3`)
  11. `sinex`: IGS 空间测地解决方案独立交换格式 (`%=SNX 2.00`, `.snx`)
  12. `antex`: 卫星天线相位中心标定格式 (`ANTEX`, `.atx`)
  13. `clk`: IGS 卫星与测站高精度钟差数据 (`CLOCK DATA`, `.clk`)
- **视音频裸流与软件无线电分卷** ([`src/detectors/rules/generated/video_streams_and_rf.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/video_streams_and_rf.rs), 69 行)：
  14. `h265`: H.265 / HEVC 视频裸流 (`\x00\x00\x00\x01\x40\x01`, `.h265`, `.hevc`)
  15. `h264`: H.264 / AVC 视频裸流 (`\x00\x00\x00\x01\x67`, `.h264`, `.264`)
  16. `vc1`: SMPTE VC-1 高级视频裸流 (`\x00\x00\x01\x0f`, `.vc1`)
  17. `ac4`: Dolby AC-4 新一代沉浸式音频裸流 (`\xac\x40`, `.ac4`)
  18. `eac3`: Dolby Digital Plus 增强音频流 (`\x0b\x77`, `.eac3`)
  19. `truehd`: Dolby TrueHD 无损环绕声音频流 (`\xf8\x72\x6f\xba`, `.thd`)
  20. `mlp`: Meridian Lossless Packing DVD-Audio 音频流 (`\xf8\x72\x6f\xbb`, `.mlp`)
  21. `avs`: AVS / AVS+ 中国国家音视频标准裸流 (`\x00\x00\x01\xb0`, `.avs`)
  22. `avs2`: AVS2 4K 超高清第二代视频标准裸流 (`\x00\x00\x01\xd0`, `.avs2`)
  23. `sdp`: 会话描述协议文本 (`v=0\r\no=`, `.sdp`)
  24. `sigmf`: 软件无线电 Signal Metadata 格式 (`{\n  "global": {`, `.sigmf`)
  25. `cfile`: GNU Radio 32位单精度复数 I/Q 采样 (`\x00\x00\x80\x3f...`, `.cfile`)
  26. `w64`: Sony Wave64 64位音频容器 (`riff\x2e\x91...`, `.w64`)
- **现代地理矢量与三维工程模型分卷** ([`src/detectors/rules/generated/geospatial_vector_cad.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/geospatial_vector_cad.rs), 69 行)：
  27. `gml`: OGC 地理标记语言 (`<gml:FeatureCollection `, `.gml`)
  28. `pbf`: OpenStreetMap 协议缓冲区二进制 (`\x00...\x0dOSMHeader\x08`, `.pbf`)
  29. `wor`: MapInfo 矢量制图工作空间脚本 (`!workspace`, `.wor`)
  30. `apr`: ESRI ArcView 3 经典 GIS 工程文件 (`/3.0\n(ODB.1`, `.apr`)
  31. `qgs`: QGIS 现代 XML 地理工程文件 (`<qgis `, `.qgs`)
  32. `dgn`: Bentley MicroStation V7 CAD 矢量文件 (`\x08\x05\x19\x00`, `.dgn`)
  33. `landxml`: LandXML 土木水利测绘交换数据 (`<LandXML `, `.landxml`)
  34. `citygml`: CityGML 城市三维多细节层次模型 (`<CityModel `, `.citygml`)
  35. `cpg`: ESRI Shapefile 字符编码页定义文件 (`UTF-8`, `.cpg`)
  36. `prj`: ESRI Shapefile 空间坐标参考系投影文本 (`GEOGCS[\"`, `.prj`)
  37. `e00_cov`: ESRI Arc/Info 拓扑覆盖数据 (`ARC  2`, `.cov`)
  38. `gxt`: Geotext 矢量地理数据 (`GXT`, `.gxt`)
  39. `gpx_trk`: 户外 GPS 航迹日志 (`<gpx `, `.trk`)
- **排版字形与字表设计分卷** ([`src/detectors/rules/generated/fonts_and_typography.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/fonts_and_typography.rs), 69 行)：
  40. `sfd`: FontForge 字体设计轮廓源码 (`SplineFontDB: `, `.sfd`)
  41. `vfb`: FontLab Studio 经典字体数据库 (`VFB `, `.vfb`)
  42. `ufo`: Unified Font Object 字形设计工程 (`<ufo `, `.ufo`)
  43. `pfm`: PostScript Type 1 打印机字体公制 (`\x00\x01\x00\x00\x00`, `.pfm`)
  44. `ttx`: AFDKO XML 格式 OpenType 字体转储 (`<ttFont sfntVersion=`, `.ttx`)
  45. `gf`: TeX Generic Font 通用点阵字形 (`\xf7\x83`, `.gf`)
  46. `vf`: TeX Virtual Font 虚拟字体定义 (`\xf7\xf6`, `.vf`)
  47. `spd`: Bitstream Speedo 矢量字形格式 (`Speedo`, `.spd`)
  48. `ch8`: CHIP-8 虚拟机字形与字节码程序 (`\x12\x00\x00\x00`, `.ch8`)
  49. `chr`: Borland BGI 矢量字符集字体 (`PK\x08\x08BGI `, `.chr`)
  50. `pkf`: TeX Packed Font 压缩光栅字体 (`\xf7\x59`, `.pkf`)
  51. `vpl`: TeX Virtual Property List 虚拟字体属性 (`(VTITLE `, `.vpl`)
  52. `bgi`: Borland BGI 图形设备驱动与字体包 (`pk\x08\x08BGI `, `.bgi`)
- **嵌入式与固件镜像分卷** ([`src/detectors/rules/generated/embedded_rtos_images.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/embedded_rtos_images.rs), 69 行)：
  53. `uf2`: Microsoft USB Flashing Format 固件烧录镜像 (`\x55\x46\x32\x0a\x57\x51\x5d\x9e`, `.uf2`)
  54. `esp32_img`: 乐鑫 ESP32 / ESP8266 Flash 固件镜像 (`\xe9\x01\x02\x00`, `.esp32`)
  55. `itb`: U-Boot Flattened Image Tree 现代化 FIT 固件镜像 (`\xd0\x0d\xfe\xed\x00\x00\x01`, `.itb`)
  56. `qnx_ifs`: QNX IFS 引导映像 (`\xeb\x7e\xff\x00\x01`, `.qnx`)
  57. `ti_txt`: 德州仪器 MSP430 固件烧录格式 (`@F000\r\n`, `.titxt`)
  58. `ubifs`: Linux UBIFS 闪存文件系统镜像 (`UBI#`, `.ubi`, `.ubifs`)
  59. `bcm_trx`: 华硕/博通双重 TRX 固件 (`HDR1`, `.trx2`)
  60. `dlink_fw`: D-Link 路由器固件镜像 (`SHRS`, `.dlk`)
  61. `netgear_fw`: Netgear 路由器固件包 (`2.00\x00\x00\x00\x00`, `.chk`)
  62. `openwrt_trx`: OpenWrt 路由器固件包 (`OWRT`, `.owrt`)
  63. `jlink_hex`: SEGGER J-Link 固件包 (`SEGGER`, `.jlink`)
  64. `allwinner_egon`: 全志 Allwinner eGON 启动引导头 (`eGON.BT0`, `.egon`)
  65. `intel_conf`: Intel 固件描述符区域 (`\x5a\xa5\xf0\x0f`, `.ifd`)
- **全局扩展名映射表** ([`src/extension/registry.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/extension/registry.rs))：
  - 严格扩充至 **672 项**，单调递增 ASCII 字典排序，0 重复项。
- **全套自动化回归测试套件** ([`tests/test_batch_670_multidisciplinary.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/tests/test_batch_670_multidisciplinary.rs))：
  - 新增专用端到端测试，全面覆盖 5 大垂直分卷核心格式，并验证扩展名双向匹配状态 (`ExtensionStatus::Match`)。

---

## 2. [当前状态] (Current State)

- **本地测试状态**：
  - 44 个测试套件，179 个测试用例，**100% 通过 (179/179 PASS)**；
  - `cargo clippy --all-targets -- -D warnings` **0 警告，0 错误**；
  - `src/detectors/rules/generated/` 下全量 35 个分卷文件行数严格分布在 **59~79 行**区间（严守 50~80 行铁律）；
- **远端 CI 状态**：
  - 前序提交 `59c7e7b`（604 项）在 GitHub Actions（macOS、Ubuntu、Windows 三端矩阵）成功全部通过。

---

## 3. [待办与阻塞] (Blockers & Next Steps)

- **阻塞项**：无阻塞。
- **下一步动作**：
  1. 执行 Git 原子提交并推送至远程仓库（`feat(detectors): expand format coverage to 672 formats across 34 partitions`）；
  2. 启动【批次 2】冲刺（672 -> 740 项），随后推进【批次 3】达成 800 项总目标！
