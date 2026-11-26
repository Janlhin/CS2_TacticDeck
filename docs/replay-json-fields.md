# replay.json 字段对照草案

本文件是 T-01 导出产物 `replay.json` 的字段对照草案：字段路径、JSON 类型、单位与坐标系、取值约束、实测示例与语义说明。所有示例值均来自一次真实 Valve GOTV 录像的实跑导出，不是凭记忆或上游文档抄写。

| 项 | 值 |
|---|---|
| 导出命令 | `cargo run -p tacticdeck-probe -- --demo <file.dem> --out <replay.json> --step 8` |
| 校验命令 | `cargo run -p tacticdeck-probe -- --validate <replay.json>` |
| 实测 demo | `match730_003794784104164098297_0397976214_308.dem`（de_mirage，本机 CS2 replays 目录） |
| 导出退出码 | 0 |
| 校验退出码 | 0 |
| 产物大小 | 13,926,051 字节（单文件 JSON） |
| 导出统计 | 地图 `de_mirage`；10 名选手；24 回合；17,353 帧；138 条击杀；`unmappedKills=19`；`worldKills=18` |
| 对应契约 | Rust `crates/tacticdeck-probe/src/schema.rs`；前端 `src/replay/types.ts` |

## 顶层结构

```json
{
  "map": "de_mirage",
  "tickRate": 64.0,
  "players": [ /* PlayerMeta[] */ ],
  "rounds": [ /* RoundInfo[] */ ],
  "frames": [ /* Frame[] */ ],
  "events": [ /* KillEvent[] */ ],
  "meta": { /* ReplayMeta */ }
}
```

| JSON 路径 | Rust 字段（`schema.rs`） | JSON 类型 | 单位 / 口径 | 取值约束 | 实测示例 | 语义 |
|---|---|---|---|---|---|---|
| `.map` | `Replay.map: String` | string | — | 非空；读取失败时导出侧回退 `"unknown"` | `"de_mirage"` | 地图名，取自 `SvcServerInfo` 的 `map_name` |
| `.tickRate` | `tick_rate: f64` | number | tick / 秒 | 必须 > 0；`tick_interval` 缺失时兜底 64.0 | `64.0` | 服务器 tickrate；秒数 = `tick / tickRate` |
| `.players` | `players: Vec<PlayerMeta>` | object[] | — | 长度 10，槽位连续 1..10 | 10 项 | 花名册；T 侧在前（slot 1–5），CT 侧在后（6–10） |
| `.rounds` | `rounds: Vec<RoundInfo>` | object[] | — | 非空 | 24 项 | 回合列表，按事件发生顺序 |
| `.frames` | `frames: Vec<Frame>` | object[] | — | 非空；按 tick 升序 | 17,353 项 | 采样帧序列 |
| `.events` | `events: Vec<KillEvent>` | object[] | — | 无长度校验；导出时按 tick 升序排列 | 138 项 | 击杀事件 |
| `.meta` | `meta: ReplayMeta` | object | — | 导出时恒定存在（前端类型标为可选，见「已知差异」） | 见下 | 解析来源与归一化信息，主要供排查 |

## `players[]`（PlayerMeta）

| JSON 路径 | Rust 字段 | JSON 类型 | 单位 / 口径 | 取值约束 | 实测示例 | 语义 |
|---|---|---|---|---|---|---|
| `.players[].slot` | `PlayerMeta.slot: u8` | number | 槽位 | 1..10；数组第 `i` 项满足 `slot = i + 1` | `1` | 槽位编号，帧与事件里用同一 slot 引用该选手 |
| `.players[].name` | `name: String` | string | — | 非空 | `"山西石老板"` | 选手昵称，可含中文、数字与特殊字符 |
| `.players[].team` | `team: String` | string | 阵营枚举 | 仅 `"t"` / `"ct"`；team 字节 2→`t`，3→`ct` | `"t"` | 所属阵营 |

实测花名册（slot / team / name）：1 `t` 山西石老板；2 `t` 真的受不了；3 `t` 元蛋爱吃圆蛋；4 `t` Tisake；5 `t` kk；6 `ct` Janlhin；7 `ct` 散场不得已人山人海；8 `ct` Ryo；9 `ct` intel芯片研发工程师；10 `ct` 不要再打了。

## `rounds[]`（RoundInfo）

| JSON 路径 | Rust 字段 | JSON 类型 | 单位 / 口径 | 取值约束 | 实测示例 | 语义 |
|---|---|---|---|---|---|---|
| `.rounds[].index` | `RoundInfo.index: u32` | number | — | 0 起，按数组位置枚举 | `0` | 回合序号 |
| `.rounds[].startTick` | `start_tick: u32` | number | tick | ≤ `endTick` | `65` | 回合开始事件 tick（`round_start` / `round_prestart`） |
| `.rounds[].endTick` | `end_tick: u32` | number | tick | ≥ `startTick`；录像截断时回退到最后一条采样帧的 tick | `5735` | 回合结束事件 tick（`round_end` / `round_officially_ended`），含端点 |
| `.rounds[].winner` | `winner: Option<String>` | string \| null | 阵营枚举 | `"t"` / `"ct"` / `null`；校验器不校验取值 | `null` | 胜方阵营；事件字段取不到合法阵营时为 `null` |

