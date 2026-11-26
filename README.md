# CS2 TacticDeck

**本地优先的 CS2 多 POV 战术复盘工作台。**
左侧 N 路 POV 视频、中间 2D 战术画布（轨迹 · 道具 · 手绘标注）、底部回合与 tick 双时间轴——三者由同一个 tick 时钟焊在一起，点谁看谁。素材不出本机，断网可用。

当前版本 `0.1.0`（初始开发阶段，公共 API 不稳定）。**已落地可运行骨架 + demo 解析链路**：React 19 + PixiJS 8 的前端、Tauri 2 + Rust 的后端壳，`bun run app:dev` 可起窗口；`crates/tacticdeck-probe` 已能把真实 `.dem` 导成 `replay.json`（S0 / T-01），窗口启动时会从本机 demo 库自动吃这份真实数据（T-13）。

## 它解决什么

1. **多路录制没有编排**：手动把同一个回合录五遍，每遍起始 tick 都不同，录出来的是五段各自漂移的视频。
2. **视频与 demo 之间没有锚点格式**：播放端只能"大约对齐"，漂移随时间线性放大。
3. **2D 图层缺内容**：只有点位与走位时讲不了战术，缺道具落点、视野锥、可保存的标注。
4. **素材不能外传**：职业队与队伍训练的真实顾虑是 demo 不上云。市面标杆把录制放在服务端，恰好绕不开这一点。

## 验收标准（可机器判定）

每条判据都必须给出**命令 + 期望退出码**，不接受"看起来对"。

| 目标 | 判据 | 命令 | 期望退出码 |
|---|---|---|---|
| 仓库门禁 | 文档结构、字段、链接、根目录白名单、脚本基线全部通过 | `bun run verify` | 0 |
| G1 2D 回放 | 任意 `de_*` demo 能导成 `replay.json` 并通过结构校验；地图校准后点位与游戏内一致 | `cargo run -p tacticdeck-probe -- --demo <file> --out replay.json` → 0，随后 `cargo run -p tacticdeck-probe -- --validate replay.json` → 0 | 0 |
| G2 多 POV 同步 | 拖时间轴时 N 路视频与 2D 偏差 < 100 ms；连续 3 分钟漂移 < 300 ms | `cargo run -p tacticdeck-probe -- --sync-test <session>` | 0 |
| G3 批量录制 | 一个回合 → N 路对齐视频；每路首帧同 tick（±2）；耗时 ≤ 6× 回合时长 | `cargo run -p tacticdeck-record -- --round <n>` | 0 |
| G4 标注与手册 | 标注可存、可载、可导出 PDF | `cargo test -p tacticdeck-annotate` | 0 |

G1–G4 的命令属于规划中的二进制，当前尚未存在（见「已知限制」）。

## 技术栈与版本

技术栈登记是「文档与代码一致」这一层校验的数据来源。**未登记即无法验证是否一致。**

| 项 | 选型 | 版本 | 版本载体 |
|---|---|---|---|
| 语言（后端） | Rust | 1.99.1（`edition 2024`） | 根 `Cargo.toml` 的 `[workspace.package]` + `src-tauri/Cargo.toml` |
| 语言（前端） | TypeScript（`strict` 全开） | 7.0.2 | `package.json` + `tsconfig.json` |
| 桌面壳 | Tauri | 2.12.5（stable；3.x 仍在 alpha） | `src-tauri/Cargo.toml` + `src-tauri/tauri.conf.json` |
| 前端框架 | React | 19.3.1 | `package.json` |
| 构建器 | Vite | 8.3.8 | `package.json` + `vite.config.ts` |
| 状态层 | Zustand | 5.1.0 | `package.json` |
| 2D 渲染 | PixiJS | 8.23.0 | `package.json` |
| 样式 | Sass（SCSS） | 1.108.2 | `package.json` |
| 3D 渲染（M2） | Three.js | 未引入（M2 接入） | 计划 |
| 标注层（S3） | Konva | 未引入（S3 接入） | 计划 |
| 包管理与工具链 | Bun | 1.4.4 | `package.json` 的 `scripts` + `bun.lock` + `.bun-version`（`verify` 的 `toolchain/bun` 步骤比对） |
| 脚本运行时 | PowerShell | 7.6.4（要求 ≥ 7.4） | `scripts/*.ps1` 的 `#requires` |
| demo 解析（主） | `source2-demo` crate | 0.5.11（已接入，T-01） | `crates/tacticdeck-probe/Cargo.toml` |
| 地图标定（雷达） | 本机 CS2 `pak01_dir.vpk` 内 `resource/overviews/*.txt` | 运行时提取（T-02） | `crates/tacticdeck-map/src/map_profile.rs` |
| 雷达底图 | 本机 CS2 `pak01_dir.vpk` 内 `panorama/images/overheadmaps/*.vtex_c` | 运行时解码，不随包分发（T-17） | `crates/tacticdeck-map/src/radar.rs` |
| 播放时钟 | 自研 `tacticdeck-sync` crate | 0.1.0（T-06：tick 主时钟 + 倍速校正 + 3 分钟漂移回归 < 100 ms） | `crates/tacticdeck-sync/src/clock.rs` |
| 标注模型 | 自研 `tacticdeck-annotate` crate | 0.1.0（T-08：命令数组 + 撤销/重做栈 + 手册 JSON 落盘） | `crates/tacticdeck-annotate/src/manual.rs` |
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
| `docs/` | 工程规范全文（`engineering-standards.md`）、第三方许可边界（`third-party-notices.md`）与 `replay.json` 字段对照草案（`replay-json-fields.md`） |
| `scripts/` | 全部可执行逻辑：`verify.ps1`、`hooks/*.ps1`、后续的 `pre-release.ps1` |
| `config/` | 工具配置：Markdown 校验、链接检查、提交信息规则 |
| `.github/` | CI：每次推送与 PR 运行 `verify` |
| `.husky/` | 提交门禁薄壳（仅转发到 `scripts/hooks/*.ps1`） |
| `crates/` | Rust 侧共享库与二进制：`tacticdeck-probe`（demo → `replay.json`，T-01）、后续的同步算法与录制编排 |
| `src/` | 前端源码：`App.tsx`、`components/`、`store/`、`styles.scss` |
| `src-tauri/` | Tauri 2 壳：`src/lib.rs` 命令层、`tauri.conf.json`、`capabilities/` |
| `Cargo.toml` / `Cargo.lock` | 根 workspace：成员 `src-tauri` 与 `crates/tacticdeck-{probe,map,sync,annotate}`，锁文件随根目录走 |

