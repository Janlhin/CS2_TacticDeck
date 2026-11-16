# CS2 TacticDeck

**本地优先的 CS2 多 POV 战术复盘工作台。**
左侧 N 路 POV 视频、中间 2D 战术画布（轨迹 · 道具 · 手绘标注）、底部回合与 tick 双时间轴——三者由同一个 tick 时钟焊在一起，点谁看谁。素材不出本机，断网可用。

当前版本 `0.1.0`（初始开发阶段，公共 API 不稳定）。**已落地可运行骨架**：React 19 + PixiJS 8 的前端、Tauri 2 + Rust 的后端壳，`npm run app:dev` 可起窗口；demo 解析与播放尚未接入。

## 它解决什么

1. **多路录制没有编排**：手动把同一个回合录五遍，每遍起始 tick 都不同，录出来的是五段各自漂移的视频。
2. **视频与 demo 之间没有锚点格式**：播放端只能"大约对齐"，漂移随时间线性放大。
3. **2D 图层缺内容**：只有点位与走位时讲不了战术，缺道具落点、视野锥、可保存的标注。
4. **素材不能外传**：职业队与队伍训练的真实顾虑是 demo 不上云。市面标杆把录制放在服务端，恰好绕不开这一点。

## 验收标准（可机器判定）

每条判据都必须给出**命令 + 期望退出码**，不接受"看起来对"。

| 目标 | 判据 | 命令 | 期望退出码 |
|---|---|---|---|
| 仓库门禁 | 文档结构、字段、链接、根目录白名单、脚本基线全部通过 | `npm run verify` | 0 |
| G1 2D 回放 | 任意 `de_*` demo 打开到首帧 ≤ 8 s；拖拽 60 fps；点位与游戏内一致 | `cargo run -p tacticdeck-probe -- --demo <file> --check-coords` | 0 |
| G2 多 POV 同步 | 拖时间轴时 N 路视频与 2D 偏差 < 100 ms；连续 3 分钟漂移 < 300 ms | `cargo run -p tacticdeck-probe -- --sync-test <session>` | 0 |
| G3 批量录制 | 一个回合 → N 路对齐视频；每路首帧同 tick（±2）；耗时 ≤ 6× 回合时长 | `cargo run -p tacticdeck-record -- --round <n>` | 0 |
| G4 标注与手册 | 标注可存、可载、可导出 PDF | `cargo test -p tacticdeck-annotate` | 0 |

G1–G4 的命令属于规划中的二进制，当前尚未存在（见「已知限制」）。

## 技术栈与版本

技术栈登记是「文档与代码一致」这一层校验的数据来源。**未登记即无法验证是否一致。**

| 项 | 选型 | 版本 | 版本载体 |
|---|---|---|---|
| 语言（后端） | Rust | 1.99.1（`edition 2024`） | `src-tauri/Cargo.toml` |
| 语言（前端） | TypeScript（`strict` 全开） | 7.0.2 | `package.json` + `tsconfig.json` |
| 桌面壳 | Tauri | 2.12.5（stable；3.x 仍在 alpha） | `src-tauri/Cargo.toml` + `src-tauri/tauri.conf.json` |
| 前端框架 | React | 19.3.1 | `package.json` |
| 构建器 | Vite | 8.3.8 | `package.json` + `vite.config.ts` |
| 状态层 | Zustand | 5.1.0 | `package.json` |
| 2D 渲染 | PixiJS | 8.23.0 | `package.json` |
| 样式 | Sass（SCSS） | 1.108.2 | `package.json` |
| 3D 渲染（M2） | Three.js | 未引入（M2 接入） | 计划 |
| 标注层（S3） | Konva | 未引入（S3 接入） | 计划 |
| 包管理器 | npm | 11.20.0 | `package-lock.json` + `package.json` 的 `engines` |
| Node 运行时 | Node.js | 26.13.0 | `.nvmrc`（`verify` 的 `toolchain/node` 步骤比对） |
| 脚本运行时 | PowerShell | 7.6.4（要求 ≥ 7.4） | `scripts/*.ps1` 的 `#requires` |
| demo 解析（主） | `source2-demo` crate | 0.5.11（S0 引入） | 计划 |
| demo 解析（备） | `@laihoe/demoparser2` | 未引入（S0 兜底） | 计划 |
| 视频编码 | FFmpeg（仅 nvenc / amf / mf + aac） | 随抓屏器 | 运行时探测 |

版本策略：**取最新稳定版，预发布不纳入基线**（见 [DECISIONS.md](DECISIONS.md) D-010）。主版本升级必须独立提案并单独提交；未真正接入的依赖不提前装进 `package.json`，避免技术栈登记失真。

## 目录结构

顶层目录与职责，登记于此。出现在声明范围外的路径，需要在提交说明里给出解释。

