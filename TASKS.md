# TASKS.md

任务队列与待裁决台账。**每条完成判据必须是可机器判定的命令 + 期望退出码**；已决条目不删除，保留可追溯。

分组按功能面；状态取值：`未开始` / `进行中` / `完成` / `阻塞`。

---

## 工程基础

| 编号 | 目标 | 完成判据（命令 → 退出码） | 状态 | 创建时间 | 完成时间 |
|---|---|---|---|---|---|
| W-01 | 建立五个文档与根目录白名单 | `bun run verify` → 0（含五文档存在、字段非空、根目录比对） | 完成 | 2026-11-16 | 2026-11-16 |
| W-02 | 唯一验收入口 `scripts/verify.ps1` | `bun run verify` → 0 | 完成 | 2026-11-16 | 2026-11-16 |
| W-03 | CI 每次推送与 PR 运行 verify | GitHub Actions 上 `verify` 步骤绿色（`gh run list` 首条 `conclusion=success`）→ 已验（run `43864689689`） | 完成 | 2026-11-16 | 2026-11-16 |
| W-04 | 提交门禁（commitlint + 钩子转发） | 故意提交 `tmp: bad message` → 非 0；合法 `docs: x` → 0 | 完成 | 2026-11-16 | 2026-11-16 |
| W-05 | 脚本基线清零（`.cmd` / `.bat` 计数 = 0） | `git ls-files '*.cmd' '*.bat'` 空输出 → 0 | 完成 | 2026-11-16 | 2026-11-16 |
| W-06 | 许可扫描接入 CI（`cargo deny` / 依赖许可检查） | `cargo deny check licenses` → 0（在首个 `Cargo.toml` 落地后启用） | 未开始 | 2026-11-16 | — |
| W-07 | 技术栈骨架：Tauri 2 壳 + React 19 前端 + Pixi 画布占位 | `bun run verify` → 0（含 rust 的 fmt / clippy / test 三段）；`vite build` → 0 | 完成 | 2026-11-16 | 2026-11-16 |
| W-08 | 前端工具链由 Deno 2.9.8 切到 Bun 1.4.4（Deno 官方 2026-10-09 宣布一年后停止运行时开发） | `bun run verify` → 0；`bun run test:web` → 0；`bun run build:web` → 0 | 完成 | 2026-11-18 | 2026-11-18 |

## 参考源码取证

| 编号 | 目标 | 完成判据（命令 → 退出码） | 状态 | 创建时间 | 完成时间 |
|---|---|---|---|---|---|
| R-01 | 六个参考仓库浅克隆到仓库外（`D:\Projects\_refs`），并在 `MANIFEST.md` 登记 HEAD 与许可 | 目录存在 + MANIFEST 行数与仓库数一致 → 人工核对通过 | 完成 | 2026-11-16 | 2026-11-16 |
| R-02 | 读完 `demoparser` 的 tick 列式输出与 `awpy` 的坐标换算，落成 `replay.json` 字段草案 | 字段草案写入 `docs/`，且 T-01 可直接按草案实现 | 未开始 | 2026-11-16 | — |

## S0 解析与地图校准

| 编号 | 目标 | 完成判据（命令 → 退出码） | 状态 | 创建时间 | 完成时间 |
|---|---|---|---|---|---|
| T-01 | 封装 `source2-demo`，产出 `replay.json`（rounds / ticks / events / players） | `cargo run -p tacticdeck-probe -- --demo <f> --out replay.json` → 0，且 `cargo run -p tacticdeck-probe -- --validate replay.json` → 0 | 完成 | 2026-11-16 | 2026-11-17 |
| T-02 | 地图 overview 提取（本机 CS2 → `map-profile.json`） | `cargo run -p tacticdeck-probe -- --extract-maps` → 0，`de_mirage` 的 `pos_x/pos_y/scale/rotate` 与已知值一致 | 完成 | 2026-11-16 | 2026-11-17 |
| T-03 | 坐标校准（Source 2 两段式世界坐标 → 雷达像素），Mirage 有真实 demo 复核 | `cargo test -p tacticdeck-probe coords` → 0 | 完成 | 2026-11-16 | 2026-11-17 |
| T-03b | 其余地图的坐标复核（10 张地图各一份真实 Valve demo 跑通） | 每张地图的 `cargo run -p tacticdeck-probe -- --validate replay.json` → 0，`--check-radar` 命中率 ≥98%，且 `tests/replay-contract.test.ts` 仍 0 | 完成 | 2026-11-17 | 2026-11-17 |

