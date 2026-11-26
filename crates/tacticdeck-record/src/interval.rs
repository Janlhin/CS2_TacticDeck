//! 录制区间与多遍切分：把一个回合的 tick 闭区间拆成 N 遍串行任务规格。
//!
//! 硬约束（AGENTS.md「录制时序四条」+ D-013）：
//! - 命令不得早于 tick 96；
//! - seek 必须早于切相机；
//! - 每段前暂停 4 tick；
//! - 同一区间重放 N 遍（N ∈ 1..=10），每遍锁一名互不相同的选手 slot（1..=10）；
//! - 不并行、不依赖多机。

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// 录制命令最早允许的 tick（AGENTS.md：命令不得早于 tick 96）
pub const MIN_START_TICK: u32 = 96;
/// 每段录制前固定暂停的 tick 数（AGENTS.md：每段前暂停 4 tick）
pub const PRE_ROLL_PAUSE_TICKS: u32 = 4;
/// 最少遍数（D-013：N 可配 1..=10）
pub const MIN_PASSES: u8 = 1;
/// 最多遍数
pub const MAX_PASSES: u8 = 10;
/// 最小选手 slot
pub const MIN_SLOT: u8 = 1;
/// 最大选手 slot
pub const MAX_SLOT: u8 = 10;

/// 一个含端点的 tick 闭区间 `[start_tick, end_tick]`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordingInterval {
    /// 起点 tick（含），不得早过 96
    pub start_tick: u32,
    /// 终点 tick（含），必须大于起点
    pub end_tick: u32,
}

/// 区间校验失败原因
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntervalError {
    /// 起点早于最低允许 tick
    StartBelowMinimum { start: u32, minimum: u32 },
    /// 终点没有大于起点
    EndNotAfterStart { start: u32, end: u32 },
}

impl RecordingInterval {
    /// 校验并构造区间；非法输入返回错误而不是 panic
    pub fn new(start_tick: u32, end_tick: u32) -> Result<Self, IntervalError> {
        if start_tick < MIN_START_TICK {
            return Err(IntervalError::StartBelowMinimum {
                start: start_tick,
                minimum: MIN_START_TICK,
            });
        }
        if end_tick <= start_tick {
            return Err(IntervalError::EndNotAfterStart {
                start: start_tick,
                end: end_tick,
            });
        }
        Ok(Self {
            start_tick,
            end_tick,
        })
    }

    /// tick 是否落在区间内（含端点）
    #[must_use]
    pub fn contains(&self, tick: u32) -> bool {
        self.start_tick <= tick && tick <= self.end_tick
    }
}

impl std::fmt::Display for IntervalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StartBelowMinimum { start, minimum } => {
                write!(f, "起点 tick {start} 早于最低允许 tick {minimum}")
            }
            Self::EndNotAfterStart { start, end } => {
                write!(f, "终点 tick {end} 没有大于起点 {start}")
            }
        }
    }
}

impl std::error::Error for IntervalError {}

/// 一遍录制要按顺序执行的命令步骤（显式序列，编排层按列表顺序回放）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RecordStep {
    /// 段前暂停若干 tick（固定 4）
    PauseTicks { ticks: u32 },
    /// seek 到区间起点；按时序四条，必须排在切相机之前
    SeekTick { tick: u32 },
    /// 开启观察者模式（`spec_mode`）
    SpecMode,
    /// 把相机锁到指定选手（`spec_player <slot>`）
    SpecPlayer { slot: u8 },
    /// 录到区间终点
    RecordUntil { tick: u32 },
}

/// 一遍录制的规格：锁定哪个 slot、暂停几 tick、按什么步骤执行
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassSpec {
    /// 本遍锁定的选手 slot（1..=10）
    pub slot: u8,
    /// 段前暂停 tick 数（固定 4）
    pub pause_ticks: u32,
    /// seek 目标 tick（= 区间起点）
    pub seek_tick: u32,
    /// 按顺序排列的命令步骤
    pub steps: Vec<RecordStep>,
}

/// 多遍切分失败原因
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanIntervalError {
    /// 区间本身不合法
    InvalidInterval(IntervalError),
    /// 遍数不在 1..=10
    PassCountOutOfRange { n_passes: u8 },
    /// slot 不在 1..=10
    SlotOutOfRange { slot: u8 },
    /// 同一 slot 被安排给了多遍
    DuplicateSlot { slot: u8 },
    /// slot 列表长度与遍数对不上
    SlotCountMismatch { expected: usize, actual: usize },
}

impl std::fmt::Display for PlanIntervalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInterval(inner) => write!(f, "区间不合法：{inner}"),
            Self::PassCountOutOfRange { n_passes } => {
                write!(f, "遍数 {n_passes} 不在 {MIN_PASSES}..={MAX_PASSES} 范围内")
            }
            Self::SlotOutOfRange { slot } => {
                write!(f, "slot {slot} 不在 {MIN_SLOT}..={MAX_SLOT} 范围内")
            }
            Self::DuplicateSlot { slot } => write!(f, "slot {slot} 重复出现"),
            Self::SlotCountMismatch { expected, actual } => {
                write!(f, "slot 列表长度 {actual} 与遍数 {expected} 对不上")
            }
        }
    }
}

impl std::error::Error for PlanIntervalError {}

