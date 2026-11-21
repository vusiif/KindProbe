# KindProbe

> 独立离线文件真实格式识别工具与 Rust 库（依据内容证据识别格式，推荐规范扩展名）。

**当前开发阶段：P1-A（最小可发布版 API / JSON v1 / CLI / 预算 / 错误与诊断稳定契约）**  
**项目许可证：GNU General Public License v3.0 或任何更新版本（SPDX：`GPL-3.0-or-later`），详见 [LICENSE](LICENSE)。第三方依赖与数据源保留各自许可证，见下文及 `research/`。**

---

## 1. 项目定位与边界

KindProbe 是一个离线、本地、只读的 Rust CLI 与核心库（依规格 `docs/FILE_FORMAT_RUST_SPEC_V1_2.md` 实现）：
- **内容优先**：识别完全基于文件内部结构与魔数证据，绝不依赖扩展名做身份确认。
- **扩展名诊断分离**：扩展名仅在内容识别后独立执行匹配诊断（`match` / `mismatch` / `missing` / `unrecognized` / `undetermined`），错误后缀不改变内容结论。
- **句柄级安全核验**：`detect_path` 先打开 `File` 句柄，再在已打开句柄上执行 `fstat`（`file.metadata()?.is_file()`）核验普通文件，拒绝目录、管道与设备，杜绝路径式检查与打开之间的 TOCTOU 竞态。
- **严格资源预算**：所有读取均经过 `BudgetReader` 计数适配器，对头窗（4 KiB）、总源读取（8 MiB）、ZIP 目录窗口（1 MiB）、容器条目数（10,000）、单 XML 解压输出（256 KiB）、总解压输出（1 MiB）、XML 嵌套深度（64）等关键资源设防，杜绝内存耗尽、拒绝服务与 XML 炸弹。进程级 Docker/OS 时间与内存护栏仅作为测试保护，不虚报为库内置保证。
- **拒绝伪确认**：严禁仅凭 `word/` 目录或文件名判定 DOCX；严禁仅凭 CFB 流名判定 DOC（必须验证 MS-DOC FIB `0xA5EC`、`nFib >= 0x00C1` 及表流存在性）。未实际核验的内部结构绝不虚报证据。
- **库无副作用**：Rust 核心库不打印标准输出、不写文件、不联网、不启动外部异步运行时。
- **明确不做的功能**：不修改扩展名、不重命名、不提取正文、不解压到磁盘、不进行宏执行或安全查毒、不进行无界递归探测、不集成 Toss 私有类型。

---

## 2. 格式能力与未支持矩阵 (Capability & Support Matrix)

