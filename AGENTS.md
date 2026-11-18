# AGENTS.md

**这是目录，不是手册。** 本文件只放强制命令入口、禁止清单、条款锚点引用与本项目特有约定；深层知识一律在 `docs/` 内，按链接过去。**行数上限 150 行。**

规则全文的唯一载体是 [docs/engineering-standards.md](docs/engineering-standards.md)。本文件不自带规范正文。

## 强制命令入口

| 目的 | 命令 |
|---|---|
| 验收（唯一入口） | `bun run verify` |
| 快速验收（不查外链） | `bun run verify:fast` |
| 文档专项（不跑 Rust） | `bun run docs:check` |
| 前端类型检查 | `bun run typecheck` |
| 前端逻辑测试 | `bun run test:web` |
| 前端开发服务器 | `bun run dev` |
| 桌面应用（开发态） | `bun run app:dev` |
| 桌面应用打包 | `bun run app:build` |
| 后端构建 | `bun run build` |
| 后端测试 | `bun run test` |

任何"已验证/已通过"的说法，必须附**命令原文 + 退出码**。未执行就写「未执行」及原因。

Bun 版本由 `.bun-version` 守卫：若 `bun --version` 的版本号不是 `1.4.4`，`toolchain/bun` 步骤会直接失败并打印解析到的路径。前端依赖一律用 `bun install` 安装、工具一律用 `bun run <script>` 或 `bun node_modules/<包入口>` 启动，**不要**再走 `npm` / `npx` / `node_modules/.bin/*.cmd`。

## 条款锚点

- 校验链与唯一入口 → [校验链](docs/engineering-standards.md#校验链)
- 五个文档的单一职责与字段 → [文档先行](docs/engineering-standards.md#文档先行)
- 原子化与提交粒度 → [原子化](docs/engineering-standards.md#原子化)
- 歧义必须询问 → [歧义协议](docs/engineering-standards.md#歧义协议)
- 先搜寻后自研、三条参考路径 → [检索纪律](docs/engineering-standards.md#检索纪律)
- 冗余清理三层触发 → [清理纪律](docs/engineering-standards.md#清理纪律)
- 根目录白名单与子目录判据 → [目录与文件放置](docs/engineering-standards.md#目录与文件放置)
- 脚本统一 pwsh、禁 cmd/bat → [脚本基线](docs/engineering-standards.md#脚本基线)
- 版本号与提交类型映射 → [版本号](docs/engineering-standards.md#版本号)
- 交付说明四项 → [交付说明](docs/engineering-standards.md#交付说明)
- 缺陷类型与拦截层对照 → [校验矩阵](docs/engineering-standards.md#校验矩阵)
- 删除授权边界 → [删除授权](docs/engineering-standards.md#删除授权)

## 禁止清单

- 未运行 `bun run verify` 就声称完成；用 `--no-verify`、注释测试、放宽容错规则来"通过"。
- 一个提交做多件事；提交信息写 `update` / `fix bug` / `小改动`。
- 新增 `.cmd` / `.bat`，或在脚本、钩子、CI 里调用 `cmd.exe`、`powershell.exe`。
- 在根目录放白名单之外的文件（尤其临时笔记、分析报告、`NOTES.md`）。
- 复制无许可证项目的代码；把未登记来源的第三方产物放进仓库。
- 提交 `.dem`、视频、`.glb` 等版权素材或任何凭据。
- 在受 VAC 保护的服务器上调用 HLAE。
- 没有决策记录就改动架构方向（基底、解析器、同步策略）。

## 本项目特有约定

- **单一时间基**：`tick = startTick + t_video × tickrate`。任何模块不得自建第二套时间换算；对齐只发生在同步层。
- **素材边界**：demo 与录制品不进仓库、不上传；地图资产运行时从本机读取。
- **录制时序四条**：命令不得早于 tick 96；seek 必须早于 camera；每段前暂停 4 tick；多路不混分辨率。改动录制编排必须同步更新 `DECISIONS.md`。
- **角色文件**：`README.md` 面向外部读者；`AGENTS.md` 面向执行者；`docs/` 放全文与判定；不要在这三者之间复制同一段话。

## 待裁决项

见 [TASKS.md](TASKS.md) 的「待裁决清单」。执行中撞到新歧义，按 [歧义协议](docs/engineering-standards.md#歧义协议)登记，不猜。
