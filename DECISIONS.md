# DECISIONS.md

追加式决策日志：只追加，不删除。每个条目记录**被采纳的方案**与**被否决的替代方案及理由**——历史属于版本控制，也属于这里，但绝不以"已废弃段落"的形式留在主文档里。

字段：编号 / 日期 / 决策 / 背景 / 被采纳方案 / 被否决的替代方案与理由 / 影响范围 / 复审条件。

---

## D-001 基底：独立 Tauri v2 应用

- **日期**：2026-11-16
- **决策**：以新建的 Tauri v2 桌面壳为项目基底，既有代码以"零件"形式接入，不做宿主级 fork。
- **背景**：四个候选分别实测——`cs-demo-manager` 分叉（2634 个 ts/tsx、101789 行、Electron、上游已分叉）、`cs2-replay`（Three.js 五视口，仅 7 列 `track.json`）、`LiteReplay_Source`（C++/Win32 抓屏，仅覆盖录制）、新建 Tauri 壳。
- **被采纳方案**：Tauri v2 + Rust 后端 + TS/SCSS 前端；解析、录制、渲染各自独立接入。
- **被否决的替代方案与理由**：
  - *以 CSDM 为宿主加页*：能白拿 demo 库与雷达，但 10 万行存量、Electron 运行时、上游持续更新导致每次同步都是 rebase；且其"按需查库"的时间模型与本项目"整段回放数据集 + N 段视频"不兼容，会引入第二套时间基。
  - *以 `cs2-replay` 为基底*：渲染已验证，但无解析、无库、无 2D、无视频，等于只拿到四分之一还要背 GLB 资产体积。
  - *以 LiteReplay 为基底*：语言栈（C++/Win32/GDI）与既定"后端迁 Rust"方向相反，且只覆盖录制。
- **影响范围**：仓库结构（`crates/`、`apps/`）、技术栈登记、CI 形态。
- **复审条件**：若 S0–S1 阶段发现需要 demo 库能力且自建成本超过两周，重新评估与 CSDM 的深度集成（而非 fork）。

## D-002 解析：`source2-demo` 优先，`demoparser` 作 sidecar 兜底

- **日期**：2026-11-16
- **决策**：S0 以 crates.io 的 `source2-demo`（MIT OR Apache-2.0，`features=["cs2"]`）为主；字段不够时挂 `@laihoe/demoparser2` 作为 sidecar，两条路径统一收敛到同一份 `replay.json`。
- **背景**：`LaihoE/demoparser` 的 Rust 核心**未发布到 crates.io**（仓库只产出 Python / Node / WASM 绑定），"`cargo add demoparser`"不成立；而 `source2-demo` 可直接嵌入。
- **被采纳方案**：crate 直依赖 + 可选 sidecar。
- **被否决的替代方案与理由**：
  - *只走 demoparser（Python/Node）*：多一个进程与运行时依赖，桌面分发成本上升；但字段最全，故保留为兜底。
  - *自写解析器*：Source 2 demo 格式的字段与压缩细节极多，自研等于把最贵的风险放在最前面，违背"先搜寻后自研"。
- **影响范围**：`crates/` 的解析模块、`replay.json` schema、`package.json` 可选依赖。
- **复审条件**：S0 实测 `source2-demo` 若缺少投射物逐 tick 位置或道具事件，则切换主路径为 sidecar，并更新本条。

## D-003 多路呈现：1 主 + N 缩略图切换

- **日期**：2026-11-16
- **决策**：MVP 只渲染主画面，其余 N−1 路以封面图呈现，点击切换。
- **背景**：5 路同时解码是 5 倍解码压力；参考产品同屏缩略图栏本质也是封面。
- **被采纳方案**：单路解码 + 懒加载切换（切主 < 1 s）。
- **被否决的替代方案与理由**：
  - *五路真同播*：观感更强，但 1080p 五路并发解码在中低端显卡上不可控，会把 MVP 的验收指标（漂移 < 100 ms）变成性能赌博。
  - *抽帧到 canvas 拼图*：需要额外一条渲染通路，且首版没有收益。
- **影响范围**：同步层（同一时间基下只驱动一个活跃播放器）、存储策略（其余路可降分辨率）。
- **复审条件**：若 720p 五路并发实测稳定且 CPU/GPU 占用可接受，在 M2 重新评估真同播。

