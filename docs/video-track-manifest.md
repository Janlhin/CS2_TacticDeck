# 录像轨清单（T-07）

本文件描述 T-07「视频条：N 路封面 + 懒加载 + 点击切主」的清单契约、注入方式、懒加载与切主规则，以及当前限制。对应前端代码在 `src/replay/tracks.ts`（纯逻辑）、`src/components/VideoBar.tsx`（封面条与主视频区）、`src/store/appStore.ts`（清单状态与选择器）。

本轮只到可视化与逻辑层：不加载任何真实封面或视频，清单里的 `cover` / `video` 都是占位路径字符串，测试与浏览器预览由假清单驱动。

## 字段契约

### `RecordingTrack`（单路录像轨）

| 字段 | 类型 | 单位 / 口径 | 取值约束 | 语义 |
|---|---|---|---|---|
| `slot` | number | 槽位 | 整数，1..10；同一清单内不重复 | 选手槽位，与 `PlayerMeta.slot` 同源 |
| `cover` | string | 路径或 URL | 非空字符串 | 封面图地址；本轮为占位字符串，仓库内不落地真实封面素材 |
| `video` | string | 路径或 URL | 非空字符串 | 录像文件地址；本轮不挂载真实视频流 |
| `startTick` | number | tick | 整数；严格小于 `endTick` | 该路在共享 tick 时间基上的起点 |
| `endTick` | number | tick | 整数；含端点 | 该路在共享 tick 时间基上的终点 |

### `TrackManifest`（一回合清单）

| 字段 | 类型 | 单位 / 口径 | 取值约束 | 语义 |
|---|---|---|---|---|
| `roundIndex` | number | 回合序号 | 非负整数；与 `RoundInfo.index` 对齐 | 本清单对应的回合 |
| `tickRate` | number | tick / 秒 | 正整数 | 服务器 tickrate；口径沿用回放元数据 |
| `tracks` | `RecordingTrack[]` | — | 长度 1..10；允许少于 10 路 | 本回合已有的录像轨列表 |

## 时间口径

沿用 `src/replay/types.ts` 的单一时间基：轨与播放头一律用整数 tick 对齐，`tickRate` 只做参考换算。本模块不出现第二套「秒」换算，对齐只发生在后续同步层。

## 注入方式

清单数据源是可注入的抽象，前端不绑定具体取数通道：

- `TrackSource` 接口：`{ list(roundIndex: number): Promise<TrackManifest> }`。未来由 Rust 桥或本地文件实现。
- `createStaticSource(manifest)`：返回直接吐内存清单的静态来源，供浏览器预览与无 DOM 单测使用。
- store 注入入口：
  - `setTrackSource(source)`：保存来源，不立刻拉取。
  - `fetchTracks(roundIndex)`：调来源取清单，经 `parseTrackManifest` 校验后落库。
  - `setTrackManifest(manifest)`：直接写入已校验清单（测试与静态预览走这条）。
  - `activeTrack()`：按当前主视图槽位从索引取轨，查不到返回 `undefined`。

### 结构校验

`parseTrackManifest(raw)` 在边界处把未知结构校验成 `TrackManifest`，失败抛出带定位信息的可读错误，不静默吞成默认值。校验项：

- 顶层为对象；`roundIndex` 为非负整数；`tickRate` 为正整数。
- `tracks` 为数组，长度 1..10（N=0 与 N=11 均被拒）。
- 每条 `slot` 在 1..10 且不重复；`cover` / `video` 为非空字符串；`startTick`、`endTick` 为整数且 `startTick < endTick`。

## N 可配范围

`tracks` 长度 N 取 1..10，允许少于 10 路：清单只列本回合实际采集到的轨，未覆盖的槽位在封面条里不出现，主视图落到这类槽位时显示「当前主视角暂无录像轨」。空清单（N=0）与超过 10 路（N=11）在解析层拒绝。

## 懒加载规则

封面与视频元素的挂载策略都是纯函数，便于无 DOM 单测：

