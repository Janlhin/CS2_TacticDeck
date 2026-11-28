//! 整回合编排：把区间、遍数、slot 顺序合成 N 遍任务列表，并校验编排层不变量。
//!
//! 本模块只算编排数据（步骤、路径、元数据条目），不读真实录像、不校验真实视频帧；
//! 真实采集的首尾 tick 由采集器回填（待 Q-01 裁决）。

use crate::interval::{PlanIntervalError, RecordStep, RecordingInterval, pass_specs};
use crate::metadata::{RecordMetadata, TrackMeta};
use crate::naming::{metadata_file_name, video_file_name};

/// 首帧对齐容差（tick）：各遍首帧预期落在同一基准 tick ±2 以内
pub const FIRST_FRAME_TOLERANCE: u32 = 2;

/// 一遍录制任务：命令步骤 + 输出路径 + 待回填的元数据条目
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassTask {
    /// 本遍锁定的选手 slot（1..=10）
    pub slot: u8,
    /// 视频输出路径（目录 + 命名规则产物，逻辑路径）
    pub video_path: String,
    /// 按顺序排列的命令步骤
    pub steps: Vec<RecordStep>,
    /// 待回填的元数据条目；首尾 tick 先填区间预期值，真实采集后由采集器覆盖
    pub metadata_entry: TrackMeta,
}

/// 一回合的完整编排计划
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoundPlan {
    /// 地图名
    pub map: String,
    /// 回合号（0 起，与前端 `RoundInfo.index` 对齐）
    pub round: u32,
    /// 录制区间
    pub interval: RecordingInterval,
    /// pass 总数
    pub pass_count: u8,
    /// 每遍一个任务
    pub tasks: Vec<PassTask>,
}

/// 编排计划构造失败原因
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanBuildError {
    /// 区间 / 遍数 / slot 不合法
    Slots(PlanIntervalError),
    /// 地图名为空
    MapNameEmpty,
}

impl std::fmt::Display for PlanBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Slots(inner) => write!(f, "编排入参不合法：{inner}"),
            Self::MapNameEmpty => write!(f, "地图名为空"),
        }
    }
}

impl std::error::Error for PlanBuildError {}

/// 把目录与文件名拼成逻辑路径；目录为空时只返回文件名。
/// 统一用 `/` 分隔，落盘时由真实后端按目标平台转换（待 Q-01 裁决）。
fn join_path(dir: &str, file: &str) -> String {
    let dir = dir.trim_matches(|c| c == '/' || c == '\\');
    if dir.is_empty() {
        file.to_string()
    } else {
        format!("{dir}/{file}")
    }
}

/// 规划一回合的 N 遍任务列表。
///
/// `output_dir` 是视频文件所在目录（逻辑路径）；元数据文件名由
/// [`RoundPlan::metadata_file_name`] 单独给出。
pub fn plan_round(
    map: &str,
    round: u32,
    interval: RecordingInterval,
    n_passes: u8,
    slot_order: &[u8],
    output_dir: &str,
) -> Result<RoundPlan, PlanBuildError> {
    if map.trim().is_empty() {
        return Err(PlanBuildError::MapNameEmpty);
    }
    let specs = pass_specs(interval, n_passes, slot_order).map_err(PlanBuildError::Slots)?;

    let tasks = specs
        .iter()
        .map(|spec| {
            let video_file = video_file_name(map, round, spec.slot);
            let video_path = join_path(output_dir, &video_file);
            let metadata_entry = TrackMeta {
                slot: spec.slot,
                video_file,
                first_tick: interval.start_tick,
                last_tick: interval.end_tick,
                seek_tick: spec.seek_tick,
            };
            PassTask {
                slot: spec.slot,
                video_path,
                steps: spec.steps.clone(),
                metadata_entry,
            }
        })
        .collect();

    Ok(RoundPlan {
        map: map.to_string(),
        round,
        interval,
        pass_count: n_passes,
        tasks,
    })
}

impl RoundPlan {
    /// 把编排计划转成统一 tick 元数据；首尾 tick 先填区间预期值，
    /// 真实采集完成后由采集器回填实测首尾 tick。
    #[must_use]
    pub fn to_metadata(&self, tick_rate: f64) -> RecordMetadata {
        RecordMetadata {
            schema_version: crate::metadata::SCHEMA_VERSION,
            tick_rate,
            map: self.map.clone(),
            round: self.round,
            start_tick: self.interval.start_tick,
            end_tick: self.interval.end_tick,
            pass_count: self.tasks.len() as u32,
            tracks: self
                .tasks
                .iter()
                .map(|t| t.metadata_entry.clone())
                .collect(),
        }
    }

