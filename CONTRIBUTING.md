# 贡献指南

## 环境要求

- PowerShell 7.4 或更高版本（`scripts/*.ps1` 首行声明 `#requires -Version 7.4`）。
- Deno 2.9.8（版本以 `.deno-version` 为准）。
- Rust 1.99（仅构建 Tauri 壳时需要；纯前端改动可先不装）。
- 不需要 Node.js：前端工具链（Vite / TypeScript / Tauri CLI / 文档检查）都由 Deno 驱动。

## 初次上手

```powershell
deno install --frozen
deno task hooks
```

## 提交前必跑

```powershell
deno task verify
```

`deno task verify` 是唯一验收入口，串联文档结构、字段、链接、根目录白名单、
脚本基线，以及 `src-tauri/Cargo.toml` 存在时的 `cargo fmt` / `clippy` / `test`。
任一步失败都不得提交，禁止用 `--no-verify` 或注释检查的方式绕过。

## 常用任务

- `deno task dev`：在浏览器里起前端开发服务器。
- `deno task app:dev`：起 Tauri 桌面窗口。
- `deno task app:build`：打包可执行文件。
- `deno task typecheck`：`tsc --noEmit` 类型检查。
- `pwsh -NoLogo -NoProfile -File ./scripts/check-syntax.ps1`：纯语法解析全部 `.ps1`，
  不依赖 PSScriptAnalyzer 模块。

## 脚本基线

- 可执行脚本一律走 PowerShell 7.4+（`scripts/` 下的 `.ps1`），不允许 `.cmd` / `.bat`。
- 新增 `.ps1` 首行必须声明 `#requires -Version 7.x`。
- 脚本内不得调用 `cmd.exe` / `powershell.exe` 等第二解析器。

## 提交规范

- 采用 Conventional Commits：`type(scope): 摘要`。
- 一个 commit 只做一件事，不把重构和行为改动混在一起。
- 未真正接入的依赖不要提前写进 `package.json`，避免技术栈登记失真。

## PR 要求

- 从功能分支向 `main` 提 PR；PR 正文写清四节：变更摘要、本地验证命令与结果、
  CI 结论、风险与回滚。
- 未在本地跑过的检查不要在 PR 里写成"已通过"。