- `visibleCoverRange(scrollTop, itemHeight, viewportHeight, count, overscan)`：按滚动位置返回应请求封面的下标闭区间 `{ start, end }`；`end = -1` 表示当前无需请求封面。区间外加上下 `overscan` 缓冲行，结果夹在 `0..count-1`。
- 封面条由 `onScroll` + `ResizeObserver` 把容器滚动偏移与实测宽度喂给 `visibleCoverRange`，再经 `sliceTracksByRange` 只渲染区间内封面；区间外的路数不挂载 DOM，不是渲染后再隐藏。单项宽度 `COVER_ITEM_WIDTH = 120`、缓冲数 `COVER_OVERSCAN = 1` 是组件模块级常量，与 `styles.scss` 中封面 `flex-basis` 写死同一数值。`<img loading="lazy" decoding="async">` 的原生懒加载只负责图片解码时机，不承担区间裁剪。
- `mountedVideoSlots(index, activeSlot)`：返回本轮应挂载 `<video>` 的 slot 列表，只允许主视图那一路，最多 1 个；未注入清单、主槽不在清单内或主槽为总览（0）时返回空数组。
- 主轨 `<video>` 固定 `playsInline`、`preload="none"`，`poster` 取该轨封面；未选中的路数只留封面，不挂视频元素。

## 切主规则

- 切主的唯一动作仍是 store 的 `setActiveView(id)`；点击封面、花名册、左栏槽位最终都走它。
- 索引 `trackIndex`（`slot → 轨` 的 `Map`）只在清单注入时由 `buildTrackIndex` 构建一次；切主路径只做 `Map` 查表，不重新解析清单、不重排列表。
- 切主耗时口径：测试用 `performance.now()` 夹住「纯函数选轨 + store 切主 + 取选择器」全过程，重复 100 次，断言单次 `< 1000 ms`。当前 bun 本地单测环境实测最大值 `0.012 ms`；该数值只代表假清单下的常数级查表开销，不代表真实媒体加载耗时。

## 当前限制

- **无真实素材**：仓库内不提交封面与视频文件，清单里的 `cover` / `video` 是占位字符串；本轮没有真实封面图、没有真实录像文件，也未验证媒体解码与播放。
- **录制来源未决（Q-01）**：本地 HLAE + 自建抓屏，还是服务端录制，尚未裁决；录制品的产出通道、分辨率与存储位置都依赖该结论。
- **POV 单人契约未决（Q-06）**：个人 POV 录像只有 1 名 controller，现有十人 `replay.json` 契约覆盖不了；单人槽位与降级 schema 的走向尚未裁决。
- **未做播放对齐**：视频元素本轮 `muted`、不自动播放，未接播放头同步、seek 对齐与音画同步；这些属于后续同步层职责。
- **视口预判未接**：区间渲染已接线并有单测覆盖（滚动事件 + `ResizeObserver` 实测容器宽度）；尚未做 `IntersectionObserver` 级的视口预判，无法在封面临近可视区前提前准备。

## 与代码的对应关系

| 概念 | 代码位置 |
|---|---|
| 清单类型与校验 | `src/replay/tracks.ts`：`RecordingTrack`、`TrackManifest`、`parseTrackManifest` |
| 可注入来源 | `src/replay/tracks.ts`：`TrackSource`、`createStaticSource` |
| 索引与查表 | `src/replay/tracks.ts`：`buildTrackIndex`、`selectTrack` |
| 懒加载 / 挂载策略 | `src/replay/tracks.ts`：`visibleCoverRange`、`sliceTracksByRange`、`mountedVideoSlots` |
| 清单状态与选择器 | `src/store/appStore.ts`：`trackManifest`、`trackIndex`、`setTrackSource`、`setTrackManifest`、`fetchTracks`、`activeTrack` |
| 封面条与主视频区 | `src/components/VideoBar.tsx` |
| 纯逻辑回归 | `tests/video-bar.test.ts` |

工程规范见 [engineering-standards.md](engineering-standards.md)。
