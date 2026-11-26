//! tick 元数据 schema：一回合 N 遍录制的统一描述，可序列化为 JSON 供前端消费。
//!
//! 时间口径与 `src/replay/types.ts` 一致：tick 是整数，单一时基
//! `tick = startTick + t × tickrate`，本 crate 不另造第二套时间换算。

use serde::{Deserialize, Serialize};

use crate::interval::{MAX_SLOT, MIN_SLOT, MIN_START_TICK};

/// 当前元数据 schema 版本；结构改动时递增
pub const SCHEMA_VERSION: u32 = 1;

/// 一条视频轨（一遍录制）的元数据
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackMeta {
    /// 锁定的选手 slot（1..=10）
    pub slot: u8,
    /// 视频文件名（与命名规则产物一致）
    pub video_file: String,
    /// 首帧 tick（整数，落在区间内）
    pub first_tick: u32,
    /// 末帧 tick（整数，落在区间内）
    pub last_tick: u32,
    /// 本遍 seek 到的起点 tick
    pub seek_tick: u32,
}

/// 一回合录制的完整元数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordMetadata {
    /// schema 版本
    pub schema_version: u32,
    /// tick 率（与 `replay.json` 的 `tickRate` 同口径）
    pub tick_rate: f64,
    /// 地图名
    pub map: String,
    /// 回合号（0 起，与前端 `RoundInfo.index` 对齐）
    pub round: u32,
    /// 区间起点 tick（含）
    pub start_tick: u32,
    /// 区间终点 tick（含）
    pub end_tick: u32,
    /// pass 总数，必须等于 `tracks.len()`
    pub pass_count: u32,
    /// 每轨一条
    pub tracks: Vec<TrackMeta>,
}

/// 元数据校验失败原因
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetadataError {
    /// 起点早于最低允许 tick
    StartBelowMinimum { start: u32 },
    /// 终点没有大于起点
    IntervalInverted { start: u32, end: u32 },
    /// tick 率不合法（非正或非有限）
    TickRateInvalid { tick_rate: f64 },
    /// 声明的 pass 总数与实际轨数对不上
    PassCountMismatch { declared: u32, actual: usize },
    /// 一条轨都没有
    NoTracks,
    /// slot 不在 1..=10
    SlotOutOfRange { slot: u8 },
    /// slot 重复
    DuplicateSlot { slot: u8 },
    /// 首尾 / seek tick 落在区间外
    TickOutsideInterval { slot: u8, tick: u32 },
    /// 首帧 tick 晚于末帧 tick
    FirstAfterLast { slot: u8, first: u32, last: u32 },
    /// 视频文件名为空
    EmptyVideoFile { slot: u8 },
}

impl std::fmt::Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StartBelowMinimum { start } => {
                write!(f, "起点 tick {start} 早于最低允许 tick {MIN_START_TICK}")
            }
            Self::IntervalInverted { start, end } => {
                write!(f, "终点 tick {end} 没有大于起点 {start}")
            }
            Self::TickRateInvalid { tick_rate } => write!(f, "tick 率 {tick_rate} 不合法"),
            Self::PassCountMismatch { declared, actual } => {
                write!(f, "pass 总数 {declared} 与实际轨数 {actual} 对不上")
            }
            Self::NoTracks => write!(f, "一条轨都没有"),
            Self::SlotOutOfRange { slot } => {
                write!(f, "slot {slot} 不在 {MIN_SLOT}..={MAX_SLOT} 范围内")
            }
            Self::DuplicateSlot { slot } => write!(f, "slot {slot} 重复出现"),
            Self::TickOutsideInterval { slot, tick } => {
                write!(f, "slot {slot} 的 tick {tick} 落在区间外")
            }
            Self::FirstAfterLast { slot, first, last } => {
                write!(f, "slot {slot} 首帧 {first} 晚于末帧 {last}")
            }
            Self::EmptyVideoFile { slot } => write!(f, "slot {slot} 视频文件名为空"),
        }
    }
}

impl std::error::Error for MetadataError {}