### T-03b 实测记录（2026-11-17，本机 `game/csgo/replays` 的 225 份 Valve demo）

每张地图取一份 demo，`--step 16` + 雷达标定导出后跑 `--validate` 与 `--check-radar`（命中率 = 十人点位落在雷达图不透明像素上的比例）：

| 地图 | 回合 | 帧 | 击杀 | 雷达命中率 |
| --- | --- | --- | --- | --- |
| de_dust2 | 18 | 6502 | 115 | 100.0% |
| de_inferno | 10 | 3410 | 50 | 100.0% |
| de_cache | 14 | 5058 | 77 | 99.9% |
| de_mirage | 17 | 4995 | 95 | 99.8% |
| de_overpass | 44 | 8214 | 124 | 99.7% |
| de_train | 14 | 5031 | 72 | 99.1% |
| de_nuke | 6 | 3168 | 40 | 98.9% |
| de_anubis | 10 | 3658 | 54 | 98.6% |
| de_vertigo | 46 | 6889 | 139 | 97.5% |
| de_ancient | 7 | 2680 | 29 | 97.1% |

同时修掉三个只在真实录像里出现的问题：① CS2 demo 只有 `round_prestart` / `round_officially_ended`，老代码只认 `round_start` / `round_end`，导致回合数 0；② 部分 demo 的 `controller → pawn` 句柄为空，加了按 `m_hController` 反查的兜底；③ 热身帧、掉线帧和个别读歪的实体帧会让坐标越界，导出时只保留「十人齐全且坐标在 0..1」的帧。

## S1 2D 画布

| 编号 | 目标 | 完成判据（命令 → 退出码） | 状态 | 创建时间 | 完成时间 |
|---|---|---|---|---|---|
| T-04 | Pixi 底图 / 点位 / 朝向 / 轨迹 | `bun run test:web` → 0（2d 用例：坐标与朝向可用、相邻帧位移有限），窗口内底图与十人轨迹可见 | 完成 | 2026-11-16 | 2026-11-17 |
| T-05 | 回合条 / 事件轴 / 播放头（tick 单位） | `bun run test:web` → 0（timeline 用例：跳转 < 200 ms），窗口内 R1–R3 回合条与滑杆可见 | 完成 | 2026-11-16 | 2026-11-17 |
| T-13 | 左栏槽位列表：1 个固定 2D 总览 + 10 人 POV 槽（滚轮滚动、点选切主视图），并接本机 demo 库与十人绑定入口 | `bun run test:web` → 0（rail 用例：11 槽、轮换、绑定计数；library 用例：条目过滤、字段校验、十槽绑定）；窗口右栏「demo 库」可列出本机回放并载入 | 完成 | 2026-11-16 | 2026-11-17 |
| T-14 | 右栏分析与联动：回合/事件列表、热图、玩家面板随播放头更新 | `bun run test:web` → 0（panels 用例：本回合事件正序/置灰、落点限定回合、热图归一化与格子定位） | 完成 | 2026-11-16 | 2026-11-17 |
| T-15 | 十人同屏 2D 回放：10 名选手的点位 / 朝向 / 血量 / 阵营配色 | `bun run test:web` → 0（ten-player 用例：任意 tick 取到 10 个坐标，两队配色可区分，缺 1 人即失败） | 完成 | 2026-11-16 | 2026-11-17 |
| T-17 | 真实雷达底图：运行时从本机 CS2 VPK 解码 `overheadmaps/*.vtex_c`，2D 板面铺真底图；取不到时退回网格 | `cargo test -p tacticdeck-map` → 0（VTEX 头 / DATA 块 / RGBA 解码 / PNG 写出）；`cargo run -q -p tacticdeck-probe -- --extract-radar de_mirage --out <png>` → 0 且 1024×1024 盘面可辨；窗口内底图与十人点位对齐 | 完成 | 2026-11-17 | 2026-11-17 |

## S2 同步