## 安装与运行

前置：PowerShell 7.4+、Bun 1.4.4（版本见 `.bun-version`）、Rust 1.99（后续构建阶段才需要）。先克隆 `https://github.com/Janlhin/CS2_TacticDeck.git` 并进入目录。

前端依赖由 Bun 安装与运行：`bun install` 读取 `package.json` 生成 `bun.lock` 与 `node_modules`，所有 npm 工具（Vite / TypeScript / Tauri CLI / commitlint / 文档检查）都由 `bun run <script>` 或 `bun node_modules/<包入口>` 驱动。若机器上 `bun --version` 与 `.bun-version` 不符，`verify` 的 `toolchain/bun` 步骤会直接失败并打印实际解析到的路径。

安装依赖（一次性，同时装提交钩子）：

```powershell
bun install --frozen-lockfile
bun run hooks
```

验收：

```powershell
bun run verify
```

前端逻辑测试（回放选择器、槽位轮换、十人点位断言）单独跑：

```powershell
bun run test:web
```

`bun run verify` 是**唯一验收入口**：它串起文档结构、字段、链接、根目录白名单、脚本基线，以及根 `Cargo.toml` 存在时的 `cargo fmt` / `clippy` / `test`（按 `--workspace` 跑全部 crate）。它失败就不得提交，禁止绕过。

### demo 导出（S0 / T-01）

`crates/tacticdeck-probe` 是独立二进制，不吃前端依赖，直接 `cargo run -p tacticdeck-probe -- …`：

- `--demo <file.dem>` + `--out <replay.json>`：解析 demo 并写出前端契约格式；`--step <n>` 控制采样间隔（默认 8 tick）。
- `--validate <replay.json>`：只读校验，检查 10 名选手、槽位连续、帧 tick 递增、坐标落在 0..1、回合起止与击杀槽位合法。
- `--extract-maps`（可配 `--cs2 <game/csgo 路径>` 与 `--out <map-profile.json>`）：从本机 CS2 的 `pak01_dir.vpk` 里读 `resource/overviews/*.txt`，抽出每张地图的 `pos_x / pos_y / scale / rotate`。CS2 路径默认按「参数 → `CS2_DIR` → Steam 注册表与 libraryfolders.vdf」的顺序探测。
- `--map-profile <map-profile.json>`：让导出改用雷达标定换算坐标（`meta.normalize` 变 `radar`），红点直接落在雷达图对应的位置。缺这张图时退回包围盒归一化。
- `--extract-radar <map>`（可配 `--cs2 <game/csgo 路径>` 与 `--out <file.png>`）：把本机 CS2 里的雷达底图纹理（`panorama/images/overheadmaps/<map>_radar_psd.vtex_c`）解成 1024×1024 PNG，用于肉眼核对底图与坐标是否对齐。
- `--check-radar <replay.json>`（可配 `--cs2 <game/csgo 路径>`）：把每帧每人的点位投到该地图的雷达底图上，算「落在不透明像素上」的比例；命中率低于 98% 直接报错，用来判断坐标口径是否真的对得上盘面。

