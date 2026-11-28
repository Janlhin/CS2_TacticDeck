//! 采集器接口占位：定义「探测能力 / 执行一遍抓取」的签名，但不实现任何真实采集。
//!
//! 待裁决事实 **Q-01**（见 `TASKS.md`）：录制来源是「本地 HLAE + 自建抓屏」还是
//! 「服务端录制」尚未定论。因此本模块只给出 trait 与一个全部返回
//! [`CollectorError::Undecided`] 的占位实现；HLAE 命令、抓屏帧缓冲、窗口裁剪、
//! 进程 / IPC 调用一律不在本阶段落地，等 Q-01 裁决后再补真实后端。

/// 采集器能力清单（占位结构：真实能力项随 Q-01 裁决后补充）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectorCapabilities {
    /// 是否能在本机完成抓屏；阶段一恒为 `false`（待 Q-01 裁决）
    pub can_capture_locally: bool,
    /// 支持的最大并发遍数；编排模型为单机串行多遍（D-013），阶段一恒为 1
    pub max_concurrent_passes: u8,
}

/// 单遍采集请求：编排层把一遍的区间、锁定 slot、输出路径交给采集器
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassRequest {
    /// 锁定的选手 slot（1..=10）
    pub slot: u8,
    /// seek 到的起点 tick
    pub seek_tick: u32,
    /// 录到的终点 tick（含）
    pub end_tick: u32,
    /// 期望产出的视频文件路径（逻辑路径；本阶段不产生真实落盘行为）
    pub video_path: String,
}

/// 单遍采集结果：产出文件与首尾 tick，供编排层回填元数据
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassOutcome {
    /// 锁定的选手 slot（1..=10）
    pub slot: u8,
    /// 实际产出的视频文件名
    pub video_file: String,
    /// 实际首帧 tick（整数）
    pub first_tick: u32,
    /// 实际末帧 tick（整数）
    pub last_tick: u32,
}

/// 采集器错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectorError {
    /// 采集方案待 Q-01 裁决：本地 HLAE + 自建抓屏，还是服务端录制
    Undecided {
        /// 说明文字，原样写明待裁决项
        detail: String,
    },
    /// 编排层传入的请求不合法（slot 越界、区间反了等）
    InvalidRequest(String),
}

impl CollectorError {
    /// Q-01 未决时统一使用的占位错误，payload 原样写明待裁决项
    pub fn undecided() -> Self {
        Self::Undecided {
            detail: "待 Q-01 裁决：本地 HLAE + 自建抓屏，还是服务端录制".to_string(),
        }
    }
}

impl std::fmt::Display for CollectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undecided { detail } => write!(f, "采集方案未决：{detail}"),
            Self::InvalidRequest(msg) => write!(f, "采集请求不合法：{msg}"),
        }
    }
}

impl std::error::Error for CollectorError {}

/// 采集器 trait：阶段一只定签名，不实现真实抓屏。
///
/// 真实后端（HLAE 抓屏 / 服务端录制）在 Q-01 裁决后实现本 trait；
/// 阶段一唯一的实现 [`UndecidedCollector`] 所有方法都返回
/// [`CollectorError::Undecided`]。
pub trait Collector {
    /// 探测当前环境具备哪些采集能力；阶段一返回未决错误。
    fn capabilities(&self) -> Result<CollectorCapabilities, CollectorError>;

    /// 执行一遍抓取：锁死指定选手，从 `seek_tick` 录到 `end_tick`，
    /// 产出视频文件并回报首尾 tick。
    ///
    /// 阶段一返回未决错误：不触碰任何真实进程、窗口、帧缓冲或 IPC。
    fn record_pass(&self, request: &PassRequest) -> Result<PassOutcome, CollectorError>;
}

/// 占位采集器：所有方法都返回 [`CollectorError::Undecided`]，
/// 用来在不引入真实采集依赖的前提下把编排链路串起来。
#[derive(Debug, Clone, Copy, Default)]
pub struct UndecidedCollector;

impl Collector for UndecidedCollector {
    fn capabilities(&self) -> Result<CollectorCapabilities, CollectorError> {
        Err(CollectorError::undecided())
    }

    fn record_pass(&self, _request: &PassRequest) -> Result<PassOutcome, CollectorError> {
        Err(CollectorError::undecided())
    }
}

#[cfg(test)]
mod tests {
    use super::{Collector, CollectorError, UndecidedCollector};

    #[test]
    fn placeholder_reports_undecided_for_every_method() {
        let collector = UndecidedCollector;
        let cap = collector.capabilities();
        assert!(matches!(cap, Err(CollectorError::Undecided { .. })));

        let request = super::PassRequest {
            slot: 1,
            seek_tick: 96,
            end_tick: 320,
            video_path: "demo.mp4".to_string(),
        };
        let outcome = collector.record_pass(&request);
        assert!(matches!(outcome, Err(CollectorError::Undecided { .. })));

        // 错误文案必须原样带上 Q-01 待裁决说明
        let CollectorError::Undecided { detail } = outcome.unwrap_err() else {
            unreachable!();
        };
        assert!(detail.contains("待 Q-01 裁决"));
    }
}
