# CS2 TacticDeck

**本地优先的 CS2 多 POV 战术复盘工作台**
*Local-first multi-POV tactical replay workbench for Counter-Strike 2*

> 左侧 N 路 POV 视频，中间 2D 战术画布（轨迹 · 道具 · 手绘标注），底部回合与 tick 双时间轴 —— 三者被同一个 tick 时钟焊在一起，点谁看谁。素材不出本机，断网可用。

**状态：立项阶段（v0.1）**。仓库当前只有设计文档，尚无可用代码。首个可运行里程碑是 S0：`dem → replay.json` 的解析 CLI。

---

## 为什么自己做

- 标杆产品 `cs2.cam` 已经把这条路走通，但它的"录 POV"发生在**它自己的服务器**上（官网原文：*Cloud recording · Record any pick server-side — no PC needed*）。用户离开网络与服务，手上就没有素材。
- 开源侧有一批 2D 回放实现，**却没有任何一个做"多路 POV 视频与 tick 同步"**。这是空位，也是本项目存在的理由。
- 职业队真实的顾虑是"demo 不外传"。本地优先不是省钱手段，是产品定位。

## 界面（目标形态）

```
┌──────────────────────────────────────────────────────────────────────┐
│ [1 POV] [2 POV] [3 POV] [4 POV] [5 POV]   ← 缩略图栏，点击切主画面   │
├───────────────────────┬──────────────────────────────────────────────┤
│                       │                                              │
│   主画面（当前 POV）   │        2D 战术画布（PixiJS）                 │
│   视频与 tick 同步      │        点位 / 朝向 / 轨迹 / 道具 / 标注        │
│                       │                                              │
├───────────────────────┴──────────────────────────────────────────────┤
│ 名册 · 经济 · 击杀交换                                                │
├──────────────────────────────────────────────────────────────────────┤
│ 回合条 1–19  ●  事件轴（击杀 / 道具 / 炸弹）  播放头 1:49              │
└──────────────────────────────────────────────────────────────────────┘
```

## 核心设计：只有一个时间基

本项目的难点不是 3D，也不是 UI，而是**把 N 段屏幕录制与一个 tick 时钟对齐**。

```
tick = startTick + t_video × tickrate
```

规则四条（来自社区踩坑记录与 `cs-demo-manager` 的提交历史）：

1. 录制命令不得早于 **tick 96** —— 跳转后游戏会丢弃最初若干 tick。
2. **seek 必须早于 camera 命令** —— 2025-10 更新后同时下发会被忽略。
3. 每段录制前**暂停 4 tick** —— 让跳转后的加载过渡过去，否则首帧是脏的。
4. 多路之间**不混用分辨率** —— 尺度一旦不同，标注无法复用。

同步策略：以 tick 累加器为主时钟，视频为从属；误差小于 100 ms 用 `playbackRate = 1 ± 0.003` 微调，超过 150 ms 才 seek；每 30 s 做一次 QPC 回归，偏差超过 0.5% 就换段重锚。

## 架构

```
L4 呈现层  视频条 / 2D 画布 / 名册 / 双时间轴 / 标注
L3 同步层  TickClock（唯一真源）+ 视频从属校正 + 段表 {player, file, startTick, tickrate}
L2 数据层  replay.json（rounds / ticks / events / grenades / players）+ 二进制轨迹缓存
L1 录制层  Rust 编排器 → HLAE 控制相机 → 抓屏器（LiteReplay 改造）→ 写锚点
L0 解析层  Rust demo 解析 → tick 表 / 事件 / 道具；地图盘面在校准后从本机提取
```

## 路线图

| 阶段 | 内容 | 交付物 | 验收 |
|---|---|---|---|
| **S0** | 解析 + 地图校准 | `dem → replay.json` CLI | 3 图 × 2 demo 解析无异常；点位与游戏内一致 |
| **S1** | 2D 画布 | 可拖拽 2D 回放页 | 60 fps；回合跳转 < 200 ms |
| **S2** | 视频条 + 同步 | 视频 / 2D 同屏 | 3 分钟漂移 < 100 ms |
| **S3** | 标注 + 手册 | 标注层、本地手册 | 存 / 载 / 导出 PDF |
| **S4** | 批量录制 | `record-round` 子命令 | 5 路首帧同 tick（±2）；耗时 ≤ 6× 回合 |
| **S5** | 打包分发 | Windows 安装包 | 冷启动 < 3 s |
| **M2** | 可选增强 | 3D 视图、分享、道具反查 | — |

止损设计：S0–S3 不依赖录制，可用离线素材独立验证；S4 若被引擎行为卡住，前三阶段仍可交付。

## 技术栈

Rust（Tauri v2 后端、解析、录制编排）· TypeScript / SCSS / React 19 · PixiJS v8（2D）· Three.js（M2 的 3D）· Konva（标注）· FFmpeg（仅 nvenc / amf / mf + aac）

## 范围与非目标

**做**：单机、单用户、本地文件、按回合、可配 N 路、2D 视图、标注、导出。
**不做**：账号与云同步、公开分享、电竞数据平台、AI 战术识别、语音同步；MVP 也不做 3D。

## 合规边界（请先读）

- **HLAE 是 VAC 风险的**：官方明确其为"技术意义上的 hack"，只可用于看 demo / 制作视频。本项目只在离线 demo 模式调用它，绝不接入任何受 VAC 保护的服务器。
- **FFmpeg 许可**：本项目只启用 LGPL 兼容组件，不启用 GPL 组件（如 x264），以保证可分发性。
- **Valve 资产**：地图盘面等游戏资源版权归 Valve。本项目**不分发**任何游戏资源，一律在运行时从用户本机已安装的 CS2 只读提取并本地缓存。
- **demo 与录制品**：demo 与 POV 视频的版权归原权利人。默认只存本地，不提供上传；`.dem` / 视频文件已在 `.gitignore` 中排除，防止误提交。