## D-004 文档体系收敛为五文档 + 根目录白名单

- **日期**：2026-11-16
- **决策**：建立 `README.md` / `AGENTS.md` / `DECISIONS.md` / `TASKS.md` / `MANIFEST.md` 五个职责单一的文件；根目录只保留白名单内条目，说明性文档移入 `docs/`。
- **背景**：原仓库根目录存在 `THIRD_PARTY_NOTICES.md` 这类散文文件，且没有决策与任务台账，导致"为什么这么做"只存在于对话记录里。
- **被采纳方案**：五文档 + `docs/`；`THIRD_PARTY_NOTICES.md` 迁移为 `docs/third-party-notices.md`。
- **被否决的替代方案与理由**：
  - *合并成一个大文档*：按文件粒度的校验（字段完整性 vs 命令可执行）写不到同一个文件上。
  - *保留根目录 notices 文件*：破坏根目录白名单，且两个入口（README、notices）会各自演化。
- **影响范围**：全部文档链接、`verify` 的字段检查、根目录比对脚本。
- **复审条件**：五文档出现职责重叠或长期空置时，合并并更新本条。

## D-005 版本号：从 `0.1.0` 起步，公共 API 视为不稳定

- **日期**：2026-11-16
- **决策**：`0.y.z` 阶段；`1.0.0` 之前不承诺兼容。
- **背景**：尚无公共 API（当前只有文档与门禁）。
- **被采纳方案**：SemVer 2.0.0；`package.json` 与 tag 同源；tag 推送后不改。
- **被否决的替代方案与理由**：
  - *直接 `1.0.0`*：无公共 API 时给出兼容承诺没有意义，且会逼出"为了不改 API 而不改设计"的反向激励。
- **影响范围**：`package.json`、后续 `Cargo.toml`、发布流程。
- **复审条件**：第一个对外可用的 CLI/API 定型时，评估升 `1.0.0`。

## D-006 脚本统一 PowerShell，禁止 cmd / bat

- **日期**：2026-11-16
- **决策**：所有可执行逻辑写在 `.ps1`；`package.json` scripts、Git hooks、CI 只做 `pwsh -NoLogo -NoProfile -File …` 转发。
- **背景**：Windows 上脚本会在 cmd.exe、sh、pwsh 三种解释器之间切换，同一命令在本地与 CI 语义不同。
- **被采纳方案**：pwsh 7.4+ 唯一入口；钩子用 `sh` 薄壳 `exec pwsh`（LF 行尾）。
- **被否决的替代方案与理由**：
  - *混用 `.cmd` 以便双击运行*：引入第二套解析器，且 `.cmd` 无法承载退出码语义与结构化输出。
- **影响范围**：`scripts/`、`.husky/`、`.github/workflows/`、`.npmrc`。
- **复审条件**：若需在无 pwsh 的环境（如部分 CI 容器）运行，只能新增引导壳探测后转调 `.ps1`，不得复制业务逻辑。

## D-007 版权素材不入库

- **日期**：2026-11-16
- **决策**：`.dem`、视频、`.glb`、地图资产一律不入库；`.gitignore` 先行拦截。
- **背景**：demo 与录制品版权归原权利人；地图资产归 Valve。
- **被采纳方案**：忽略规则 + 运行时从本机提取 + 文档声明。
- **被否决的替代方案与理由**：
  - *先提交再清理历史*：Git 历史不可轻改，清理成本远高于事前拦截。
- **影响范围**：`.gitignore`、`docs/third-party-notices.md`、未来的资源加载模块。
- **复审条件**：若确需样例数据用于测试，只能使用自制合成数据或明确授权的公开样本，并登记到 `MANIFEST.md`。

## D-008 暂不启用 AI 代码审查通道

- **日期**：2026-11-16
- **决策**：当前只依赖确定性通道（文档校验 + 后续的 Rust/TS 静态检查与测试）；AI 审查不接入，也不作为合并阻断。
- **背景**：启用条件要求误报率 < 10%、采纳率 > 30%、且有 ≥ 1 次真实拦截证明；当前仓库无 PR 历史，无法测定。
- **被采纳方案**：不启用，并在 `AGENTS.md` 中不声明该通道。
- **被否决的替代方案与理由**：
  - *先接入再测指标*：会得到"看起来更严格"的假象，实际是给不确定性加了一层可被绕过的话术。
