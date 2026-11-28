# CSDM 代码复用清单

本文件回答一个问题：**从 `cs-demo-manager` 拿什么、怎么拿、拿回来要改哪里**。只写清单与边界，不含代码。

| 项 | 值 |
|---|---|
| 上游 | <https://github.com/akiver/cs-demo-manager> |
| 许可证 | MIT（可商用、可修改、可再分发；义务是保留版权声明与许可全文） |
| 锁定快照 | `b0d3258cfdb435eb6707ed8fee771774c450aa11`（2026-11-16） |
| 本地参考克隆 | `D:\Projects\_tmp\csdm-ref`（只读参考，不入库、不提交） |
| 登记日期 | 2026-11-18 |
| 当前状态 | 清单已定，**尚未搬运任何代码** |

一句话：**能搬、值得搬的是"纯逻辑 + 2D 板面"；不能搬的是游戏内插件与 Electron 主进程。**

## 1. 合规动作（搬第一批之前先做完，一次管全部批次）

1. 每个搬来的文件顶部保留原版权头，并加一行：`Adapted from akiver/cs-demo-manager (MIT) @ b0d3258`。
2. 本仓库根 `NOTICE` 与 `docs/third-party-notices.md` 已登记 MIT 许可；批次落地时在该表补一列"实际搬运文件清单"，指向本文件。
3. `README.md` 致谢补一段：来源仓库、借用范围、锁定提交。
4. 不改上游版权人姓名、不用其名做产品或商标；我们的产品名仍是 CS2 TacticDeck。
5. 上游若有同名商标/图标，**只搬代码不搬视觉资产**。

## 2. 批次 A：序列编排（纯函数，风险最低，建议第一个做）

| 文件（`src/common/video/...`） | 行数 | 作用 | 搬回来要改什么 |
|---|---|---|---|
| `sequences/build-players-event-sequences.ts` | 147 | 按"击杀/死亡"事件切片段：前后各留 N 秒、相邻事件过近则合并 | 无需改逻辑；入参从 `match.kills` 映射到我们的回放 JSON |
| `sequences/build-players-rounds-sequences.ts` | 77 | 按回合切片段（整回合录制用） | 同上 |
| `sequences/build-players-options.ts` | 38 | 生成每名玩家的录制选项（击杀提示、高亮、语音） | 语音开关映射到我们录制侧 |
| `sequences/sort-sequences-by-start-tick.ts` | 15 | 按起始 tick 排序 | 无 |
| `sequences/sort-sequences-by-number.ts` | 5 | 按序号排序 | 无 |
| `sequences/get-next-sequence-number.ts` | 12 | 续编号 | 无 |

配套类型（同批搬走，否则编译不过）：

| 文件（`src/common/types/`） | 行数 |
|---|---|
| `sequence.ts` | 24 |
| `sequence-player-options.ts` | 10 |
| `player-sequence-event.ts` | 11 |
| `perspective.ts` | 10 |
| `kill.ts` | 43 |
| `match.ts` | 73 |
| `match-player.ts` | 68 |
| `video.ts` | 47 |
| `video-container.ts` | 12 |
| `camera.ts` | 19 |
| `custom-camera-focus.ts` | 6 |
| `player-camera-focus.ts` | 5 |
| `recording-system.ts` | 10 |
| `recording-output.ts` | 11 |

改动点只有四类：

1. **导入路径**：`csdm/common/...` 改成我们自己的别名（如 `@/replay/types/...`）。
2. **设置类型**：`VideoSettings` / `FfmpegSettings` 来自 `csdm/node/settings/settings`，必须换成我们的设置模块；实际只用到 6 个字段（`showOnlyDeathNotices` / `deathNoticesDuration` / `showXRay` / `showAssists` / `recordAudio` / `playerVoicesEnabled`）。
3. **时间基**：他们全程用 tick 与 `match.tickrate`，与我们 `crates/tacticdeck-sync` 的 tick 口径一致，不做换算。
4. **事件字段**：`kill.tick` / `killerSteamId` / `victimSteamId` / `weaponName`，与我们导出的 `replay.json` 事件字段对齐后再接。

## 3. 批次 B：2D 板面（`src/ui/match/viewer-2d/`）

规模：**71 个文件 / 5064 行**，是整个项目里最值钱的一块。

