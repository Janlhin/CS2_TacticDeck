# 第三方组件与许可边界

本项目以 MIT 分发（见 `LICENSE`）。以下是实际使用、计划使用或仅作参考的第三方工作，以及我们对每一类**能做什么、不能做什么**的判断。

> 判定原则：**公开可见 ≠ 授予再分发许可。** 没有明确许可证的项目，只读、不抄、不引入。

## 1. 将作为依赖引入（许可证明确，允许商用与修改）

| 组件 | 来源 | 许可证 | 用途 | 备注 |
|---|---|---|---|---|
| `source2-demo` | crates.io | MIT OR Apache-2.0 | CS2 demo 解析（S0 首选） | 需实测道具/投射物字段粒度 |
| `demoparser` | github.com/LaihoE/demoparser | MIT | tick / 事件解析（sidecar） | Rust 核心未发布到 crates.io，只能走 Py/Node/WASM 绑定 |
| `demoinfocs-golang` | github.com/markus-wa/demoinfocs-golang | MIT | 字段语义参照（Go） | 不进入运行时 |
| `awpy` | github.com/pnxenopoulos/awpy | MIT | 算法口径验证（Python） | 不进入运行时 |
| `akiver/cs-demo-manager` | github.com/akiver/cs-demo-manager | MIT | 数据口径、雷达样式、录制 shim | 不 fork 其产品；仅取所需逻辑 |
| `advancedfx`（HLAE） | github.com/advancedfx/advancedfx | MIT（**子模块除外**） | demo 相机与 tick 命令 | **外部工具调用，不内嵌**；子模块需单独核许可 |
| `PixiJS` | github.com/pixijs/pixijs | MIT | 2D 画布 | — |
| `Three.js` | github.com/mrdoob/three.js | MIT | 3D 视图（M2） | — |
| `Konva` | github.com/konvajs/konva | MIT | 标注层 | — |
| `Tauri` | github.com/tauri-apps/tauri | MIT / Apache-2.0 | 桌面壳 | — |
| `React` | github.com/facebook/react | MIT | 前端 | — |
| `TypeScript` | github.com/microsoft/TypeScript | Apache-2.0 | 语言 | — |
| `Zustand` | github.com/pmndrs/zustand | MIT | 状态管理 | — |
| `FFmpeg` | ffmpeg.org | LGPL 2.1+（默认构建） | 编码 / 封装 | **仅启用 nvenc / amf / mf + aac**；启用 GPL 组件会使整个产品转为 GPL，禁止 |

## 2. 仅作参考，不复制代码

| 项目 | 许可证状态 | 参考内容 | 边界 |
|---|---|---|---|
| `ok2ju/oversite` | **未声明许可证** | PixiJS 组织方式、Go+Wails 结构 | 只读；如需复用先取得作者授权 |
| `huaxiangqiming/CSdemowatch` | **未声明许可证**（README 明示） | Replay V2 二进制缓存、高度剖切、从本机提取地图的做法 | 只读；如需复用先取得作者授权 |
| `Rovniy/cs2-highlights-maker` | 待核 | HLAE + shim 的录制编排时序 | 待许可核实后决定是否引用其代码 |
| `cs2.cam` | 闭源商业 | 产品形态与功能分级 | **未逆向、未抓取、未复制** |

## 3. 资产与内容

| 对象 | 权利人 | 我们的处理 |
|---|---|---|
| CS2 游戏本体、demo 格式 | Valve | 不分发；运行时读取用户本机安装 |
| 地图雷达图 / 地图几何 / 图标 | Valve | 不分发；运行时从本机只读提取并本地缓存 |
| `cs2-map-icons` 整理的 overview 数值 | 整理者 + Valve 原始资产 | 开发期参考；发布版改为本机提取 |
| demo 文件、POV 录制品 | 原权利人（HLTV / 选手 / 队伍） | 只存本地，不上传、不公开分享；`.dem` 与视频已在 `.gitignore` |

## 4. 风险提示

- **HLAE 与 VAC**：HLAE 官方声明其为"技术意义上的 hack"，加入受 VAC 保护的服务器大概率被封。本项目只在离线 demo 模式调用，并在 UI 中显著提示。
- **许可证传染**：新增任何依赖前先查许可证；CI 计划引入 `cargo deny` 与 `pnpm license-checker` 做自动扫描（详见路线图 S5 后的工程化任务）。
- **更正机制**：若你发现本项目使用了某组件却未在此登记，或对许可状态的判断有误，请开 issue，我们会修正并更新本文件。

*本文件随依赖变化更新，最近一次：2026-11-16（立项阶段，尚无代码依赖）。*