## 致谢

这个项目能起步，几乎完全建立在别人公开的工作之上。以下按"我们从中拿了什么"分类；**所列作者与本项目无隶属关系，致谢不等于背书**。

### 解析层

- **[LaihoE/demoparser](https://github.com/LaihoE/demoparser)**（MIT，746★）——CS2 demo 解析的事实标准。它"像查数据库一样查询 demo"的思路（`parse_ticks` / `parse_event` 输出列式数据）直接塑造了我们的 `replay.json` 形状。其 Rust 核心以 Py/Node/WASM 绑定发布，本项目按需以 sidecar 方式调用。
- **[source2-demo](https://crates.io/crates/source2-demo)**（MIT OR Apache-2.0）——可 `cargo add` 的 Rust 原生 Source 2 解析库，是本项目 S0 的首选嵌入方案。
- **[markus-wa/demoinfocs-golang](https://github.com/markus-wa/demoinfocs-golang)**（MIT）——Go 侧的成熟实现，是我们核对字段语义时的参照。
- **[pnxenopoulos/awpy](https://github.com/pnxenopoulos/awpy)**（MIT）——分析向的封装（视线、距离、热图），我们借它验证算法口径。
- **[dotabuff/manta](https://github.com/dotabuff/manta)**——demoparser 的作者把功劳归于它；Source 2 demo 格式的许多理解由此而来，本项目因此间接受益。

### 录制层

- **[advancedfx/advancedfx](https://github.com/advancedfx/advancedfx)**（HLAE，MIT，**子模块另有许可**）——demo 相机控制与逐 tick 命令的事实标准。上面四条时序规则，大部分是 HLAE 使用者用失败换来的经验。
- **[akiver/cs-demo-manager](https://github.com/akiver/cs-demo-manager)**（MIT）——demo 库、2D 雷达、`spec_player` 驱动、录制注入 shim。本项目**不复刻它的产品**，但把它当作数据口径与雷达样式的参照，并在 S4 复用其 shim 思路。
- **[Rovniy/cs2-highlights-maker](https://github.com/Rovniy/cs2-highlights-maker)**——"以 HLAE 为外壳、按 tick 精确执行命令、帧直通 FFmpeg"的编排范本。CS2 移除 `.vdm` 之后，这几乎是唯一可靠的路子。
- **[LiteReplay](https://www.xiaoheihe.cn/app/bbs/link/196995653)**（作者 Yuzi_L，MIT）——录制引擎的源码是本项目 L1 的起点：DXGI 抓屏、NVENC/AMF 参数、WASAPI 环回与时间戳对齐、`.partial` 原子提交。作者已表示将公开仓库，我们保留了同源快照用于逐文件比对。
- **[ravy.pro · From a demo file to a highlight reel](https://ravy.pro/blogs/from-a-demo-file-to-a-highlight-reel-a-cs2-tool-i-always-wanted)**——把整条流水线讲清楚的一手实践记录。

### 前端与渲染

- **[PixiJS](https://github.com/pixijs/pixijs)**（MIT）——2D 画布；几百条轨迹线还能稳 60 fps，靠的是它。
- **[Three.js](https://github.com/mrdoob/three.js)**（MIT）——M2 的 3D 视图。
- **[Konva](https://github.com/konvajs/konva)**（MIT）——标注层。
- **[Tauri](https://github.com/tauri-apps/tauri)**（MIT / Apache-2.0）——桌面壳。
- **[React](https://github.com/facebook/react)**（MIT）、**[TypeScript](https://github.com/microsoft/TypeScript)**（Apache-2.0）、**[Zustand](https://github.com/pmndrs/zustand)**（MIT）。

### 地图数据

- **[MurkyYT/cs2-map-icons](https://github.com/MurkyYT/cs2-map-icons)**——雷达图与 `overview` 数据（`pos_x` / `pos_y` / `scale` / `rotate`）的整理者。注意：其中雷达图属 Valve 资产，本项目仅开发期使用，发布版改为运行时从本机提取。

### 产品与架构启发（非代码复用）

- **[cs2.cam](https://cs2.cam)**——产品形态的参照物。闭源商业产品，本项目**未逆向、未复制**其任何实现或素材；我们做的正是它没有的那一半：本地优先。
- **[ok2ju/oversite](https://github.com/ok2ju/oversite)**（Go + Wails + React + PixiJS 的 2D 回放）与 **[huaxiangqiming/CSdemowatch](https://github.com/huaxiangqiming/CSdemowatch)**（Godot 3D 沙盘）——两者均**未声明开源许可证**。本项目只把它们当作架构与交互的读物，**未复制任何代码**。若将来需要复用，会先取得作者授权。

### 平台

- **Valve / Counter-Strike 2**——游戏本体、demo 格式与地图资产的所有者。本项目是第三方工具，与 Valve 无关联。

**关于上述"未声明许可"的项目**：本项目严格遵守"公开可见 ≠ 授予再分发许可"。如果你在任何地方发现疑似复制，请开 issue，我们会立即核查并修正。

## 参与

- 现在最有价值的贡献是**经验**而非代码：如果你录过 CS2 demo 视频，欢迎把"哪些命令在哪个 tick 上会失效"写成 issue。
- 代码贡献请保持提交原子化：一个 commit 只做一件事。
- 想一起做，先看 `THIRD_PARTY_NOTICES.md` 与合规边界，再动手。

## 许可证

[MIT](LICENSE) © 2026 Janlhin

第三方组件的许可与边界见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。