/// 把一个区间切成 N 遍规格。
///
/// `slot_order` 给出每遍锁定的 slot（长度必须等于 `n_passes`，互不相同且都在 1..=10）。
/// 每遍步骤固定为：暂停 4 tick → seek 到起点 → `spec_mode` → `spec_player <slot>`
/// → 录到终点，保证 seek 早于 camera。
pub fn pass_specs(
    interval: RecordingInterval,
    n_passes: u8,
    slot_order: &[u8],
) -> Result<Vec<PassSpec>, PlanIntervalError> {
    RecordingInterval::new(interval.start_tick, interval.end_tick)
        .map_err(PlanIntervalError::InvalidInterval)?;

    if !(MIN_PASSES..=MAX_PASSES).contains(&n_passes) {
        return Err(PlanIntervalError::PassCountOutOfRange { n_passes });
    }
    let expected = n_passes as usize;
    if slot_order.len() != expected {
        return Err(PlanIntervalError::SlotCountMismatch {
            expected,
            actual: slot_order.len(),
        });
    }

    let mut seen = HashSet::new();
    for &slot in slot_order {
        if !(MIN_SLOT..=MAX_SLOT).contains(&slot) {
            return Err(PlanIntervalError::SlotOutOfRange { slot });
        }
        if !seen.insert(slot) {
            return Err(PlanIntervalError::DuplicateSlot { slot });
        }
    }

    Ok(slot_order
        .iter()
        .map(|&slot| PassSpec {
            slot,
            pause_ticks: PRE_ROLL_PAUSE_TICKS,
            seek_tick: interval.start_tick,
            steps: vec![
                RecordStep::PauseTicks {
                    ticks: PRE_ROLL_PAUSE_TICKS,
                },
                RecordStep::SeekTick {
                    tick: interval.start_tick,
                },
                RecordStep::SpecMode,
                RecordStep::SpecPlayer { slot },
                RecordStep::RecordUntil {
                    tick: interval.end_tick,
                },
            ],
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::{
        IntervalError, MAX_PASSES, MAX_SLOT, MIN_SLOT, MIN_START_TICK, PRE_ROLL_PAUSE_TICKS,
        PlanIntervalError, RecordStep, RecordingInterval, pass_specs,
    };

    fn interval() -> RecordingInterval {
        RecordingInterval::new(MIN_START_TICK, 320).unwrap()
    }

    #[test]
    fn interval_rejects_early_start_and_inverted_range() {
        assert!(matches!(
            RecordingInterval::new(80, 320),
            Err(IntervalError::StartBelowMinimum { .. })
        ));
        assert!(matches!(
            RecordingInterval::new(100, 100),
            Err(IntervalError::EndNotAfterStart { .. })
        ));
        assert!(matches!(
            RecordingInterval::new(100, 90),
            Err(IntervalError::EndNotAfterStart { .. })
        ));
    }

    #[test]
    fn splits_into_n_passes_with_distinct_slots() {
        let specs = pass_specs(interval(), 3, &[1, 5, 10]).unwrap();
        assert_eq!(specs.len(), 3);
        assert_eq!(specs[0].slot, 1);
        assert_eq!(specs[1].slot, 5);
        assert_eq!(specs[2].slot, 10);
        for spec in &specs {
            assert_eq!(spec.pause_ticks, PRE_ROLL_PAUSE_TICKS);
            assert_eq!(spec.seek_tick, MIN_START_TICK);
        }
    }

    #[test]
    fn step_order_keeps_seek_before_camera() {
        let specs = pass_specs(interval(), 2, &[2, 9]).unwrap();
        let steps = &specs[0].steps;
        let seek_pos = steps
            .iter()
            .position(|s| matches!(s, RecordStep::SeekTick { .. }))
            .unwrap();
        let camera_pos = steps
            .iter()
            .position(|s| matches!(s, RecordStep::SpecPlayer { .. }))
            .unwrap();
        assert!(seek_pos < camera_pos, "seek 必须早于 camera");
        assert!(matches!(steps[0], RecordStep::PauseTicks { ticks: 4 }));
        assert!(matches!(
            steps.last(),
            Some(RecordStep::RecordUntil { tick: 320 })
        ));
    }

    #[test]
    fn rejects_bad_pass_counts() {
        assert!(matches!(
            pass_specs(interval(), 0, &[]),
            Err(PlanIntervalError::PassCountOutOfRange { n_passes: 0 })
        ));
        assert!(matches!(
            pass_specs(interval(), 11, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 1]),
            Err(PlanIntervalError::PassCountOutOfRange { n_passes: 11 })
        ));
    }

    #[test]
    fn rejects_out_of_range_and_duplicate_slots() {
        assert!(matches!(
            pass_specs(interval(), 2, &[0, 2]),
            Err(PlanIntervalError::SlotOutOfRange { slot: 0 })
        ));
        assert!(matches!(
            pass_specs(interval(), 2, &[1, MAX_SLOT + 1]),
            Err(PlanIntervalError::SlotOutOfRange { slot: 11 })
        ));
        assert!(matches!(
            pass_specs(interval(), 2, &[3, 3]),
            Err(PlanIntervalError::DuplicateSlot { slot: 3 })
        ));
        assert!(matches!(
            pass_specs(interval(), 3, &[1, 2]),
            Err(PlanIntervalError::SlotCountMismatch { .. })
        ));
    }

    #[test]
    fn accepts_full_ten_slots() {
        let slots: Vec<u8> = (MIN_SLOT..=MAX_SLOT).collect();
        let specs = pass_specs(interval(), MAX_PASSES, &slots).unwrap();
        assert_eq!(specs.len(), 10);
    }
}
