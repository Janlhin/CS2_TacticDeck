# MANIFEST.md

第三方二进制、源码与逆向对象的来源登记。字段：名称 / 来源 URL / 版本 / SHA256 / 用途 / 许可结论。

**只登记仓库实际持有或计划引入的对象。** 尚未获取的，SHA256 写「未获取」并说明引入条件，不留空。

## 已持有

| 名称 | 来源 URL | 版本 | SHA256 | 用途 | 许可结论 |
|---|---|---|---|---|---|
| `LiteReplay-2.0-Editor4.zip`（源码发布包） | 官方源码仓库已公开：<https://github.com/MikaW21/LiteReplay>（MIT）。同源核对提交 `fc23672`（Release LiteReplay 2.0 Editor Preview 4） | 2.0 Editor Preview 4 | `74C747962F74FB81E586A192663E8685A4BF81E58B12FA47A0CF8603079C9400` | 抓屏与编码层的源码参考；同源核对 | MIT（可改、可分发，保留版权与许可声明） |
| `LiteReplay.exe`（发行包内二进制） | 同上（微信发行包，2026-11-15） | 2.0 Editor Preview 4 | `9592D7CBEAAFA668CA78F617F6DE1E0F964CA841687840321AD005FE41C474E7` | 与源码包做同源校验（哈希一致） | MIT |
| `LiteReplay_Source`（本地源码快照） | 由上述源码包解压而来；已与上游 `main` 文件树逐项比对，仅差上游后续新增的 6 张 README 截图 | 2.0 Editor Preview 4 | 211 文件 / 48314308 B；`SHA256SUMS` 209 行与上游逐字节一致 | 引擎行为的逐文件取证 | MIT（上游仓库公开后确认） |
| `source2-demo`（crate） | <https://docs.rs/source2-demo> | 0.5.11 | 未获取（由 `Cargo.lock` 锁定版本，未做本地哈希） | CS2 demo 解析（S0 主路径） | MIT OR Apache-2.0（二者择一） |

## 计划引入（尚未获取）

| 名称 | 来源 URL | 版本 | SHA256 | 用途 | 许可结论 |
|---|---|---|---|---|---|
| `@laihoe/demoparser2`（npm） | <https://github.com/LaihoE/demoparser> | 待定 | 未获取（引入时登记） | 解析兜底（sidecar） | MIT |
| HLAE（`advancedfx`） | <https://github.com/advancedfx/advancedfx> | 待定 | 未获取（作为外部工具调用，不内嵌） | demo 相机与逐 tick 命令 | MIT；**子模块另有许可，须逐项核对**；含 VAC 风险警告，仅限离线 demo |
| `cs2-map-icons`（overview 数据） | <https://github.com/MurkyYT/cs2-map-icons> | 待定 | 未获取 | 地图盘面校准（开发期） | 许可待核；雷达图为 Valve 资产，发布版改为本机提取 |

## 构建与运行时依赖（由 `package.json` / `Cargo.lock` 锁定）

本表只登记主依赖的许可结论；版本以锁文件为准，升级随锁文件变化，不在本表逐次追写。根 `Cargo.toml` 建立 workspace 后，`Cargo.lock` 随根目录走，成员为 `src-tauri` 与 `crates/tacticdeck-probe`。

| 名称 | 版本 | 用途 | 许可结论 |
|---|---|---|---|
| `tauri` + `tauri-build`（crate） | 2.12.x / 2.7.x | 桌面壳与构建脚本 | MIT OR Apache-2.0 |
| `winreg`（crate，仅 Windows） | 0.55.x | 读 Steam 安装路径以定位本机 CS2 | MIT OR Apache-2.0 |
| `react` / `react-dom` | 19.3.1 | 前端框架 | MIT |
| `zustand` | 5.1.0 | 状态层 | MIT |
| `pixi.js` | 8.23.0 | 2D 画布 | MIT |
| `vite` / `@vitejs/plugin-react` | 8.3.8 / 6.3.0 | 构建与开发服务器 | MIT |
| `typescript` | 7.0.2 | 类型检查 | Apache-2.0 |
| `sass` | 1.108.2 | SCSS 编译 | MIT |
| `@tauri-apps/api` | 2.12.4 | 前端调用 Rust 桥 | MIT OR Apache-2.0 |
| `markdownlint-cli2` / `markdown-link-check` | 0.24.1 / 3.15.0 | 文档校验 | MIT / ISC |
| `remark-cli` / `remark-validate-links` | 12.0.1 / 13.1.0 | 锚点与相对链接校验 | MIT |
| `@commitlint/*` / `husky` / `lint-staged` | 21.x / 9.1.7 / 17.6.1 | 提交门禁 | MIT |
| `cargo-deny`（crate，CI 工具） | 0.22.1 | Rust 依赖许可扫描（不进入运行时） | MIT OR Apache-2.0 |

## 本地参考源码（浅克隆，位于仓库外 `D:\Projects\_refs`，不入库）

只作阅读与比对，不进入本仓库的版本控制；如需复用代码，先按上表登记并核对许可。

| 名称 | 来源 URL | 版本 | SHA256 | 用途 | 许可结论 |
|---|---|---|---|---|---|
| `cs-demo-manager` | <https://github.com/akiver/cs-demo-manager> | HEAD `b0d3258`（2026-11-16） | 不比对（浅克隆，以 HEAD 定位） | demo 库、2D 雷达、录制 shim 的架构参照 | MIT |
| `demoparser` | <https://github.com/LaihoE/demoparser> | HEAD `fb2baab`（2026-11-15） | 同上 | tick 列式输出与 `parse_ticks` 口径 | MIT |
| `manta` | <https://github.com/dotabuff/manta> | HEAD `d4e4647`（2026-10-23） | 同上 | Source 2 demo 格式理解 | MIT |
| `awpy` | <https://github.com/pnxenopoulos/awpy> | HEAD `4d819ee`（2026-11-11） | 同上 | 视线、距离、热图算法口径 | MIT |
| `cs2-highlights-maker` | <https://github.com/Rovniy/cs2-highlights-maker> | HEAD `69835f3`（2026-06-12） | 同上 | HLAE + FFmpeg 编排范本 | **仓库无 LICENSE 文件 = 未声明许可**；只读，不复制代码 |
| `cs2-map-icons` | <https://github.com/MurkyYT/cs2-map-icons> | HEAD `6dcfb3a`（2026-11-12） | 同上 | 地图 `overview` 校准数据（开发期） | **仓库无 LICENSE 文件 = 未声明许可**；雷达图属 Valve 资产，发布版改为本机提取 |

## 不入库的对象

| 名称 | 来源 | 原因 |
|---|---|---|
| `.dem` demo 文件 | HLTV / 自有 | 版权归原权利人；只本地使用，`.gitignore` 排除 |
| POV 录制品（`.mp4` 等） | 本项目产出 | 含第三方比赛内容；只本地保存 |
| 地图几何 / `.glb` | Valve 或自行烘焙 | 资产版权与体积；运行时从本机提取或本地生成 |
| 任何凭据、令牌 | — | 安全边界；`.env` 禁止入库 |

## 维护约定

新增第三方二进制或源码前，先在本表登记并给出许可结论；**许可结论为空即不得引入**。许可结论发生变化的，追加新行，不修改旧行。
