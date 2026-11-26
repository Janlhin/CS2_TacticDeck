# 第三方许可边界

本文件只回答**许可与边界**（能不能用、能怎么用）。**致谢在 [README](../README.md#致谢)**，那里回答的是"我们从中拿了什么"。

本项目以 **AGPL-3.0-or-later** 分发（见 `LICENSE` 与 `NOTICE`）。判定原则：**公开可见 ≠ 授予再分发许可。** 没有明确许可证的项目，只读、不抄、不引入。

本项目采取 copyleft 的动因是**阻止闭源商业化**：任何分发本项目的衍生作品，或通过网络向用户提供其功能，都必须同样以 AGPL 提供完整对应源码。历史版本（截至 `4d97624`）曾以 MIT 发布，那部分授权不可撤回，已在 `NOTICE` 中说明。

## 1. 作为依赖引入（许可证明确，允许商用与修改）

| 组件 | 来源 | 许可证 | 用途 | 备注 |
|---|---|---|---|---|
| `source2-demo` | crates.io | MIT OR Apache-2.0 | CS2 demo 解析（S0 主路径） | 需实测道具/投射物字段粒度 |
| `demoparser` | github.com/LaihoE/demoparser | MIT | tick / 事件解析（sidecar） | Rust 核心未发布到 crates.io，只能走 Py/Node/WASM 绑定 |
| `demoinfocs-golang` | github.com/markus-wa/demoinfocs-golang | MIT | 字段语义参照（Go） | 不进入运行时 |
| `awpy` | github.com/pnxenopoulos/awpy | MIT | 算法口径验证（Python） | 不进入运行时 |
| `cs-demo-manager` | github.com/akiver/cs-demo-manager | MIT | 数据口径、雷达样式、录制 shim | 不 fork 其产品；仅取所需逻辑 |
| `advancedfx`（HLAE） | github.com/advancedfx/advancedfx | MIT（**子模块除外**） | demo 相机与 tick 命令 | **外部工具调用，不内嵌**；子模块需逐项核许可 |
| `PixiJS` | github.com/pixijs/pixijs | MIT | 2D 画布 | — |
| `Three.js` | github.com/mrdoob/three.js | MIT | 3D 视图（M2） | — |
| `Konva` | github.com/konvajs/konva | MIT | 标注层 | — |
| `Tauri` | github.com/tauri-apps/tauri | MIT / Apache-2.0 | 桌面壳 | — |
| `React` | github.com/facebook/react | MIT | 前端 | — |
| `TypeScript` | github.com/microsoft/TypeScript | Apache-2.0 | 语言 | — |
| `Zustand` | github.com/pmndrs/zustand | MIT | 状态管理 | — |
| `FFmpeg` | ffmpeg.org | LGPL 2.1+（默认构建） | 编码 / 封装 | **仅启用 nvenc / amf / mf + aac**；启用 GPL 组件会使整个产品转为 GPL，禁止 |
| `cargo-deny` | crates.io（github.com/EmbarkStudios/cargo-deny） | MIT OR Apache-2.0 | 构建 / CI 期 Rust 依赖许可扫描 | 仅工具链，不进入运行时；版本锁定 0.22.1，配置见 `config/deny.toml` |

## 2. 仅作参考，不复制代码

| 项目 | 许可证状态 | 参考内容 | 边界 |
|---|---|---|---|
| `ok2ju/oversite` | **未声明许可证** | PixiJS 组织方式、Go + Wails 结构 | 只读；如需复用先取得作者授权 |
| `huaxiangqiming/CSdemowatch` | **未声明许可证**（README 明示） | Replay V2 二进制缓存、高度剖切、从本机提取地图的做法 | 只读；如需复用先取得作者授权 |
| `Rovniy/cs2-highlights-maker` | 待核 | HLAE + shim 的录制编排时序 | 许可核实后再决定是否引用其代码 |
| `cs2.cam` | 闭源商业 | 产品形态与功能分级 | **未逆向、未抓取、未复制** |
| `MikaW21/LiteReplay` | MIT（上游仓库公开，对账提交 `fc23672`） | 抓屏（DXGI）、NVENC/AMF 编码参数、WASAPI 环回、时间戳对齐、`.partial` 原子提交、时间线剪辑 | 可复用代码，但须保留版权与许可声明并在此登记；其自带 FFmpeg DLL **不随本项目分发**（本项目 FFmpeg 只作独立进程调用） |

## 3. 资产与内容

| 对象 | 权利人 | 处理方式 |
|---|---|---|
| CS2 游戏本体、demo 格式 | Valve | 不分发；运行时读取用户本机安装 |
| 地图雷达图 / 几何 / 图标 | Valve | 不分发；运行时从本机只读提取并本地缓存 |
| `cs2-map-icons` 整理的 overview 数值 | 整理者 + Valve 原始资产 | 开发期参考；发布版改为本机提取 |
| demo 文件、POV 录制品 | 原权利人（HLTV / 选手 / 队伍） | 只存本地，不上传、不公开分享；`.dem` 与视频已在 `.gitignore` |

## 4. 风险提示

- **HLAE 与 VAC**：HLAE 官方声明其为"技术意义上的 hack"，加入受 VAC 保护的服务器大概率被封。本项目只在离线 demo 模式调用，并在 UI 中显著提示。
- **许可证传染**：新增任何依赖前先查许可证。Rust 侧已接入 `cargo deny`（0.22.1，配置 `config/deny.toml`；本地验收与 CI 均执行 `cargo deny --config config/deny.toml check licenses`）。判定口径：allow 列表只收录依赖图中实际出现的宽松许可，列入却未被任何 crate 使用的许可直接判失败；copyleft（AGPL）不进全局 allow，仅按包名对本仓库自有 crate 经 exceptions 授权，第三方依赖出现 AGPL/GPL 系列一律拒绝。npm 依赖许可检查尚未接入，留待后续任务。
- **更正机制**：若发现本项目使用了某组件却未在此登记，或对许可状态的判断有误，请开 issue，我们会修正并更新本文件。

## 5. 与本项目 AGPL-3.0-or-later 的兼容性

- **宽松许可依赖可并入 AGPL**：MIT / ISC / BSD / Apache-2.0 / MPL-2.0 / BlueOak 等均可作为组件进入 AGPL 作品，只需保留各自的版权与许可声明。已对 511 个已安装依赖做过许可分布核对，无 GPL 系列。
- **GPLv3 可组合**：依 AGPL 第 13 条，可与 GPLv3 作品链接成单一作品（组合后整体按 AGPL 分发）；**GPLv2-only 不可**，引入前必须确认是 "or later"。
- **FFmpeg（LGPL 2.1+）不内嵌**：本项目只把 FFmpeg 作为**独立进程调用**，不链接、不随包分发二进制，因此不构成 LGPL 与 AGPL 的直接组合。若将来改为随包分发 FFmpeg 二进制，必须重新评估（LGPL 2.1 无法一键升级到 AGPL），并记入 `DECISIONS.md`。
- **不得引入的许可**：GPLv2-only、SSPL、以及任何禁止商用的条款（如 CC-BY-NC），与 AGPL 目标冲突或无法满足。
- **分发时的义务**：任何分发行为都要附 `LICENSE` 全文与 `NOTICE`；网络交互场景依 AGPL 第 13 条提供对应源码。