| 格式名称 | 格式 ID (`format_id`) | 容器 ID (`container_id`) | 状态 | 验证手段与结构检查限制 |
|---|---|---|---|---|
| **Portable Network Graphics** | `png` | `null` | **Supported** | 8 字节头部签名 + IHDR 块有效性验证；仅有 8 字节头时仅给出 `signature` 证据，不虚报 `IHDR`；不完整签名降级为 unknown |
| **JPEG 图像格式** | `jpeg` | `null` | **Supported** | `\xFF\xD8\xFF` (SOI) + 标记段（APP0/APP1/DQT/SOF）结构验证；仅有 SOI 时给出 `signature` 证据，段结构完整给出 `structure` 证据 |
| **GIF 图像格式** | `gif` | `null` | **Supported** | `GIF87a` / `GIF89a` 魔数 + 逻辑屏幕描述符宽高非 0 校验；异常尺寸降级为 `signature` 证据 |
| **Portable Document Format** | `pdf` | `null` | **Supported** | `%PDF-` BOF 前缀与签名验证；不执行全量交叉引用表校验与流解码 |
| **ZIP 归档格式** | `zip` | `zip` | **Supported** | EOCD 尾部搜索 + 中央目录完整性校验；在无任何 OOXML 部件时确认为纯 ZIP |
| **Microsoft Word Open XML** | `docx` | `zip` | **Supported** | ECMA-376 OPC 完整验证：`[Content_Types].xml` 内容类型 + `_rels/.rels` 包关系正规化解析 + 主部件实际存在 + `<w:document>` 命名空间根元素；支持 Transitional 与 Strict |
| **Microsoft Excel Open XML** | `xlsx` | `zip` | **Supported** | ECMA-376 OPC 完整验证：`spreadsheetml.sheet` 内容类型 + `<workbook>` 命名空间根元素；支持 Transitional 与 Strict |
| **Microsoft PowerPoint Open XML** | `pptx` | `zip` | **Supported** | ECMA-376 OPC 完整验证：`presentationml.presentation` 内容类型 + `<presentation>` 命名空间根元素；支持 Transitional 与 Strict |
| **Word 启用宏文档** | `docm` | `zip` | **Supported** | ECMA-376 OPC 验证：`ms-word.document.macroEnabled` 内容类型 + `<w:document>` 命名空间根元素 |
| **Excel 启用宏工作簿** | `xlsm` | `zip` | **Supported** | ECMA-376 OPC 验证：`ms-excel.sheet.macroEnabled` 内容类型 + `<workbook>` 命名空间根元素 |
| **PowerPoint 启用宏演示文稿** | `pptm` | `zip` | **Supported** | ECMA-376 OPC 验证：`ms-powerpoint.presentation.macroEnabled` 内容类型 + `<presentation>` 命名空间根元素 |
| **疑似 OOXML 未确认** | `null` | `zip` | **Container Only** | 存在 OOXML 特征但缺少关系或主部件时，降级为 `container_only`，不假报也不退为普通 ZIP |
| **Compound File Binary** | `null` | `cfb` | **Container Only** | OLE2 复合文件结构；扇区链循环保护；无 WordDocument 流时确认为纯容器 |
| **Microsoft Word 97-2003** | `doc` | `cfb` | **Supported** | 验证 CFB 容器 + 根目录 `WordDocument` 流 + MS-DOC FIB 头（`wIdent == 0xA5EC`, `nFib >= 0x00C1`）+ 表流验证；检测加密标志；虚假流/嵌套流降级为 `container_only` |
| **Microsoft Excel 97-2003** | `xls` | `cfb` | **Supported** | 验证 CFB 容器 + 根目录 `Workbook`/`Book` 流 + MS-XLS BIFF5/BIFF8 BOF 记录（`0x0809`，版本 `0x0500`/`0x0600`）|
| **Microsoft PowerPoint 97-2003** | `ppt` | `cfb` | **Supported** | 验证 CFB 容器 + 根目录 `PowerPoint Document` 流 + 32 字节结构头 + `Current User` 流联动 |
| **7-Zip 归档格式** | `7z` | `7z` | **Supported** | 6 字节魔数签名 + 32 字节起始头 (StartHeader) CRC32 完整性校验 |
| **RAR 归档格式** | `rar` | `rar` | **Supported** | 严格区分并核验 RAR4（主档案头 `0x73`）与 RAR5（8 字节扩展签名）结构 |
| **MP4 视频格式** | `mp4` | `isobmff` | **Supported** | ISO-BMFF box 结构解析：验证 `ftyp` box 长度与主要品牌（`isom`/`mp41`/`mp42`/`m4v` 等）|
| **QuickTime MOV 视频** | `mov` | `isobmff` | **Supported** | 验证 `ftyp` 中的 `qt  ` 品牌或经典 `moov`/`wide`/`free` 文件头结构 |
| **WebM 开放视频格式** | `webm` | `matroska` | **Supported** | EBML 头部签名 (`1A 45 DF A3`) + 深度解析 `DocType` 为 `"webm"` |
| **Matroska MKV 视频** | `mkv` | `matroska` | **Supported** | EBML 头部签名 (`1A 45 DF A3`) + 解析 `DocType` 为 `"matroska"` |
| **WAV 音频格式** | `wav` | `riff` | **Supported** | RIFF 容器头部 + `WAVE` 格式标志 + `fmt ` 核心音频格式块结构核验 |
| **MP3 音频格式** | `mp3` | `null` | **Supported** | 支持 ID3v2 容器标签与 synchsafe 尺寸校验，或直接核验 MPEG-1/2 Audio Layer III 帧同步字 (`0xFF 0xFB/0xF3`) |
| **WebP 图像格式** | `webp` | `riff` | **Supported** | RIFF 容器头部 + `WEBP` 格式标志 + `VP8 `/`VP8L`/`VP8X` 图像数据块核验 |
| **Linux/Unix ELF 二进制** | `elf` | `null` | **Supported** | `\x7FELF` 魔数 + 32/64 位类、字节序、版本及目标文件类型 (`ET_EXEC`/`ET_DYN`/`ET_REL`) 结构核验 |
| **Windows Portable Executable** | `pe` | `null` | **Supported** | DOS `MZ` 头部 + `e_lfanew` 指针与 `PE\0\0` 签名核验；自动按 COFF 特征区分 EXE 与 DLL |
| **WebAssembly 二进制格式** | `wasm` | `null` | **Supported** | `\0asm` 魔数 + 4 字节版本号 (`0x01\0\0\0`) 结构校验 |
| **macOS Mach-O 二进制** | `macho` | `null` | **Supported** | 32/64 位单架构及通用 FAT 二进制魔数；严格与 Java `.class` 字节码消歧 |
| **Java Class 字节码** | `class` | `null` | **Supported** | `0xCAFEBABE` 魔数 + 主次版本号结构核验 |
| **EPUB 电子书** | `epub` | `zip` | **Supported** | 验证 ZIP 容器未压缩 `mimetype` 条目 (`application/epub+zip`) 与 `META-INF/container.xml` 结构 |
| **Mobipocket / Kindle 电子书** | `mobi` | `palmdoc` | **Supported** | Palm Database 头部结构 + `BOOKMOBI` / `BOOKTEXt` 标识核验 |
| **OpenDocument 格式** | `odt`/`ods`/`odp` | `zip` | **Supported** | 验证 ZIP 容器 `mimetype` 条目（`opendocument.text` / `spreadsheet` / `presentation`）|
| **TrueType 字体** | `ttf` | `null` | **Supported** | `\0\x01\0\0` 魔数 + 字体表目录数量结构核验 |
| **OpenType 字体** | `otf` | `null` | **Supported** | `OTTO` 魔数 + 字体表目录数量结构核验 |
| **WOFF 1.0 网页字体** | `woff` | `null` | **Supported** | `wOFF` 魔数 + 头部表长度与目录数量结构核验 |
| **WOFF 2.0 网页字体** | `woff2` | `null` | **Supported** | `wOF2` 魔数 + 头部表结构核验 |
| **SQLite 3 数据库** | `sqlite3` | `null` | **Supported** | `SQLite format 3\0` 16 字节头部魔数 + 页大小及读写格式版本结构核验 |
| **PCAP 网络抓包转储** | `pcap` | `null` | **Supported** | 纳秒/微秒时间戳魔数（大小端序）+ 2.4 主次版本号结构核验 |
| **PCAPNG 下一代抓包格式** | `pcapng` | `null` | **Supported** | Section Header Block (`0x0A0D0D0A`) + 字节序魔数 (BOM) 核验 |
| **Apache Parquet 列式存储** | `parquet` | `null` | **Supported** | `PAR1` 头部魔数核验 |
| **Apache Avro 数据容器** | `avro` | `null` | **Supported** | `Obj\x01` 头部魔数核验 |
| **QEMU Copy-On-Write 镜像** | `qcow2` | `null` | **Supported** | `QFI\xFB` 魔数 + 版本 2/3 及聚簇位 (cluster_bits 9..21) 结构核验 |
| **VMware 虚拟磁盘** | `vmdk` | `null` | **Supported** | `KDMV` Sparse 二进制魔数与版本核验，或 `# Disk DescriptorFile` 文本描述符核验 |
| **微软虚拟硬盘 (VHDX)** | `vhdx` | `null` | **Supported** | `vhdxfile` 头部签名核验 |
| **微软虚拟硬盘 (VHD)** | `vhd` | `null` | **Supported** | `conectix` 头部签名核验 |
| **ISO 9660 光盘镜像** | `iso` | `null` | **Supported** | 扇区 16 (offset 0x8000) 主卷描述符 `\x01CD001` / `\x02CD001` 流探测核验 |
| **POSIX Tar 归档格式** | `tar` | `tar` | **Supported** | 512 字节头部块 + `ustar\0`/`ustar ` 签名 + 八进制校验和 (Checksum) 累加核验 |
| **Gzip 压缩容器** | `gzip` | `gzip` | **Supported** | RFC 1952 魔数 `\x1F\x8B` + DEFLATE 压缩方法 (8) 与保留标志位核验 |
| **Bzip2 压缩格式** | `bzip2` | `bzip2` | **Supported** | `BZh` 魔数 + 块大小 (`'1'`..`'9'`) 与 Pi 常数块特征 (`0x314159265359`) 核验 |
| **XZ 压缩格式** | `xz` | `xz` | **Supported** | `\xFD7zXZ\x00` 魔数 + Stream Flags 的 IEEE CRC32 完整性核验 |
| **Zstandard 压缩格式** | `zstd` | `zstd` | **Supported** | `0x28 0xB5 0x2F 0xFD` 头部魔数核验 |
| **JSON 文档** | `json` | `null` | **Supported** | 前置网关 BOM 剥离与控制字符过滤 + `{...}` / `[...]` 根语法结构核验 |
| **XML 标记语言** | `xml` | `null` | **Supported** | `<?xml` 处理指令或标准 XML 根标签与命名空间核验 |
| **HTML 网页文档** | `html` | `null` | **Supported** | `<!DOCTYPE html`、`<html`、`<head`、`<body` 核心标记结构核验 |
| **YAML 数据序列化** | `yaml` | `null` | **Supported** | `---` 文档起始标记或多行合规键值/列表结构模式核验 |
| **TOML 配置文件** | `toml` | `null` | **Supported** | `[section]` 标准表头或结构化赋值语句核验 |
| **CSV 逗号分隔表格** | `csv` | `null` | **Supported** | 多行逗号分隔列数完全一致性与行结构特征核验（>= 2 列） |
| **TSV 制表符分隔表格** | `tsv` | `null` | **Supported** | 多行制表符分隔列数一致性与行结构特征核验（>= 2 列） |
| **纯文本文件** | `txt` | `null` | **Supported** | 严格网关校验：UTF-8 编码合法性 + 零 `\0` 空字节 + 控制字符率严格低于 1% |
| **FLAC 无损音频** | `flac` | `null` | **Supported** | `fLaC` 魔数 + STREAMINFO 元数据块头校验 |
| **Ogg 多媒体容器** | `ogg` | `ogg` | **Supported** | `OggS` 容器魔数 + 头部版本及逻辑流校验（支持 Vorbis / Opus 等） |
| **Photoshop 图像文档** | `psd` | `null` | **Supported** | `8BPS` 魔数 + 版本 1 及保留字节校验 |
| **TIFF 图像格式** | `tiff` | `null` | **Supported** | 支持大端 `MM\0*` 与小端 `II*\0` 双变体魔数 + IFD 偏移校验 |
| **Windows 图标格式** | `ico` | `null` | **Supported** | `\0\0\x01\0` 魔数 + 图标目录项数量校验 |
| **Windows 光标格式** | `cur` | `null` | **Supported** | `\0\0\x02\0` 魔数 + 光标目录项数量校验 |
| **Debian 软件包** | `deb` | `ar` | **Supported** | 多段魔数联合校验：`!<arch>\n` + `debian-binary` 首条目签名 |
| **Red Hat 软件包** | `rpm` | `null` | **Supported** | `\xED\xAB\xEE\xDB` 魔数 + 主版本号校验 |
| **Chrome 扩展包** | `crx` | `null` | **Supported** | `Cr24` 魔数 + 版本号校验 |
| **Android 安装包** | `apk` | `zip` | **Supported** | 验证 ZIP 容器内部 `AndroidManifest.xml` 与 `classes.dex` 核心部件 |
| **iOS 安装包** | `ipa` | `zip` | **Supported** | 验证 ZIP 容器内部 `Payload/*.app/Info.plist` 应用清单结构 |
| **Java 归档包** | `jar` | `zip` | **Supported** | 验证 ZIP 容器内部 `META-INF/MANIFEST.MF` 清单文件 |
| **AAC 高级音频编码** | `aac` | `null` | **Supported** | ADTS 同步字魔数（`0xFF 0xF1` / `0xFF 0xF9`）结构核验 |
| **AIFF 音频格式** | `aiff` | `null` | **Supported** | 多段魔数联合校验：`FORM` 块 + `AIFF` / `AIFC` 标志 |
| **MIDI 音乐格式** | `mid` | `null` | **Supported** | `MThd` 4 字节魔数 + 头块长度 6 字节结构核验 |
| **已编译 HTML 帮助文档** | `chm` | `null` | **Supported** | `ITSF` 4 字节魔数 + 格式版本号核验 |
| **Flash 视频格式** | `flv` | `null` | **Supported (Codegen)** | `FLV\x01` 4 字节魔数核验 |
| **AMR 语音音频** | `amr` | `null` | **Supported (Codegen)** | `#!AMR\n` 6 字节魔数核验 |
| **Dolby AC-3 音频** | `ac3` | `null` | **Supported (Codegen)** | `\x0B\x77` 16 位同步字魔数核验 |
| **Sun/NeXT 音频** | `au` | `null` | **Supported (Codegen)** | `.snd` 4 字节魔数核验 |
| **Apple Core Audio (CAF)** | `caf` | `null` | **Supported (Codegen)** | `caff` 4 字节魔数核验 |
| **Dirac 视频流** | `drc` | `null` | **Supported (Codegen)** | `BBCD` 4 字节魔数核验 |
| **MNG 动画图像** | `mng` | `null` | **Supported (Codegen)** | `\x8aMNG\r\n\x1a\n` 8 字节魔数核验 |
| **GIMP XCF 图像** | `xcf` | `null` | **Supported (Codegen)** | `gimp xcf ` 9 字节魔数核验 |
| **OpenEXR 高动态图像** | `exr` | `null` | **Supported (Codegen)** | `\x76\x2F\x31\x01` 4 字节魔数核验 |
| **FITS 天文图像数据** | `fits` | `null` | **Supported (Codegen)** | `SIMPLE  =` 9 字节头部特征核验 |
| **Apple 图标 (ICNS)** | `icns` | `null` | **Supported (Codegen)** | `icns` 4 字节魔数核验 |
| **JPEG 2000 图像** | `jp2` | `null` | **Supported (Codegen)** | 12 字节标准魔数签名核验 |
| **BPG 图像格式** | `bpg` | `null` | **Supported (Codegen)** | `BPG\xFB` 4 字节魔数核验 |
| **Netpbm 图形格式** | `pbm`/`pgm`/`ppm` | `null` | **Supported (Codegen)** | `P4\n` (PBM), `P5\n` (PGM), `P6\n` (PPM) 魔数核验 |
| **DICOM 医学影像** | `dicom` | `null` | **Supported (Codegen)** | 偏移 128 处 `DICM` 4 字节魔数签名核验 |
| **PostScript 文档** | `postscript` | `null` | **Supported (Codegen)** | `%!PS-Adobe-` 头部签名核验 |
| **RTF 富文本格式** | `rtf` | `null` | **Supported (Codegen)** | `{\\rtf` 5 字节魔数核验 |
| **AutoCAD DWG 图纸** | `dwg` | `null` | **Supported (Codegen)** | `AC10` 4 字节版本头部魔数核验 |
| **CPIO 系统归档** | `cpio` | `null` | **Supported (Codegen)** | `070707` 6 字节魔数核验 |
| **Lzip 压缩格式** | `lz` | `null` | **Supported (Codegen)** | `LZIP` 4 字节魔数核验 |
| **ARJ 压缩归档** | `arj` | `null` | **Supported (Codegen)** | `\x60\xEA` 魔数核验 |
| **Zoo 归档格式** | `zoo` | `null` | **Supported (Codegen)** | 偏移 20 处 `\xDC\xA7\xC4\xFD` 4 字节魔数核验 |
| **Microsoft Cabinet 柜文件** | `cab` | `null` | **Supported (Codegen)** | `MSCF` 4 字节魔数签名核验 |
| **Blender 3D 项目** | `blend` | `null` | **Supported (Codegen)** | `BLENDER` 7 字节魔数核验 |
| **glTF 2.0 二进制模型** | `glb` | `null` | **Supported (Codegen)** | `glTF\x02\0\0\0` 8 字节魔数核验 |
| **Autodesk FBX 二进制** | `fbx` | `null` | **Supported (Codegen)** | `Kaydara FBX Binary  ` 20 字节魔数核验 |
| **3D Studio 几何网格** | `3ds` | `null` | **Supported (Codegen)** | `\x4D\x4D` 主块魔数核验 |
| **Polygon File Format** | `ply` | `null` | **Supported (Codegen)** | `ply\n` 魔数核验 |
| **STEP 3D CAD 模型** | `step` | `null` | **Supported (Codegen)** | `ISO-10303-21;` 头部特征核验 |
| **NES 游戏卡带镜像** | `nes` | `null` | **Supported (Codegen)** | `NES\x1a` 4 字节魔数核验 |
| **HDF5 层次化科学数据** | `hdf5` | `null` | **Supported (Codegen)** | `\x89HDF\r\n\x1a\n` 8 字节魔数核验 |
| **HDF4 科学数据容器** | `hdf4` | `null` | **Supported (Codegen)** | `\x0E\x03\x13\x01` 4 字节魔数核验 |
| **NetCDF 气象海洋数据** | `netcdf` | `null` | **Supported (Codegen)** | `CDF\x01` / `CDF\x02` 魔数核验 |
| **ESRI Shapefile 矢量地理** | `shp` | `null` | **Supported (Codegen)** | `0x0000270a` 大端魔数核验 |
| **WMO GRIB 气象网格数据** | `grib` | `null` | **Supported (Codegen)** | `GRIB` 4 字节魔数核验 |
| **MATLAB 5.0 数据文件** | `mat5` | `null` | **Supported (Codegen)** | `MATLAB 5.0 MAT-file` 头部特征核验 |
| **CERN ROOT 物理数据** | `root` | `null` | **Supported (Codegen)** | `root\0` 5 字节魔数核验 |
| **冷冻电镜密度图 (MRC)** | `mrc` | `null` | **Supported (Codegen)** | 偏移 208 处 `MAP ` 4 字节魔数核验 |
| **Commodore 64 芯片音乐** | `sid` | `null` | **Supported (Codegen)** | `PSID` / `RSID` 4 字节魔数核验 |
| **SNES SPC700 芯片音乐** | `spc` | `null` | **Supported (Codegen)** | `SNES-SPC700` 头部特征核验 |
| **Video Game Music (VGM)** | `vgm` | `null` | **Supported (Codegen)** | `Vgm ` 4 字节魔数核验 |
| **BitTorrent 种子元数据** | `torrent` | `null` | **Supported (Codegen)** | `d8:announce` Bencode 头部核验 |
| **Windows 映像镜像 (WIM)** | `wim` | `null` | **Supported (Codegen)** | `MSWIM\0\0\0` 8 字节魔数核验 |
| **StuffIt 压缩归档** | `sit` | `null` | **Supported (Codegen)** | `StuffIt` 7 字节魔数核验 |
| **SQX 压缩归档** | `sqx` | `null` | **Supported (Codegen)** | `SQX\0` 4 字节魔数核验 |
| **DjVu 高保真文档影像** | `djvu` | `null` | **Supported (Codegen)** | `AT&TFORM` 8 字节表单魔数核验 |
| **NITF 卫星与遥感成像** | `nitf` | `null` | **Supported (Codegen)** | `NITF` 4 字节版本头部核验 |
| **Microsoft MODI 扫描格式** | `mdi` | `null` | **Supported (Codegen)** | `\x45\x50\x2A\x00` 4 字节魔数核验 |
| **AutoCAD 二进制 DXF** | `dxf_bin` | `null` | **Supported (Codegen)** | `AutoCAD Binary DXF` 头部特征核验 |
| **Autodesk DWF 设计图纸** | `dwf` | `null` | **Supported (Codegen)** | `(DWF V` 头部特征核验 |
| **Microsoft Write 旧式文档** | `wri` | `null` | **Supported (Codegen)** | `\x31\xBE\x00\x00` 4 字节魔数核验 |
| **Xara 矢量绘图文档** | `xar` | `null` | **Supported (Codegen)** | `xar!` 4 字节魔数核验 |