| 子目录 | 文件数 | 内容 | 我们怎么用 |
|---|---|---|---|
| `drawing/` | 14 | 雷达底图、玩家点位、死亡连线、弹道、手雷轨迹、燃烧瓶范围、炸弹与拆包、鸡、人质 | 直接补我们的 2D 固定面板 |
| `playback-bar/` | 21 | 时间轴、播放/暂停、倍速、上下回合、绘制开关、全屏、计时器 | 对应我们的回合条与播放控件 |
| `teams-panel/` | 7 | 十人队伍面板（血量、装备、语音状态） | 正对 D-014 的左栏 11 槽 |
| `bomb-damage/` | 8 | C4 伤害区估算与图示（含 3 个单测） | 复盘时标出致死范围 |
| `audio/` | 2 | 语音音轨加载与偏移对齐 | 与 demo 语音导出配合 |
| `rounds-bar/` | 2 | 回合选择条 | 接入我们回合列表 |
| 其它 | 17 | reducer / context / 全屏 provider / 时间索引 / 视图状态机 | `viewer-2d-reducer.ts` 可直接支撑 T-16 自动切 2D |

宿主依赖只有 **2 个文件**，说明这块几乎没有 Electron 味道：

- `playback-bar/audio-selector-button.tsx`
- `viewer-2d-loader.tsx`

这两个要改成 Tauri command 或直接读本地文件，其余 69 个文件可原样搬。

必须一起搬的底座（都在 `src/ui/hooks/`，**0 宿主依赖**）：

| 文件 | 行数 |
|---|---|
| `use-map-canvas.ts` | 146 |
| `use-interactive-map-canvas.ts` | 248 |

雷达资源与他们一致：运行时从本机 CS2 读取（对应我们已完成的 T-17）。

**渲染选择**：他们用原生 canvas 2D，不依赖 pixi。我们若坚持 pixi，需要重写全部绘制逻辑；建议先原样 canvas 跑通，再评估是否换成 pixi（见第 7 节待决）。

## 4. 批次 C：热图（可选，M2 再做）

`src/ui/components/heatmap/*`（前端结构与交互）+ `src/node/database/heatmap/*`（SQL 聚合口径）。后者要改成 Rust 侧聚合或前端计算，属于"借口径不借实现"。

## 5. 不搬清单（附原因，写死在这，避免以后有人手滑）

| 路径 | 原因 |
|---|---|
| `cs2-server-plugin/**` | 依赖 `alliedmodders/s2sdk`，该仓库**无许可证**（Unofficial Source 2 SDK），法务不可控；且它靠游戏内插件画回放 UI，我们不需要 |
| `csgo-server-plugin/**` | 同上，依赖 `alliedmodders/hl2sdk` |
| `src/node/**`、`src/preload/**`、`src/server/**` | Electron 主进程 / IPC / 本地 HTTP 服务，与 Tauri 架构冲突 |
| `package.json`、`electron-builder.config.mjs`、`crowdin.yml`、`lingui.config.ts` | 构建与本地化体系不同（我们是 Bun + Tauri + 自有文案） |

## 6. 与 `TASKS.md` 的映射

| 我们的任务 | 用哪批 |
|---|---|
| T-07 视频条 | 批次 A 的 `Sequence` 模型（起止 tick + 序号 + 机位） |
| T-09 LiteReplay 改造（录制编排） | 批次 A + HLAE `mirv_streams` |
| 右栏 2D 固定板（D-014） | 批次 B `drawing/` + 两个 canvas hook |
| T-16 录制完成自动切 2D | 批次 B `viewer-2d-reducer.ts` / `viewer-2d-loader.tsx` |
| 十人视角轮换（滚轮换人） | 批次 A 的 `playerCameras`：每条含 `tick` + 目标 `playerSteamId`，天然支持"按 tick 切锁人"，可映射到我们的 10 槽 |
| timing heatmap（未排期） | 批次 C |

## 7. 待决与风险

- **待决 Q-A**：2D 板面接受原生 canvas 吗？若要用 pixi，等于重写 5064 行里的绘制部分，工作量翻倍。
- **待决 Q-B**：TypeScript 侧的 `Match` / `Kill` 模型，是直接采用 CSDM 的形状（改我们导出层适配它），还是在 `src/replay/` 建适配层？建议后者，避免被上游字段牵着走。
- **风险 R-A**：上游迭代快（本次快照前一天还有提交）。策略是"搬一次、锁 sha、不长期跟"；升级只在确实需要新能力时做。
- **风险 R-B**：`video.ts` 里的 `trueView` 对应 CS2 官方 TrueView，开/关会让录制画面与选手当时所见不一致，录制参数里必须显式定死并记录。
- **风险 R-C**：`playerCameras` 的切换粒度是 tick，切人瞬间若与我们的 `TickClock` 校正冲突，会出现"人跳了一下"；接入时对齐到 5% 小偏差策略之外，不允许在切人时做大跳。