坐标口径：Source 2 的世界坐标是「格子 + 格内偏移」两段式存的——格子边长 **512** 单位、世界原点在 **-16384**，还原公式是 `cell * 512 - 16384 + vec`。拿到世界坐标后按地图的 `pos_x / pos_y / scale` 换成雷达像素，再压到 0..1 并把 y 轴翻到左下原点（板面口径）。

坐标口径：Source 2 把世界坐标拆成 `cell` 与 `vec` 余数，还原公式 `cell * 512 - 16384 + vec`（格子边长 512，世界原点在 -16384）；导出默认按等比例包围盒压到 0..1 并翻转 y 轴（板面左下为原点），带 `--map-profile` 时改走雷达标定。

真实 demo 实测（`test_demo.dem`，de_mirage，64 tick）：10 回合、6575 帧、48 条击杀、10 名选手、5 MB 单文件。`replay.json` 属运行数据，已进 `.gitignore`；`tests/replay-contract.test.ts` 在本地存在该文件时才跑契约回归，缺失则跳过。

本机 CS2 实测（`--extract-maps`）：17 张地图（同图的多份 `_v2 / _night` overview 已按图名归一合并），`de_mirage` 抽到 `pos_x=-3230 / pos_y=1713 / scale=5 / rotate=0`，与公开标定值一致。`map-profile.json` 同样是运行数据，不入库。

真实 Valve 录像实测（10 张地图各一份，`--step 16`）：`--validate` 全部通过，`--check-radar` 命中率 97.1%–100.0%（Dust2 与 Inferno 100.0%、Mirage 99.8%、Vertigo 97.5%、Ancient 97.1%）。CS2 demo 的回合事件是 `round_prestart` / `round_officially_ended`，导出时按这对事件切回合；热身帧、掉线帧和个别读歪的实体帧会被丢掉（只保留十人齐全且坐标在 0..1 的帧）。

本机雷达底图实测（`--extract-radar de_mirage`）：VTEX 1024×1024、format 28（RGBA8888，4 字节/像素）、单 mip、LZ4 压缩，解出 4,194,304 字节像素，写成 271,686 字节 PNG；盘面与十人点位在窗口内对齐。

起窗口：`bun run app:dev`（需要 WebView2 运行时，Win10/11 默认自带）；只调前端样式可先跑 `bun run dev` 在浏览器里看。打包：`bun run app:build`（当前 `bundle.active` 为 `false`，只产出可执行文件，不做安装包）。

demo 库目录（按优先级）：环境变量 `TACTICDECK_DEMO_DIR` → `文档/CS2_TacticDeck/demos` → 仓库根目录。前两处放 `replay.json` 即被扫描到；`.dem` 会列出来但标记「待导出」，先用 probe 导出成 `replay.json` 再点载入。窗口启动时自动载入库里第一条可用的回放，没有才退回内置合成样本。

## 已知限制

- **骨架 + 解析 + 本机库入口**：窗口、布局、左栏 11 槽（1 固定 2D + 10 人）、槽位状态层、Pixi 网格可跑；`.dem → replay.json` 已能产出真实数据，右栏「demo 库」会扫描本机回放并一键载入（T-13）。播放 / 同步 / 录制仍未接入，下表 G2–G4 的命令是待实现的契约。
- **底图是本机现取的真图**：2D 板面启动时向 Rust 侧要一张 `data:image/png` 底图，Rust 直接读本机 CS2 的 `panorama/images/overheadmaps/<map>_radar_psd.vtex_c`（VTEX：RGBA8888 + LZ4，1024×1024），解出来铺在板面下面。底图属 Valve 资产，只在运行时从玩家自己的安装里解码，不随仓库或安装包分发；没装 CS2 或取不到时退回网格（T-17）。
- **只有 Mirage 有真记录验证**：坐标校准用 de_mirage 的真实 demo 跑通（65750 个采样点 99.6% 落在雷达图可走区域），其余地图的 `pos/scale` 能抽出来但还没有 demo 复核。

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
| **S0** | 解析 + 地图校准 | `dem → replay.json` CLI | 进行中（T-01 完成，T-02 / T-03 未开始） |
| **S1** | 2D 画布 | 可拖拽 2D 回放页 | 进行中（T-04 / T-05 / T-13 / T-14 / T-15 完成） |
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
- **[LiteReplay](https://github.com/MikaW21/LiteReplay)**（作者 MikaW21 / Yuzi_L，MIT）——录制引擎源码是抓屏层的起点：DXGI 抓屏、NVENC/AMF 参数、WASAPI 环回与时间戳对齐、`.partial` 原子提交。上游仓库已公开，本地快照与其 `fc23672` 逐字节一致。
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