**支持平台验证状态**：
- Linux x86_64 (`Debian trixie / Linux 6.6`): **Verified (PASS)**
- Windows x86_64 (`Windows 10/11`): **Verified (PASS)**
- macOS: **Not Verified (未验证)**

---

## 3. Rust API 快速上手

```rust
use kindprobe::{detect_path, detect_bytes, detect_reader, DetectOptions};
use std::path::Path;

// 1. 路径探测（安全打开句柄 + fstat 校验普通文件 + 独立扩展名诊断）
let options = DetectOptions::default();
let report = detect_path(Path::new("tests/sample.docx"), &options)?;
assert_eq!(report.format_id.as_deref(), Some("docx"));
assert_eq!(report.schema_version, 1);

// 2. 内存切片探测
let bytes = b"\x89PNG\r\n\x1a\n...";
let report_bytes = detect_bytes(bytes, &options)?;

// 3. Read + Seek 流探测
let mut cursor = std::io::Cursor::new(bytes);
let report_stream = detect_reader(&mut cursor, &options)?;
```

---

## 4. CLI 用法与退出码

### 4.1 基本命令
```bash
# 文本诊断输出
kindprobe document.docx

# 机器可读 JSON v1 输出
kindprobe --json document.docx

# 支持 -- 安全分隔以处理前导连字符文件名
kindprobe --json -- -document.docx

# 帮助与版本
kindprobe --help
kindprobe --version
```