- **影响范围**：CI 配置、评审流程。
- **复审条件**：出现 20 个以上 PR 后，用历史 PR 回放测定误报率与采纳率，达标再启用。

## D-009 工具启动层豁免：npm 的 `.cmd` 垫片

- **状态**：已被 D-015 取代（工具一律由 Deno 直跑包入口文件，不再经过 `.cmd` 垫片）。
- **日期**：2026-11-16
- **决策**：允许 `scripts/verify.ps1` 调用 `node_modules/.bin/*.cmd` 垫片；项目自写代码中仍不出现 `.cmd` / `.bat` 文件，也不调用 `cmd.exe`。
- **背景**：npm 在 Windows 上为每个依赖生成 `.cmd` 启动器，属于工具链自身产物，不在"项目自写脚本"范围内；若不使用它，就必须自行解析每个包的 bin 入口，反而引入新的脆弱点。
- **被采纳方案**：登记豁免，范围仅限 `node_modules/.bin/`。
- **被否决的替代方案与理由**：
  - *自写 node 直接调用 `package.json#bin`*：多出一层解析逻辑，包升级换入口时静默失效。
  - *改用 `npm exec`*：同样经过启动器，等于换个名字。
- **影响范围**：`scripts/verify.ps1`、`scripts/hooks/commit-msg.ps1`。
- **复审条件**：若出现项目自写的 `.cmd` 文件，或垫片调用导致退出码失真，立即撤销豁免。

## D-010 技术栈版本基线：取最新稳定版，预发布不纳入

- **日期**：2026-11-16
- **决策**：桌面壳用 Tauri `2.12.5`（stable），不采用 `3.0.0-alpha.6`；前端用 React `19.3.1`、TypeScript `7.0.2`、Vite `8.3.8`、Zustand `5.1.0`、PixiJS `8.23.0`、Sass `1.108.2`。Three.js 与 Konva 暂不进入依赖，等 M2 真正接入 3D 视图与标注层时再加。
- **背景**：技术栈方向是"升级到最新"。但"最新"必须区分稳定版与预发布：Tauri 3 只有 alpha，装进基线会让整个壳层跟着漂。Three / Konva 若现在装而不用，读代码的人会把它们当既有能力，违反"登记即真实"。
- **被采纳方案**：稳定版最新 + 实测通过（`tsc --noEmit` 与 `vite build` 均退出码 0），预发布版本只在提案中讨论。
- **被否决的替代方案与理由**：
  - *Tauri 3 alpha*：alpha 期间破坏性变更频繁，壳层升级成本不可控。
  - *预先装齐 Three / Konva*：装而不用会让技术栈登记失真。
  - *保守用 Tauri 1 / Vite 5*：与"升级到最新"的方向直接冲突，且 Tauri 2 已是当前主版本。
- **影响范围**：`package.json`、`src-tauri/Cargo.toml`、`README.md` 的技术栈表。
- **复审条件**：Tauri 3 发布 stable，或任一依赖的 major 升级导致实测失败。

## D-011 Node 运行时定为 26.13.0，并把版本一致性做成门禁