实测：24 回合 `winner` 全部为 `null`；无 `endTick = 0` 的回合；无起止反序回合；相邻回合首尾相接（第 0 回合 `endTick=5735`，第 1 回合 `startTick=5735`）。

## `frames[]`（Frame）

| JSON 路径 | Rust 字段 | JSON 类型 | 单位 / 口径 | 取值约束 | 实测示例 | 语义 |
|---|---|---|---|---|---|---|
| `.frames[].tick` | `Frame.tick: u32` | number | tick | 非递减；相邻帧间隔为 `meta.tickStep`，掉帧补采处更大 | `72` | 该帧对应的 demo tick；首帧 72，末帧 138,896 |
| `.frames[].players` | `players: Vec<PlayerFrame>` | object[] | — | 长度 10 | 10 项 | 该帧十人状态 |

实测帧间隔：17,352 个间隔中 17,351 个为 8，1 个为 16（掉帧后补采）；无重复 tick。

## `frames[].players[]`（PlayerFrame）

| JSON 路径 | Rust 字段 | JSON 类型 | 单位 / 口径 | 取值约束 | 实测示例 | 语义 |
|---|---|---|---|---|---|---|
| `.slot` | `PlayerFrame.slot: u8` | number | 槽位 | 1..10，对应 `players[].slot` | `1` | 选手槽位 |
| `.x` | `x: f32` | number | 归一化坐标 | [0, 1]，左下原点 | `0.94897723` | 板面横坐标 |
| `.y` | `y: f32` | number | 归一化坐标 | [0, 1]，左下原点 | `0.31775624` | 板面纵坐标（y 轴已翻转） |
| `.dir` | `dir: f32` | number | 度 | 实测 0.001..359.999；取自眼睛偏航角 yaw | `270.0` | 朝向角 |
| `.hp` | `hp: i32` | number | 血 | 实测 0..100 | `100` | 血量 |
| `.alive` | `alive: bool` | boolean | — | 实测与 `hp = 0` 完全对应 | `true` | 是否存活 |

实测分布（173,530 个玩家行 = 17,353 帧 × 10）：

| 字段 | 实测范围 / 分布 |
|---|---|
| `hp` | 0：48,777；1–25：5,682；26–50：6,963；51–75：5,592；76–100：106,516 |
| `alive` | `true`：124,753；`false`：48,777；`alive = (hp > 0)`，不一致 0 条 |
| `dir` | 0.001..359.999 |
| `x` | 0..1 |
| `y` | 0.0705..0.9295 |

## `events[]`（KillEvent）

| JSON 路径 | Rust 字段 | JSON 类型 | 单位 / 口径 | 取值约束 | 实测示例 | 语义 |
|---|---|---|---|---|---|---|
| `.events[].tick` | `KillEvent.tick: u32` | number | tick | 事件发生时的 demo tick | `2475` | 击杀时刻 |
| `.killer` | `killer: u8` | number | 槽位 | 1..10 | `1` | 击杀者槽位 |
| `.victim` | `victim: u8` | number | 槽位 | 1..10 | `6` | 受害者槽位 |
| `.headshot` | `headshot: bool` | boolean | — | — | `true` | 是否爆头 |

实测：138 条事件按 tick 正序；槽位越界 0 条；`headshot=true` 58 条，`false` 80 条。首条 `{"tick":2475,"killer":1,"victim":6,"headshot":true}`，末条 `{"tick":137623,"killer":4,"victim":6,"headshot":false}`。

## `meta`（ReplayMeta）

| JSON 路径 | Rust 字段 | JSON 类型 | 单位 / 口径 | 取值约束 | 实测示例 | 语义 |
|---|---|---|---|---|---|---|
| `.meta.source` | `source: String` | string | — | 导出时写入的 demo 路径，仅供排查 | `D:\SteamLibrary\...\match730_003794784104164098297_0397976214_308.dem` | 来源录像路径（命令行原样回写） |
| `.meta.tickStep` | `tick_step: u32` | number | tick | 等于 `--step` 参数；默认 8；内部下限 1 | `8` | 帧采样间隔 |
| `.meta.normalize` | `normalize: String` | string | 口径枚举 | `"radar"` / `"bbox-uniform"` | `"bbox-uniform"` | 坐标归一化口径，见下节 |
| `.meta.bounds` | `bounds: [f32; 4]` | number[] | Hammer 单位 | `[minX, minY, maxX, maxY]`；无采样时回退 `[0, 0, 1, 1]` | `[-2633.4324, -2596.4758, 1422.9688, 887.9684]` | 世界坐标包围盒（未归一化） |
| `.meta.controllers` | `controllers: u32` | number | 人 | 等于 `players.length` | `10` | 识别到的控制器数量 |
| `.meta.unmappedKills` | `unmapped_kills: u32` | number | 起 | — | `19` | 击杀双方有一方映射不到花名册槽位 |
| `.meta.worldKills` | `world_kills: u32` | number | 起 | — | `18` | 环境致死或自杀（attacker=0 或 attacker=victim） |