### 4.2 退出码说明
| 退出码 | 含义 | 示例 |
|---|---|---|
| `0` | 格式确认 (`identification=format`) 且检查完整 (`inspection=complete`) | 校验合法的 PNG, DOCX, DOC, 普通 ZIP |
| `2` | 命令行参数错误 | 缺少文件路径、未知命令行选项 |
| `3` | 未达到完整格式确认 | 未知格式、仅容器确认 (`container_only`)、预算耗尽 (`limited`)、多候选冲突 (`ambiguous`) |
| `4` | 真实底层 I/O 错误或不支持的文件对象 | 文件不存在、无读取权限、目录、FIFO/管道、字符/块设备 |
| `1` | 非预期程序缺陷 | 内部异常 |

### 4.3 JSON v1 结构规范

正常识别输出信封（退出码 0 或 3）：
```json
{
  "schema_version": 1,
  "format_id": "docx",
  "category": "document",
  "container_id": "zip",
  "identification": "format",
  "verification": "structure",
  "inspection": "complete",
  "recommended_extension": "docx",
  "accepted_extensions": ["docx"],
  "extension_status": "match",
  "evidence": [
    {"code": "zip.central_directory"},
    {"code": "ooxml.content_types"},
    {"code": "ooxml.relationship.office_document"},
    {"code": "ooxml.word_main_part"},
    {"code": "ooxml.word_document_root"}
  ],
  "warnings": [],
  "stats": {
    "source_bytes_read": 16384,
    "seek_count": 4,
    "entries_inspected": 6,
    "decompressed_bytes": 2048,
    "duration_micros": 450
  }
}
```

