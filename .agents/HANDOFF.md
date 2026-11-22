# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-22  
> **当前版本**：v0.1.0 (扩展名索引跃升至 338 项，地理/流媒体/现代资产全面就绪，32 大测试套件 146 用例全绿)  
> **远端分支**：`origin/main`

---

## 1. [已完成] (Completed)

### 1.1 扩展支持 24+ 类核心格式与结构化文本方言（全局覆盖 338 项）
- **地理与标记结构化文本**：
  - `gpkg` (OGC GeoPackage)：在 SQLite 探测器中识别偏移量 68 处 application_id (`GPKG`/`GP10`)，推荐扩展名直出 `.gpkg`，标记 `sqlite3` 容器关系；
  - `svg` / `kml` / `gpx` / `dae` / `plist`：在 XML 网关中快速消歧，识别矢量图、地理标记、GPS 航迹、COLLADA 3D 与苹果属性列表；
  - `geojson`：在 JSON 网关中基于 Feature/Geometry 结构识别，输出推荐 `.geojson`；
- **流式解压复合嵌套**：
  - `svgz`：在 Gzip 流式就地解压前 512 字节中精准识别内嵌 `<svg`，输出证据链并推荐 `.svgz`；
- **广播流媒体与工业现代资产**：
  - `ts` / `m2ts`：基于 188 / 192 字节固定步长的 0x47 同步字节精确识别 MPEG 传输流与 BDAV 流；
  - `usdc` / `u3d`：覆盖 Pixar USD 二进制 Crate 格式与 Universal 3D 场景；
  - `vtf` / `pkm`：支持 Valve Source 引擎纹理与移动端 ETC 纹理；
- **现代字节码、专用归档与磁盘**：
  - `beam` / `pyc`：支持 Erlang/Elixir BEAM 虚拟机字节码与 Python 预编译字节码；
  - `arc` / `alz`：支持 FreeArc 与 ALZip 压缩归档；
  - `daa` / `ecm`：支持 Direct-Access-Archive 与 Error Code Modeler 镜像；
  - `sshkey` / `csr` / `bcfks`：精准消歧 OpenSSH 私钥、CSR 证书签名请求，PEM 作为低优先级兜底。

### 1.2 全局扩展名索引表扩充至 338 项 (`src/extension/registry.rs`)
- 严格按 ASCII 升序排列，通过 `test_registry_is_strictly_sorted_without_duplicates` 守卫测试；
- 所有源文件严格控制在 **50~80 行**黄金规范区间内。

### 1.3 专项集成测试 (`tests/test_geospatial_and_multimedia.rs`)
- 新增 4 大测试组，累计 32 个测试套件，**146 个测试用例全部 PASS**；
- `cargo clippy --all-targets -- -D warnings` 保持绝对 0 告警。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **代码质量与静态检查**：`cargo clippy --all-targets -- -D warnings` 严格零警告 (0 Errors, 0 Warnings)。
- **测试覆盖率与基线**：全量 `cargo test --all-targets` 全部通过 (146/146 PASS)。
- **Git 仓库状态**：准备提交并推送验证 CI。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无。
- **下一步动作**：
  1. 执行原子提交并推送到 GitHub 远程仓库；
  2. 监控 GitHub Actions 确认三大平台 (Ubuntu, macOS, Windows) 全绿。