死亡事件对账：`events.length` 138 + `unmappedKills` 19 + `worldKills` 18 = 175 起 `player_death`。

## 坐标口径

### 世界坐标两段式还原

Source 2 把世界坐标拆成「格子编号 + 格内偏移」两段存储，还原公式（`probe.rs` 的 `cell_to_world`）：

```
world = cell * 512 - 16384 + vec
```

- 格子边长 `CELL_SIZE = 512` Hammer 单位；世界原点偏移 `WORLD_HALF = 16384`。
- `cellX/cellY` 与 `vecX/vecY` 取自 pawn 实体的 `m_skeletonInstance.m_vecOrigin`。

### 归一化两条路径

| `normalize` | 触发条件 | 换算（`main.rs` / `map_profile.rs`） | 特点 |
|---|---|---|---|
| `"bbox-uniform"` | 未提供 `--map-profile`，或该地图无标定参数 | `span = max(maxX-minX, maxY-minY, 1)`；`nx = 0.5 + (x-cx)/span`，`ny = 0.5 - (y-cy)/span` | 等比例不拉变形；坐标对不上雷达图 |
| `"radar"` | 提供 `--map-profile` 且命中该地图 | `px = (wx-pos_x)/scale`，`py = (pos_y-wy)/scale`；`nx = px/imageSize`，`ny = 1 - py/imageSize` | 对齐雷达盘面；本次产物未走该分支 |

输出坐标一律落在 0..1，**左下角为原点**，y 轴已翻转。`meta.bounds` 是还原后的世界坐标（Hammer 单位），不是归一化坐标。

## tick 语义

| 概念 | 口径 |
|---|---|
| `tickRate` | 服务器 tickrate，取自 `tick_interval` 的倒数；缺失兜底 64.0。秒 = `tick / tickRate` |
| `frames[].tick` | demo 原始 tick 号；采样只在 `tick % tickStep == 0` 的 tick 上发生 |
| `meta.tickStep` | 回写的采样间隔，对应 `--step`；默认 8，下限 1 |
| `events[].tick` | 击杀事件发生时的 demo tick |
| `rounds[].startTick` / `endTick` | 回合起止事件 tick，含端点；同一套 tick 时间基 |

## `--validate` 校验清单

以下规则逐条对应 `main.rs` 的 `validate_replay` 实现；「失败条件」即触发 `anyhow::ensure!` 的判定与报错信息。校验器只读 JSON 反序列化后的结构，不重跑解析。

| # | 检查项 | 失败条件 | 报错信息 |
|---|---|---|---|
| 1 | JSON 可按 `Replay` 结构反序列化 | 类型不符、缺字段或 JSON 非法 | `replay.json 结构不合法` |
| 2 | `map` 非空 | `map.is_empty()` | `map 为空` |
| 3 | `tickRate` 为正 | `tick_rate <= 0.0` | `tickRate 必须为正` |
| 4 | 选手人数为 10 | `players.len() != 10` | `选手必须 10 人，实际 N` |
| 5 | 槽位连续 1..10 | 第 `index` 项 `slot != index + 1` | `槽位应连续 1..10，第 index 项是 slot` |
| 6 | 选手名字非空 | `name.is_empty()` | `选手 slot 名字为空` |
| 7 | 阵营合法 | `team` 不是 `"t"` 也不是 `"ct"` | `选手 slot 阵营非法` |
| 8 | 回合列表非空 | `rounds.is_empty()` | `没有回合` |
| 9 | 帧列表非空 | `frames.is_empty()` | `没有帧` |
| 10 | 帧 tick 非递减 | `frame.tick < last_tick` | `帧 tick 必须递增，tick 出现在 last 之后` |
| 11 | 每帧十人 | `frame.players.len() != 10` | `tick t 只有 n 名选手` |
| 12 | 帧内坐标在 [0,1] | `x` 或 `y` 落在 [0,1] 之外 | `tick t 槽位 s 坐标越界 (x, y)` |
| 13 | 回合起止合法 | `start_tick > end_tick` | `回合 index 起止 tick 反了` |
| 14 | 事件槽位合法 | `killer` 或 `victim` 不在 1..10 | `击杀事件槽位越界` |

