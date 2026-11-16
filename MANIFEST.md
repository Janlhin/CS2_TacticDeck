# MANIFEST.md

第三方二进制、源码与逆向对象的来源登记。字段：名称 / 来源 URL / 版本 / SHA256 / 用途 / 许可结论。

**只登记仓库实际持有或计划引入的对象。** 尚未获取的，SHA256 写「未获取」并说明引入条件，不留空。

## 已持有

| 名称 | 来源 URL | 版本 | SHA256 | 用途 | 许可结论 |
|---|---|---|---|---|---|
| `LiteReplay-2.0-Editor4.zip`（源码发布包） | 作者私有分发；公开仓库尚未建立。发布公告：<https://www.xiaoheihe.cn/app/bbs/link/196995653> | 2.0 Editor Preview 4 | `74C747962F74FB81E586A192663E8685A4BF81E58B12FA47A0CF8603079C9400` | 抓屏与编码层的源码参考；同源核对 | MIT（可改、可分发，保留版权与许可声明） |
| `LiteReplay.exe`（发行包内二进制） | 同上（微信发行包，2026-11-15） | 2.0 Editor Preview 4 | `9592D7CBEAAFA668CA78F617F6DE1E0F964CA841687840321AD005FE41C474E7` | 与源码包做同源校验（哈希一致） | MIT |
| `LiteReplay_Source`（本地源码快照） | 由上述源码包解压而来，未发布 | 2.0 Editor Preview 4 | 210 文件 / 48314308 B；209/209 与包内 `SHA256SUMS` 一致 | 引擎行为的逐文件取证 | MIT |
| `source2-demo`（crate） | <https://docs.rs/source2-demo> | 0.5.11 | 未获取（由 `Cargo.lock` 锁定版本，未做本地哈希） | CS2 demo 解析（S0 主路径） | MIT OR Apache-2.0（二者择一） |

## 计划引入（尚未获取）

| 名称 | 来源 URL | 版本 | SHA256 | 用途 | 许可结论 |
|---|---|---|---|---|---|
| `@laihoe/demoparser2`（npm） | <https://github.com/LaihoE/demoparser> | 待定 | 未获取（引入时登记） | 解析兜底（sidecar） | MIT |
| HLAE（`advancedfx`） | <https://github.com/advancedfx/advancedfx> | 待定 | 未获取（作为外部工具调用，不内嵌） | demo 相机与逐 tick 命令 | MIT；**子模块另有许可，须逐项核对**；含 VAC 风险警告，仅限离线 demo |
| `cs2-map-icons`（overview 数据） | <https://github.com/MurkyYT/cs2-map-icons> | 待定 | 未获取 | 地图盘面校准（开发期） | 许可待核；雷达图为 Valve 资产，发布版改为本机提取 |

## 不入库的对象

| 名称 | 来源 | 原因 |
|---|---|---|
| `.dem` demo 文件 | HLTV / 自有 | 版权归原权利人；只本地使用，`.gitignore` 排除 |
| POV 录制品（`.mp4` 等） | 本项目产出 | 含第三方比赛内容；只本地保存 |
| 地图几何 / `.glb` | Valve 或自行烘焙 | 资产版权与体积；运行时从本机提取或本地生成 |
| 任何凭据、令牌 | — | 安全边界；`.env` 禁止入库 |

## 维护约定

新增第三方二进制或源码前，先在本表登记并给出许可结论；**许可结论为空即不得引入**。许可结论发生变化的，追加新行，不修改旧行。