    /// 回合级元数据文件名（含全部轨）
    #[must_use]
    pub fn metadata_file_name(&self) -> String {
        metadata_file_name(&self.map, self.round)
    }
}

/// 首帧对齐校验失败原因
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignmentError {
    /// 计划里一条任务都没有
    NoTasks,
    /// 某遍首帧预期 tick 偏离基准超过容差
    FirstFrameDrift {
        slot: u8,
        base: u32,
        actual: u32,
        tolerance: u32,
    },
}

impl std::fmt::Display for AlignmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoTasks => write!(f, "计划里一条任务都没有"),
            Self::FirstFrameDrift {
                slot,
                base,
                actual,
                tolerance,
            } => write!(
                f,
                "slot {slot} 首帧 {actual} 偏离基准 {base} 超过 ±{tolerance} tick"
            ),
        }
    }
}

impl std::error::Error for AlignmentError {}

/// 校验编排不变量：所有遍的预期首帧 tick 落在同一基准 tick ±2 以内。
///
/// 只比对编排数据里的 `first_tick`，不读真实视频、不核对帧内容。
pub fn check_first_frame_alignment(plan: &RoundPlan) -> Result<(), AlignmentError> {
    let Some(base) = plan.tasks.first().map(|t| t.metadata_entry.first_tick) else {
        return Err(AlignmentError::NoTasks);
    };
    for task in &plan.tasks {
        let actual = task.metadata_entry.first_tick;
        if actual.abs_diff(base) > FIRST_FRAME_TOLERANCE {
            return Err(AlignmentError::FirstFrameDrift {
                slot: task.slot,
                base,
                actual,
                tolerance: FIRST_FRAME_TOLERANCE,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        FIRST_FRAME_TOLERANCE, PlanBuildError, RecordingInterval, check_first_frame_alignment,
        plan_round,
    };
    use crate::interval::RecordStep;

    fn interval() -> RecordingInterval {
        RecordingInterval::new(96, 320).unwrap()
    }

    #[test]
    fn builds_n_tasks_with_paths_and_entries() {
        let plan = plan_round("de_mirage", 4, interval(), 3, &[1, 5, 10], "recordings/r4").unwrap();
        assert_eq!(plan.tasks.len(), 3);
        assert_eq!(plan.pass_count, 3);
        assert_eq!(
            plan.tasks[0].video_path,
            "recordings/r4/de_mirage_round04_slot01.mp4"
        );
        assert_eq!(plan.tasks[2].metadata_entry.slot, 10);
        assert_eq!(plan.tasks[0].metadata_entry.seek_tick, 96);
        assert_eq!(plan.metadata_file_name(), "de_mirage_round04_meta.json");
    }

    #[test]
    fn tasks_keep_step_order() {
        let plan = plan_round("de_nuke", 1, interval(), 2, &[2, 9], "out").unwrap();
        let steps = &plan.tasks[0].steps;
        assert!(matches!(steps[0], RecordStep::PauseTicks { ticks: 4 }));
        let seek_pos = steps
            .iter()
            .position(|s| matches!(s, RecordStep::SeekTick { .. }))
            .unwrap();
        let camera_pos = steps
            .iter()
            .position(|s| matches!(s, RecordStep::SpecPlayer { .. }))
            .unwrap();
        assert!(seek_pos < camera_pos);
    }

    #[test]
    fn metadata_from_plan_validates() {
        let plan = plan_round("de_mirage", 4, interval(), 3, &[1, 5, 10], "recordings/r4").unwrap();
        let meta = plan.to_metadata(64.0);
        meta.validate().unwrap();
        assert_eq!(meta.pass_count, 3);
        assert_eq!(meta.tracks.len(), 3);
    }

    #[test]
    fn alignment_holds_within_tolerance() {
        let plan = plan_round("de_mirage", 4, interval(), 3, &[1, 5, 10], "out").unwrap();
        check_first_frame_alignment(&plan).unwrap();
    }

    #[test]
    fn alignment_rejects_drifted_first_frame() {
        let mut plan = plan_round("de_mirage", 4, interval(), 2, &[1, 2], "out").unwrap();
        plan.tasks[1].metadata_entry.first_tick = 96 + FIRST_FRAME_TOLERANCE + 1;
        assert!(matches!(
            check_first_frame_alignment(&plan),
            Err(super::AlignmentError::FirstFrameDrift { slot: 2, .. })
        ));
    }

    #[test]
    fn rejects_empty_map_and_bad_slots() {
        assert!(matches!(
            plan_round("  ", 4, interval(), 1, &[1], "out"),
            Err(PlanBuildError::MapNameEmpty)
        ));
        assert!(matches!(
            plan_round("de_mirage", 4, interval(), 2, &[1, 1], "out"),
            Err(PlanBuildError::Slots(_))
        ));
    }
}