校验器**不检查**：事件是否正序、`winner` 取值是否合法、`hp` / `dir` 范围、`meta` 各字段一致性、`bounds` 四元组是否自洽。

## 与前端契约的对应关系

前端类型定义在 `src/replay/types.ts`，选择器在 `src/replay/selectors.ts`。字段名经 serde `rename_all = "camelCase"` 输出，下表为 Rust 字段到 JSON / TS 的映射。

| Rust 结构 | TS 类型 | 字段映射 |
|---|---|---|
| `PlayerMeta` | `PlayerMeta` | `slot` → `slot`；`name` → `name`；`team` → `team` |
| `PlayerFrame` | `PlayerFrame` | `slot` → `slot`；`x` → `x`；`y` → `y`；`dir` → `dir`；`hp` → `hp`；`alive` → `alive` |
| `Frame` | `Frame` | `tick` → `tick`；`players` → `players` |
| `RoundInfo` | `RoundInfo` | `index` → `index`；`start_tick` → `startTick`；`end_tick` → `endTick`；`winner` → `winner` |
| `KillEvent` | `KillEvent` | `tick` → `tick`；`killer` → `killer`；`victim` → `victim`；`headshot` → `headshot` |
| `ReplayMeta` | `ReplayMeta` | `source` → `source`；`tick_step` → `tickStep`；`normalize` → `normalize`；`bounds` → `bounds`；`controllers` → `controllers`；`unmapped_kills` → `unmappedKills`；`world_kills` → `worldKills` |
| `Replay` | `Replay` | `map` → `map`；`tick_rate` → `tickRate`；`players` / `rounds` / `frames` / `events` / `meta` 同名 |

前端选择器消费 `frames`、`rounds`、`events`、`players`、`tickRate`；不读取 `meta`（`meta` 仅供排查与后续口径判断）。

## 已知差异

| # | 位置 | 现状 | 说明 |
|---|---|---|---|
| D-1 | `Replay.meta` 可选性 | TS `Replay.meta?: ReplayMeta` 标可选；Rust 导出恒定写入 `meta` | 前端内置合成样本可以不带 `meta`；probe 产物一定带 |
| D-2 | `winner` 类型收窄 | TS 为 `TeamSide \| null`（`"t"` / `"ct"` / `null`）；Rust 为 `Option<String>` | 本份实测 24/24 为 `null`，未观察到非 null 样本；事件字段解析出合法阵营时才写入 |
| D-3 | `normalize` 注释滞后 | TS 注释写「目前是 `bbox-uniform`，T-02 换雷达校准」；代码已支持 `"radar"`（带 `--map-profile` 时） | 注释未随 T-02 更新；两种取值以上节「坐标口径」为准 |
| D-4 | `dir` 旋转方向 | TS 注释写「0 表示朝右、逆时针为正」；`dir` 取自 Source 眼睛偏航角 yaw | 角度单位与 0 朝向东一致；旋转正方向待与游戏内视角复核，暂不改动类型注释 |
| D-5 | 数值精度 | Rust：`x/y/dir` 为 `f32`、`tickRate` 为 `f64`、`hp` 为 `i32`、槽位/计数为 `u8` / `u32`；TS 一律 `number` | JSON 不携带精度信息，前端按 number 处理 |
| D-6 | POV 录像限制 | 指定样例 `g161-20250128230749590167333_de_mirage.dem` 是个人 POV 录像：实跑仅识别到 1 名控制器、0 条帧，`--validate` 退出 1（`选手必须 10 人，实际 1`） | 当前契约面向 GOTV 十人录像；POV 录像只有观察者自身是 controller，不在本草案覆盖范围内 |
| D-7 | `radar` 口径未观测 | 本次导出未带 `--map-profile`，`normalize` 实测为 `bbox-uniform` | `"radar"` 分支由代码路径确定，未在本次产物中采样 |

## 附：本次实测统计快照

| 指标 | 值 |
|---|---|
| `map` | `de_mirage` |
| `tickRate` | `64.0` |
| `players` | 10 |
| `rounds` | 24 |
| `frames` | 17,353 |
| `events` | 138 |
| `meta.tickStep` | 8 |
| `meta.normalize` | `bbox-uniform` |
| `meta.bounds` | `[-2633.4324, -2596.4758, 1422.9688, 887.9684]` |
| `meta.controllers` | 10 |
| `meta.unmappedKills` | 19 |
| `meta.worldKills` | 18 |
| 首帧 / 末帧 tick | 72 / 138,896 |
| `winner` 为 null 的回合 | 24 / 24 |
| `hp` 范围 | 0..100 |
| `dir` 范围 | 0.001..359.999 |
| `x` / `y` 范围 | x 0..1；y 0.0705..0.9295 |