| 路径 | 职责 |
|---|---|
| `README.md` | 项目定位、验收标准、技术栈、目录、已知限制 |
| `AGENTS.md` | AI 协作约束的**目录**（≤150 行），只放命令入口、禁止清单与链接 |
| `DECISIONS.md` | 追加式决策日志：采纳方案、否决方案、理由、复审条件 |
| `TASKS.md` | 任务队列与待裁决台账，按功能面分组 |
| `MANIFEST.md` | 第三方二进制、源码与逆向对象的来源登记（URL + SHA256 + 用途 + 许可结论） |
| `docs/` | 工程规范全文（`engineering-standards.md`）与第三方许可边界（`third-party-notices.md`） |
| `scripts/` | 全部可执行逻辑：`verify.ps1`、`hooks/*.ps1`、后续的 `pre-release.ps1` |
| `config/` | 工具配置：Markdown 校验、链接检查、提交信息规则 |
| `.github/` | CI：每次推送与 PR 运行 `verify` |
| `.husky/` | 提交门禁薄壳（仅转发到 `scripts/hooks/*.ps1`） |
| `crates/` | Rust 侧共享库：解析、录制编排、同步算法（S0 起按需建立） |
| `src/` | 前端源码：`App.tsx`、`components/`、`store/`、`styles.scss` |
| `src-tauri/` | Tauri 2 壳：`src/lib.rs` 命令层、`tauri.conf.json`、`capabilities/` |

## 安装与运行

前置：PowerShell 7.4+、Node.js 26.13.0（版本见 `.nvmrc`）、Rust 1.99（后续构建阶段才需要）。先克隆 `https://github.com/Janlhin/CS2_TacticDeck.git` 并进入目录。

机器上若存在多个 Node（例如某些工具自带运行时并排在 PATH 前面），把 Node 26 放到 PATH 最前；`npm run verify` 的 `toolchain/node` 步骤会在版本不符时直接失败并打印实际解析到的 `node.exe` 路径。`.npmrc` 已设 `engine-strict=true`，装依赖时同样会拦。

安装依赖（一次性）：

```powershell
npm ci
```

验收：

```powershell
npm run verify
```

`npm run verify` 是**唯一验收入口**：它串起文档结构、字段、链接、根目录白名单、脚本基线，以及 `src-tauri/Cargo.toml` 存在时的 `cargo fmt` / `clippy` / `test`。它失败就不得提交，禁止绕过。

起窗口：`npm run app:dev`（需要 WebView2 运行时，Win10/11 默认自带）；只调前端样式可先跑 `npm run dev` 在浏览器里看。打包：`npm run app:build`（当前 `bundle.active` 为 `false`，只产出可执行文件，不做安装包）。

## 已知限制

- **只到骨架**：窗口、布局、五路槽位状态层、Pixi 网格可跑；demo 解析、播放、同步、录制均未接入。下表 G1–G4 的命令是待实现的契约，不是现状。
- **盘面是占位**：`RadarCanvas` 只画网格，等地图校准（`pos_x` / `pos_y` / `scale` / `rotate`）接入后替换。
- **平台**：只支持 Windows x64（抓屏依赖 DXGI）。
- **录制依赖外部工具**：HLAE 官方声明其为"技术意义上的 hack"，仅可用于看 demo 与制作视频。本项目只在离线 demo 模式调用，**绝不接入受 VAC 保护的服务器**。
- **图形局限**：只支持 SDR；不支持旋转屏；独占全屏可能抓不到；多实例需自行加锁。
- **素材**：demo 与 POV 录制品版权归原权利人，只存本地、不上传；`.dem` / 视频 / `.glb` 已在 `.gitignore` 排除。
- **资产**：地图盘面等游戏资源版权归 Valve，本项目不分发，运行时从本机 CS2 只读提取。
- **未验证**：S4 录制流水线尚未在任何真实回合上跑过；多路同步算法的漂移方程只有设计值，没有实测数据。
- **CI 口径**：CI 覆盖每次推送与 PR 的最新提交，**采用顶端提交验证**，未逐提交验证历史。

## 合规边界

- FFmpeg 只启用 LGPL 兼容组件（nvenc / amf / mf + aac），不启用 GPL 组件（如 x264），以保证可分发性。
- 未声明许可证的项目（`oversite`、`CSdemowatch`）只读参考，**不复制任何代码**；`cs2.cam` 为闭源产品，未逆向、未抓取。
- 详细清单与判定见 [第三方许可边界](docs/third-party-notices.md)。

## 路线图

| 阶段 | 内容 | 交付物 | 状态 |
|---|---|---|---|
| **S-0** | 技术栈骨架 | Tauri 2 壳 + React 19 前端 + Pixi 画布占位 | 完成（2026-11-16） |
| **S0** | 解析 + 地图校准 | `dem → replay.json` CLI | 未开始 |
| **S1** | 2D 画布 | 可拖拽 2D 回放页 | 未开始 |
| **S2** | 视频条 + 同步 | 视频 / 2D 同屏 | 未开始 |
| **S3** | 标注 + 手册 | 标注层、本地手册 | 未开始 |
| **S4** | 批量录制 | `record-round` 子命令 | 未开始 |
| **S5** | 打包分发 | Windows 安装包 | 未开始 |
| **M2** | 可选增强 | 3D 视图、分享、道具反查 | 未开始 |