CLI 错误输出信封（退出码 4 或 1，使用 `--json` 时输出）：
```json
{
  "schema_version": 1,
  "error": {
    "code": "io.not_regular_file",
    "message": "Path 'fixtures' is not a regular file (e.g. directory or device)"
  }
}
```

### 4.4 Windows 资源管理器交互与拖拽工作流 (Explorer Drag-and-Drop)

- **拖拽识别**：在 Windows 资源管理器中将单个文件拖拽到 `kindprobe.exe` 图标上，程序将检测由 Explorer 新分配的独立交互式控制台（仅宿主当前进程、标准输入输出为真实控制台缓冲区），完成格式识别后保持控制台开启，并显示 `Press Enter to close...` 提示，供用户阅读识别结论。用户敲击回车或直接关闭窗口均可安全退出。
- **双击运行**：在资源管理器中直接双击 `kindprobe.exe`（无文件参数），程序会在独立交互控制台打印简明拖拽使用指引（`Drag and drop a file onto kindprobe.exe to identify its format.`），同样显示回车关闭提示并保持窗口开启，退出码保持 `2`。
- **自动化无感兼容（硬契约）**：
  - 在既有终端环境（`cmd.exe` / `PowerShell` / `wt.exe` 等父外壳已挂接控制台时，`GetConsoleProcessList > 1`）、CI 流水线、无交互脚本或任何重定向/管道模式（`kindprobe.exe file | more`、`kindprobe.exe file > out.txt`）下，程序**绝不暂停**、**绝不打印回车提示**，维持既有标准输出/错误与退出码契约。
  - 在 `--json` 模式下，程序强制禁用控制台暂停逻辑，确保自动化与机器解析流程零阻塞。