- **状态**：已被 D-015 取代（Node 不再出现在工具链中，门禁改为 Deno + `.deno-version`）。
- **日期**：2026-11-16
- **决策**：项目 Node 基线为 `26.13.0`（本机 `C:\Program Files\nodejs\`），npm 为随附的 `11.20.0`；`.npmrc` 设 `engine-strict=true`；`verify` 新增 `toolchain/node` 步骤，比对 `.nvmrc` 与实际 `node -v`，不符即失败并打印解析到的 `node.exe` 路径。
- **背景**：本机同时存在多个 Node 运行时（工具自带运行时会在 PATH 中排在系统 Node 之前），早期开发因此一直跑在 22.23.2 上，与"技术栈取最新"的方向不符，且这类偏差不会有任何报错——只会安静地用旧版本跑通。
- **被采纳方案**：把"用哪个 Node"从口头约定变成可执行门禁（`.nvmrc` + `engines` + `engine-strict` + `verify` 步骤）。
- **被否决的替代方案与理由**：
  - *只在文档里写版本要求*：已经证明会失效，跑的人不会去核对。
  - *改系统 PATH 顺序*：影响本机其他工具，副作用超出本仓库边界，且工具运行时可能再次前置自己。
  - *降级基线到 22 求稳*：与既定方向冲突；26 下 `tsc` / `vite build` / `verify` 均实测通过。
- **影响范围**：`.nvmrc`、`.npmrc`、`package.json` 的 `engines`、`scripts/verify.ps1`、CI 的 `node-version-file`。
- **复审条件**：Node 28 发布，或任一依赖在 26 下实测失败。

## D-012 许可证由 MIT 改为 AGPL-3.0-or-later

- **日期**：2026-11-16
- **决策**：仓库许可改为 `AGPL-3.0-or-later`，根目录新增 `NOTICE` 记录历史授权；`package.json` 的 `license` 字段同步。
- **背景**：目标是**阻止闭源商业化**——避免出现"拿本项目源码套壳成闭源产品"的情形，尤其是对标产品 cs2.cam 那种闭源商业形态。MIT 允许这种用法，且不要求回馈。
- **被采纳方案**：AGPL-3.0-or-later。copyleft 强，且第 13 条覆盖"网络提供服务"的场景；"or later" 保留升级路径。
- **被否决的替代方案与理由**：
  - *维持 MIT*：正是要阻止的情形被允许。
  - *Apache-2.0*：只加专利条款，不产生 copyleft 效果，闭源套壳照样可行。
  - *BSL / SSPL*：非 OSI 认可，会同时挡住正常开源使用者，且条款模糊、争议大。
  - *AGPL + 商业授权双许可*：可作为后期选项，但当前没有商业化收入，先不铺。
- **影响范围**：`LICENSE`、`NOTICE`、`package.json`、`README.md`、`docs/third-party-notices.md`、界面底栏（AGPL 第 5(d) 条要求交互界面展示法律声明）。
- **复审条件**：① 若决定做服务端录制或 Web 形态（Q-01 选 B），须复核第 13 条义务；② 若社区贡献因 AGPL 明显受阻，重新评估是否增加商业/宽松双许可；③ 若 LLM 生成的代码中混入不可再许可的来源，立即核查。

## D-013 录制模型：单机、多遍、逐遍锁定选手相机

- **日期**：2026-11-16
- **决策**：POV 采集采用**多遍录制**——同一 tick 区间重放 N 遍，每遍用 `spec_mode` + `spec_player <slot>` 把相机锁死在一名选手上再抓屏，输出 N 条视频轨与统一 tick 元数据；N 可配 1–10。不做十路并行，不依赖多机，也不赌单实例多相机流。
- **背景**：需求是"单机录制，但十个人各自的视角都要能看"。十路并行需要 10 套独立渲染管线（一份 CS2 实例吃满一块显卡），单机不可行；单实例多相机流在 CS2 + HLAE 下未经验证，收益不确定。
- **被采纳方案**：单机多遍采集。已核实依据：`cs2-highlights-maker` 的 `actions_file.py` 以 `spec_player <slot>` 逐 tick 驱动相机；`cs-demo-manager` 的 `playback_player_lock` 会连续补发 `spec_player`，直到相机真正落到目标选手（游戏会忽略偶发的切人指令）。
- **被否决的替代方案与理由**：① *十路并行*——硬件成本 10 倍且不可控；② *分机采集*——多机运维与时钟对齐复杂度不适合本阶段；③ *单实例多相机流*——未验证，且即便可行也受渲染吞吐限制。
- **代价**：录制耗时 ≈ N × 回合时长，十人一回合约 5–10 分钟，属离线批处理可接受范围；每遍必须能复现同一 tick，依赖 HLAE 回放确定性（由 T-11 回归守住）。
- **影响范围**：`T-09`（自检项含相机锁定）、`T-10`（编排器改为多遍循环）、`T-16`（录制完成后交接 2D 回放）。
- **复审条件**：① 实测单遍录制单回合 > 2 分钟，则重新评估并行或多机；② 若 HLAE 在 CS2 下的 `spec_player` 行为不稳定（出现切人失败），改为锁定时重试 + 校验帧内视角；③ 若后续验证 `mirv_streams` 单实例多路可行，可缩短耗时而无需改架构。

## D-014 视图组织：左栏 11 槽 + 主区单路放大

- **日期**：2026-11-16
- **决策**：界面由"五宫格同屏"改为**左栏 11 个槽位**——顶部 1 个固定 2D 总览（sticky，不随滚动移出），下方 10 名选手 POV 槽纵向排列、滚轮可滚动；点击任一槽把该路放到主区放大显示。主区一次只放一路（2D 或某个 POV），不同屏并排多路。
- **背景**：录制模型定为十人全录（D-013），若仍用五宫格，十路无处安放；且十路真同播的解码压力已在 D-003 评估过不可控。
- **被采纳方案**：11 槽竖排 + 单主视图。滚动即"轮换"，`Shift+滚轮` 直接切主视角（`rotate`）。
- **被否决的替代方案与理由**：① *十宫格同屏*：1080p 十路并发解码不可行；② *五宫格 + 另外五路折叠*：层级混乱，找人不直观；③ *纯列表侧栏 + 无主视图*：失去放大观察的用途。
- **影响范围**：`src/App.tsx`、`src/store/appStore.ts`（槽位模型改为 `overview` + 10 `player`）、`src/styles.scss`、`src-tauri/src/lib.rs`（`POV_SLOTS` 由 5 改 10，测试更名）。
- **复审条件**：① 若实测 720p 六路并发稳定，增加"多格同播"开关按需开启；② 若单队十人以上（教练/替补），槽位需支持滚动越界与分页；③ 若用户反馈滚动找人不便，改为按队伍分组的折叠面板。

## D-015 前端工具链由 Node/npm 整体切到 Deno

- **日期**：2026-11-17
- **决策**：前端依赖安装、脚本入口与全部 npm 工具（Vite 8 / TypeScript 7 / Tauri CLI / commitlint / markdownlint-cli2 / remark / markdown-link-check）改由 **Deno 2.9.8** 驱动；删除 `.nvmrc`、`package-lock.json` 与 `package.json` 的 `engines` / `scripts`，改由 `deno.json` 的 `task` 承载，锁文件换 `deno.lock`，版本门禁换 `.deno-version` + `verify` 的 `toolchain/deno` 步骤。项目**不再需要 Node.js**。
- **背景**：本机 `C:\Program Files\nodejs\node.exe` 被替换为 `v28.0.0-nightly`（npm 12.2.2 在该 Node 上 `npm ci` 直接崩：`Class extends value undefined is not a constructor or null`），导致验收链整体不可执行。修 Node 只是把同一个坑往后挪，而"工具链取最新"的方向要求一个自带包管理、自带锁文件、且不受系统 Node 摆布的运行时。
- **被采纳方案**：Deno 直跑 `node_modules/<包入口文件>`。实测通过的入口：`vite/bin/vite.js`（8.3.8）、`typescript/bin/tsc`（7.0.2）、`@tauri-apps/cli/tauri.js`（2.12.4）、`remark-cli/cli.js`（含 `remark-validate-links` 插件）、`markdownlint-cli2/markdownlint-cli2.mjs`（读 `config/markdownlint-cli2.jsonc`）、`markdown-link-check`（25 条外链）。`deno install` 产出 `deno.lock`（153 KB）与 338 个 `node_modules` 目录。
- **被否决的替代方案与理由**：① *装回 Node 26.13.0*——环境被别人改一次就复发，且用户明确要求换 Deno；② *用 `deno run npm:<包>` 跑 CLI*——实测 `markdownlint-cli2` 会忽略 `--config`（报 MD013 行长错误），必须直跑本地入口文件才生效；③ *只把 Node 降级到 22*——与"取最新"冲突且收益为零；④ *保留 npm 双轨*——两套锁文件必然漂移。
- **影响范围**：`deno.json`、`deno.lock`、`.deno-version`、`.npmrc`（仅留 registry）、`package.json`（去 `engines`/`scripts`）、`scripts/verify.ps1`（工具调用与 `toolchain/deno` 步骤）、`scripts/hooks/commit-msg.ps1`、`scripts/build.ps1`、`.github/workflows/verify.yml`、`README.md`、`AGENTS.md`、`docs/engineering-standards.md`、`TASKS.md`。
- **复审条件**：① Deno 2.9.x 之外的大版本升级需单独提案；② 若任一 npm 工具在 Deno 下出现退出码失真或忽略配置，改回该工具的独立入口脚本并在本条记录；③ 若 Tauri CLI 的打包（`deno task app:build`）在 Deno 下失败，回退为"仅前端用 Deno、打包用独立 Node 便携版"，并重开本节。
