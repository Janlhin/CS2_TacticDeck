//! 录制编排骨架：区间切分、多遍任务列表、tick 元数据 schema、输出命名规则与采集器 trait 占位。
//!
//! 本 crate **只做编排与元数据**：把一个回合的 tick 区间拆成 N 遍串行任务、算好输出路径、
//! 生成可序列化的元数据；**不实现真实采集**。
//!
//! 待裁决事实 **Q-01**（`TASKS.md`）：本地 HLAE + 自建抓屏，还是服务端录制，尚未定论。
//! 因此抓屏、窗口裁剪、IPC / 进程调用一律不在本阶段落地，只在 [`collector::Collector`]
//! 留 trait 占位；真实后端待 Q-01 裁决后接入。
//!
//! 编排约束：D-013 单机多遍（N ∈ 1..=10，逐遍锁一名选手，不并行、不依赖多机）；
//! AGENTS.md「录制时序四条」：命令不得早于 tick 96、seek 早于 camera、每段前暂停 4 tick、
//! 多路不混分辨率。

mod collector;
mod interval;
mod metadata;
mod naming;
mod tasks;

pub use collector::{
    Collector, CollectorCapabilities, CollectorError, PassOutcome, PassRequest, UndecidedCollector,
};
pub use interval::{
    IntervalError, PRE_ROLL_PAUSE_TICKS, PassSpec, PlanIntervalError, RecordStep,
    RecordingInterval, pass_specs,
};
pub use metadata::{MetadataError, RecordMetadata, SCHEMA_VERSION, TrackMeta};
pub use naming::{metadata_file_name, video_file_name};
pub use tasks::{
    AlignmentError, FIRST_FRAME_TOLERANCE, PassTask, PlanBuildError, RoundPlan,
    check_first_frame_alignment, plan_round,
};