- **限制说明**：
  - 仅支持单文件拖拽探测；若同时拖拽多个文件，程序输出标准参数错误（`multiple file arguments provided`）并在独立控制台保持可见。
  - 不支持目录递归探测，不修改磁盘文件，不集成 GUI 框架。
  - 此交互暂停与控制台宿主探测逻辑仅在 Windows 平台通过 `cfg(windows)` 原生生效；非 Windows 平台一律不触发。
  - 测试环境变量覆盖（`KINDPROBE_TEST_DEDICATED_CONSOLE`）：test-only override; when set to `1` the session is treated as a dedicated console (and `--json` still never pauses); unset = normal detection behavior. 仅供测试与仿真覆盖使用，未设置时严格遵循上述原生检测行为。

### 4.5 环境语言自动检测与中英文双语人读文本 (Automatic Locale Detection)

- **语言检测机制**：
  - **Windows 平台**：调用原生 Win32 `GetUserDefaultUILanguage()` 获取当前用户首选 UI 显示语言（含 `zh-CN`、`zh-TW`、`zh-HK`、`zh-SG`、`zh-MO`、`zh-Hans`、`zh-Hant` 等所有中文变体），自动切换为简体中文人读输出；非中文、未分配或 API 失败时安全回退为英文。不依赖时区、键盘布局、系统代码页、文件名或区域格式；不修改全局或控制台配置。
  - **Linux / POSIX 平台**：严格遵循 POSIX 本地化变量优先级（`LC_ALL` > `LC_MESSAGES` > `LANG`）；`zh` 变体语言标签激活中文呈现，`C`、`POSIX`、非中文、空值或未知变量回退为英文。