| 编号 | 目标 | 完成判据（命令 → 退出码） | 状态 | 创建时间 | 完成时间 |
|---|---|---|---|---|---|
| T-06 | `TickClock`：主时钟 + 漂移回归 + `playbackRate` 校正 | `cargo test -p tacticdeck-sync` → 0（含 3 分钟模拟漂移 < 100 ms） | 完成 | 2026-11-16 | 2026-11-17 |
| T-07 | 视频条：N 路封面 + 懒加载 + 点击切主 | 交互冒烟：切主 < 1 s 断言通过 | 未开始 | 2026-11-16 | — |

## S3 标注与手册

| 编号 | 目标 | 完成判据（命令 → 退出码） | 状态 | 创建时间 | 完成时间 |
|---|---|---|---|---|---|
| T-08 | 标注命令数组 + 撤销栈 + 本地手册存储 | `cargo test -p tacticdeck-annotate` → 0 | 完成 | 2026-11-16 | 2026-11-17 |

## S4 录制流水线

| 编号 | 目标 | 完成判据（命令 → 退出码） | 状态 | 创建时间 | 完成时间 |
|---|---|---|---|---|---|
| T-09 | LiteReplay 改造四项（IPC / 窗口裁剪 / 输出命名 / tick 元数据） | `cargo run -p tacticdeck-record -- --self-test` → 0 | 未开始 | 2026-11-16 | — |
| T-10 | 编排器 `record-round`：多遍采集——同一 tick 区间重放 N 遍，每遍用 `spec_mode` + `spec_player` 锁定一名选手，输出 N 轨视频 + tick 元数据；N 可配 1–10 | 一个真实回合产出 N 路视频，首帧同 tick（±2）→ 0 | 未开始 | 2026-11-16 | — |
| T-11 | 版本适配回归（tick 96 / seek 顺序 / 暂停 4 tick） | `pwsh -File scripts/regress-record.ps1` → 0 | 未开始 | 2026-11-16 | — |
| T-16 | 录制完成后自动切 2D 回放：录制结束 → 解析 → 进入十人回放并停在首回合首 tick | `pwsh -File scripts/regress-record.ps1 -Mode handoff` → 0（3 s 内进入回放，tick 对齐 ±2） | 未开始 | 2026-11-16 | — |

## S5 打包

| 编号 | 目标 | 完成判据（命令 → 退出码） | 状态 | 创建时间 | 完成时间 |
|---|---|---|---|---|---|
| T-12 | Windows 安装包 + 冷启动 < 3 s | `bun run build` → 0；启动计时脚本断言通过 | 未开始 | 2026-11-16 | — |

---

## 待裁决清单

记录单位与任务相同：提出时间 / 问题 / 可选方法 / 解决时间。**未决条目不得删除**，裁决后补「解决时间」与结论编号（对应 `DECISIONS.md`）。

| 编号 | 提出时间 | 问题 | 可选方法 | 解决时间 |
|---|---|---|---|---|
| Q-01 | 2026-11-16 | 录制来源：本地 HLAE + 自建抓屏，还是服务端录制？ | A 本地（成本 0，质量受机器）；B 服务端（稳定，需 GPU 服务器与运维） | 未决 |
| Q-02 | 2026-11-16 | 素材边界：只处理公开/自有 demo，还是允许任意上传？ | A 仅公开/自有；B 允许上传（需隐私与版权声明） | 未决 |
| Q-03 | 2026-11-16 | 是否把本仓库登记进 `D:\Projects\_docs\命名规范.md` 的现行条目？ | A 登记；B 暂不登记 | 未决 |
| Q-04 | 2026-11-16 | 中文文档是否需要拼写检查（cspell）通道？ | A 接入并维护中文词典；B 暂不接入，理由记入 DECISIONS | 未决 |
| Q-05 | 2026-11-16 | 十路 POV（五对五全录）的采集方式？ | A 分机采集：多台机器各 1–2 路，靠 `TickClock` 对齐；B 单机只录 5 路（每队选人），2D 回放补全十人；C 单实例多相机流（须先验证 HLAE `mirv_streams` 在 CS2 下可用） | 2026-11-16 **裁决 D-013**：单机、多遍、逐遍锁选手（N 可配 1–10），不并行、不依赖多机 |