impl RecordMetadata {
    /// 校验元数据：slot 唯一且在范围内、区间自洽、首尾与 seek tick 都落在区间内。
    ///
    /// 这里只编排 / 结构校验，不读真实视频，也不核对帧内容。
    pub fn validate(&self) -> Result<(), MetadataError> {
        if self.start_tick < MIN_START_TICK {
            return Err(MetadataError::StartBelowMinimum {
                start: self.start_tick,
            });
        }
        if self.end_tick <= self.start_tick {
            return Err(MetadataError::IntervalInverted {
                start: self.start_tick,
                end: self.end_tick,
            });
        }
        if !self.tick_rate.is_finite() || self.tick_rate <= 0.0 {
            return Err(MetadataError::TickRateInvalid {
                tick_rate: self.tick_rate,
            });
        }
        if self.tracks.is_empty() {
            return Err(MetadataError::NoTracks);
        }
        if self.pass_count as usize != self.tracks.len() {
            return Err(MetadataError::PassCountMismatch {
                declared: self.pass_count,
                actual: self.tracks.len(),
            });
        }

        let mut seen = std::collections::HashSet::new();
        for track in &self.tracks {
            if !(MIN_SLOT..=MAX_SLOT).contains(&track.slot) {
                return Err(MetadataError::SlotOutOfRange { slot: track.slot });
            }
            if !seen.insert(track.slot) {
                return Err(MetadataError::DuplicateSlot { slot: track.slot });
            }
            if track.video_file.is_empty() {
                return Err(MetadataError::EmptyVideoFile { slot: track.slot });
            }
            if track.first_tick > track.last_tick {
                return Err(MetadataError::FirstAfterLast {
                    slot: track.slot,
                    first: track.first_tick,
                    last: track.last_tick,
                });
            }
            for tick in [track.first_tick, track.last_tick, track.seek_tick] {
                if tick < self.start_tick || tick > self.end_tick {
                    return Err(MetadataError::TickOutsideInterval {
                        slot: track.slot,
                        tick,
                    });
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{MetadataError, RecordMetadata, SCHEMA_VERSION, TrackMeta};

    fn track(slot: u8, first: u32, last: u32) -> TrackMeta {
        TrackMeta {
            slot,
            video_file: format!("de_mirage_round04_slot{slot:02}.mp4"),
            first_tick: first,
            last_tick: last,
            seek_tick: 96,
        }
    }

    fn metadata(tracks: Vec<TrackMeta>) -> RecordMetadata {
        let pass_count = tracks.len() as u32;
        RecordMetadata {
            schema_version: SCHEMA_VERSION,
            tick_rate: 64.0,
            map: "de_mirage".to_string(),
            round: 4,
            start_tick: 96,
            end_tick: 320,
            pass_count,
            tracks,
        }
    }

    #[test]
    fn valid_metadata_passes_validate() {
        let meta = metadata(vec![track(1, 96, 320), track(5, 96, 320)]);
        meta.validate().unwrap();
    }

    #[test]
    fn json_roundtrip_keeps_shape() {
        let meta = metadata(vec![track(1, 96, 320), track(10, 97, 319)]);
        let json = serde_json::to_string(&meta).unwrap();
        let back: RecordMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back, meta);
        // 字段名走 camelCase，与前端契约对齐
        assert!(json.contains("tickRate"));
        assert!(json.contains("startTick"));
        assert!(json.contains("endTick"));
    }

    #[test]
    fn rejects_duplicate_slot() {
        let meta = metadata(vec![track(3, 96, 320), track(3, 96, 320)]);
        assert!(matches!(
            meta.validate(),
            Err(MetadataError::DuplicateSlot { slot: 3 })
        ));
    }

    #[test]
    fn rejects_tick_outside_interval() {
        let mut bad = track(2, 80, 320);
        bad.first_tick = 80;
        let meta = metadata(vec![bad]);
        assert!(matches!(
            meta.validate(),
            Err(MetadataError::TickOutsideInterval { slot: 2, tick: 80 })
        ));
    }

    #[test]
    fn rejects_first_after_last_and_count_mismatch() {
        let mut bad = track(2, 200, 100);
        bad.first_tick = 200;
        bad.last_tick = 100;
        let meta = metadata(vec![bad]);
        assert!(matches!(
            meta.validate(),
            Err(MetadataError::FirstAfterLast { slot: 2, .. })
        ));

        let mut meta = metadata(vec![track(1, 96, 320)]);
        meta.pass_count = 2;
        assert!(matches!(
            meta.validate(),
            Err(MetadataError::PassCountMismatch {
                declared: 2,
                actual: 1
            })
        ));
    }

    #[test]
    fn rejects_inverted_interval_and_bad_rate() {
        let mut meta = metadata(vec![track(1, 96, 320)]);
        meta.end_tick = 90;
        assert!(matches!(
            meta.validate(),
            Err(MetadataError::IntervalInverted { .. })
        ));

        let mut meta = metadata(vec![track(1, 96, 320)]);
        meta.tick_rate = 0.0;
        assert!(matches!(
            meta.validate(),
            Err(MetadataError::TickRateInvalid { .. })
        ));
    }
}