- **人读本地化范围**：
  - 命令行帮助标题/选项描述/退出码说明。
  - 独立控制台拖拽/无参数双击使用指引、退出前回车关闭提示（`按 Enter 键关闭...` / `Press Enter to close...`）。
  - 命令行参数错误与 I/O 诊断说明（如 `错误: 未识别的选项` / `错误 [io.open_failed]: 打开或读取文件 '...' 失败`）。
  - 格式识别结果、分类、验证方式、检测深度、扩展名状态描述、统计标签以及已知警告代码的中文释义。
- **机器契约与稳定性保证**：
  - 程序名（`kindprobe`）、命令行 flag（`--json`, `-h`, `--help` 等）、文件路径、格式 ID（`png`, `docx`, `doc` 等）、文件扩展名、稳定诊断码（如 `io.open_failed`, `cfb.table_stream_missing` 等）在任何语言下均严格保持原样。
  - `--json` 输出路径严格不受语言检测影响：schema、字段名、枚举值、诊断码与序列化内容完全不变，不输出任何横幅、提示或中文化文本。
  - 核心 DetectionReport / ErrorEnvelope 库契约与退出码（0/2/3/4/1）完全保持不变。
  - 采用轻量内置 catalog 映射，零引入外部大型 i18n 框架或网络依赖。

---

## 5. 依赖与第三方许可清单