止损设计：S0–S3 不依赖录制，可用离线素材独立验证；S4 若被引擎行为卡住，前三阶段仍可交付。

## 致谢

这个项目建立在别人公开的工作之上。以下按「我们从中拿了什么」分类；所列作者与本项目无隶属关系，致谢不等于背书。

### 解析层

- **[LaihoE/demoparser](https://github.com/LaihoE/demoparser)**（MIT，746★）——"像查数据库一样查询 demo"的思路（`parse_ticks` / `parse_event` 输出列式数据）直接塑造了 `replay.json` 的形状。
- **[source2-demo](https://docs.rs/source2-demo)**（MIT OR Apache-2.0，crates.io 0.5.11）——可 `cargo add` 的 Rust 原生 Source 2 解析库，S0 的首选嵌入方案。
- **[markus-wa/demoinfocs-golang](https://github.com/markus-wa/demoinfocs-golang)**（MIT）——核对字段语义的参照。
- **[pnxenopoulos/awpy](https://github.com/pnxenopoulos/awpy)**（MIT）——验证视线、距离、热图等算法口径。
- **[dotabuff/manta](https://github.com/dotabuff/manta)**——Source 2 demo 格式的许多理解由此而来，本项目因此间接受益。

### 录制层

- **[advancedfx/advancedfx](https://github.com/advancedfx/advancedfx)**（HLAE，MIT，**子模块另有许可**）——demo 相机控制与逐 tick 命令的事实标准；录制时序的四条硬规则，多数是它的使用者用失败换来的。
- **[akiver/cs-demo-manager](https://github.com/akiver/cs-demo-manager)**（MIT）——demo 库、2D 雷达、`spec_player` 驱动与录制 shim 的参考。
- **[Rovniy/cs2-highlights-maker](https://github.com/Rovniy/cs2-highlights-maker)**——"HLAE 作外壳、按 tick 精确执行命令、帧直通 FFmpeg"的编排范本。
- **[LiteReplay](https://www.xiaoheihe.cn/app/bbs/link/196995653)**（作者 Yuzi_L，MIT）——录制引擎源码是抓屏层的起点：DXGI 抓屏、NVENC/AMF 参数、WASAPI 环回与时间戳对齐、`.partial` 原子提交。
- **[ravy.pro · From a demo file to a highlight reel](https://ravy.pro/blogs/from-a-demo-file-to-a-highlight-reel-a-cs2-tool-i-always-wanted)**——把整条流水线讲清楚的一手记录。

### 前端与渲染

- **[PixiJS](https://github.com/pixijs/pixijs)**（MIT）——2D 画布。
- **[Three.js](https://github.com/mrdoob/three.js)**（MIT）——M2 的 3D 视图。
- **[Konva](https://github.com/konvajs/konva)**（MIT）——标注层。
- **[Tauri](https://github.com/tauri-apps/tauri)**（MIT / Apache-2.0）——桌面壳。
- **[React](https://github.com/facebook/react)**（MIT）、**[TypeScript](https://github.com/microsoft/TypeScript)**（Apache-2.0）、**[Zustand](https://github.com/pmndrs/zustand)**（MIT）。

### 地图数据

- **[MurkyYT/cs2-map-icons](https://github.com/MurkyYT/cs2-map-icons)**——雷达图与 `overview` 数据（`pos_x` / `pos_y` / `scale` / `rotate`）的整理者。其中雷达图属 Valve 资产，仅开发期使用。

### 产品与架构启发（非代码复用）

- **[cs2.cam](https://cs2.cam)**——产品形态的参照物。闭源商业产品，未逆向、未复制其任何实现或素材；我们做的正是它没有的那一半：本地优先。
- **[ok2ju/oversite](https://github.com/ok2ju/oversite)** 与 **[huaxiangqiming/CSdemowatch](https://github.com/huaxiangqiming/CSdemowatch)**——均**未声明开源许可证**。只当作架构与交互的读物，**未复制任何代码**；将来若要复用，先取得作者授权。

### 平台

- **Valve / Counter-Strike 2**——游戏本体、demo 格式与地图资产的所有者。本项目是第三方工具，与 Valve 无关联。

如果你在任何地方发现疑似复制未授权代码，请开 issue，我们会立即核查并修正。

## 许可证

**AGPL-3.0-or-later**（GNU Affero General Public License v3 或更新版本）© 2026 Janlhin。全文见 [LICENSE](LICENSE)，历史授权与免责见 [NOTICE](NOTICE)。

选择 copyleft 的目的很明确：**不允许把本项目改成闭源商业产品**。分发衍生作品、或通过网络向用户提供其功能，都必须以 AGPL 提供完整对应源码；内部自用、二次开发、自行构建均不受限制。

历史版本（截至提交 `4d97624`）曾以 MIT 发布，那部分授权对已获得副本的人不可撤回，已在 [NOTICE](NOTICE) 中说明。第三方组件的许可与边界见 [docs/third-party-notices.md](docs/third-party-notices.md)。