| 依赖 crate | 版本 | 许可证 | 用途 |
|---|---|---|---|
| `zip` | 2.6.1 | MIT | ZIP 中央目录解析（受 `BudgetReader` 严格限制） |
| `quick-xml` | 0.36.2 | MIT | 流式 XML pull parser（限制深度 <= 64） |
| `cfb` | 0.10.0 | MIT | OLE2 扇区与流导航（附带 FIB 自研校验层） |
| `serde` | 1.0.229 | MIT / Apache-2.0 | 结构化模型序列化 |
| `serde_json` | 1.0.154 | MIT / Apache-2.0 | CLI JSON v1 输出格式 |

第三方数据源与调研工具：
- **Apache Tika**: Apache-2.0，官方 `tika-mimetypes.xml` 作为权威特征与扩展名数据库来源（见 `tools/data/`）。
- **PRONOM / DROID**: Open Government Licence 2.0 (OGL-v2.0)，仅作为离线对照基准，不直接打包未审查规则。
- **Siegfried**: Apache-2.0，仅作为测试期间独立复测对照工具。
- **Microsoft MS-DOC / MS-CFB**: 依据 Microsoft Open Specifications 技术公开规范查证实现。

### 5.1 离线代码生成管线 (Offline Codegen Pipeline)

项目内置零依赖的离线代码生成工具 `tools/codegen.py`，支持直接从权威数据库（如 Apache Tika `tika-mimetypes.xml`）自动提取静态声明式规则和规范扩展名注册表：
- **纯固定偏移与预算控制**：严格提取固定偏移且 `offset + len <= 4096` 的二进制魔数规则，完美适配 `BudgetReader` 读取窗口；
- **分卷模块化**：生成的声明式规则按大类输出至 `src/detectors/rules/generated/`（`media.rs`、`images.rs`、`documents.rs`、`archives.rs`），单文件保持在 35~70 行，零运行时堆分配（`&'static [SignatureRule]`）；
- **单一真实信源（SSOT）扩展名检索**：提取权威扩展名映射并在 `src/extension/registry.rs` 中按 ASCII 字典序严格升序排列，支持零哈希开销的 $O(\log N)$ 二分查找，并通过编译期与单元测试（`test_registry_is_strictly_sorted_without_duplicates`）守护排序严格性。

---

## 6. 构建与回归测试验证指南 (Clean Checkout)

在普通的 clean checkout 仓库下，全部测试与夹具均已受控保存在仓库中，不依赖任何 `.sandbox-test` 历史绝对路径。

### 6.1 离线 Docker 沙箱测试契约
- 镜像：`local/rust-test:1.99.1-trixie`
- 镜像 ID：`sha256:cdf54f8f43b6987e28d89c2d9fe0548bfdfbcd3db227b5ac20ca45e7f8330fe3`

### 6.2 执行测试命令
```bash
# 1. 运行核心集成测试（包含 9 项失败回归、CLI 契约、预算对抗、API 一致性）
docker run --rm --name kindprobe-test \
  --pull never --network none --read-only \
  --cap-drop ALL --security-opt no-new-privileges \
  --user $(id -u):$(id -g) --group-add 1004 \
  --cpus 2 --memory 2g --pids-limit 128 \
  --tmpfs /tmp:rw,nosuid,size=256m,mode=1777 \
  -v $(pwd):/workspace:ro \
  -v $(pwd)/.sandbox-test:/output:rw \
  local/rust-test:1.99.1-trixie \
  cargo test --offline --manifest-path /workspace/Cargo.toml

# 2. 运行仓库自带的 52 组正反/对抗矩阵回归
python3 scripts/run_52_fixtures_regression.py

# 3. 运行仓库自带的 Codex 18 项全量验收测试
python3 scripts/run_acceptance.py
```

### 6.3 本地全量回归与对抗模糊测试 (In-Repo Native Suite)
无需 Docker 环境，在任何受支持平台（Linux/Windows/macOS）直接运行：
```bash
# 运行全量 118 个集成与单元测试（覆盖 85+ 格式、真实样本句柄矩阵、全量对抗变异模糊）
cargo test
```
- **真实文件样本矩阵 (`fixtures/real/`)**：包含 25 个干净真实的跨领域格式切片（FLV, AMR, AC3, CAF, DRC, MNG, XCF, EXR, FITS, ICNS, JP2, BPG, Netpbm, DICOM, PostScript, RTF, DWG, CPIO, Lzip, Cabinet, SQLite3, PCAP, Parquet 等），通过 `test_real_fixtures_matrix` 执行端到端真实 OS 句柄与扩展名正反校验；
- **确定性对抗模糊套件 (`test_adversarial_fuzzing`)**：零外部依赖，使用确定性伪随机变异引擎执行数千次混沌随机流注入、魔数任意边界截断与随机位翻转，自动化守护 No-Panic、预算严格锁定与幂等不变量。
