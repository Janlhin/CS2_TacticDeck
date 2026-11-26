//! 交接状态机（T-16）：录制完成 → 解析 → 进入十人回放并停在首回合首 tick。
//!
//! 纯逻辑：耗时由虚拟时钟累计，不读墙钟（不使用 `Instant::now`）；解析器通过 trait 注入，
//! 测试 / 自检里挂假解析结果，不调真实进程、不读真实录像。
//!
//! 判定口径：
//! - 总预算 3000 ms：进入回放时累计虚拟耗时必须 ≤ 3000；
//! - tick 对齐：清单起点 tick 与解析摘要首回合起点 tick 之差 ≤ 2（含端点）；
//! - 选手数必须等于 10。
//!
//! 待裁决：**Q-06**（POV 单人录像契约）未决——单人录像应当降级还是失败，不在本轮范围；
//! 本状态机保持「必须 10 人，否则 `Failed`」，降级策略留待 Q-06 裁决后调整。

use crate::metadata::RecordMetadata;

/// 录制完成到进入回放的总预算（虚拟毫秒）
pub const HANDOFF_BUDGET_MS: u32 = 3000;
/// tick 对齐容差（含端点）
pub const TICK_ALIGNMENT_TOLERANCE: u32 = 2;
/// 十人回放要求的选手数
pub const EXPECTED_PLAYER_COUNT: u8 = 10;

/// 回放摘要：解析器从录像清单里读回的关键事实
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplaySummary {
    /// 首回合起点 tick（整数）
    pub first_round_start_tick: u32,
    /// 选手数（必须等于 10）
    pub player_count: u8,
    /// 回合数
    pub round_count: u8,
}

/// 解析失败原因
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// 解析后端尚未接入（占位实现返回此错误）
    NotImplemented,
    /// 其他解析错误，带说明
    Other(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented => write!(f, "解析后端尚未接入（待后续阶段）"),
            Self::Other(reason) => write!(f, "解析失败：{reason}"),
        }
    }
}

impl std::error::Error for ParseError {}

/// 可注入的回放解析器。阶段二只定 trait；真实解析后端待接入。
pub trait ReplayParser {
    /// 解析录像清单，返回回放摘要。实现不得依赖墙钟时间参与判定。
    fn parse(&mut self, bundle: &RecordMetadata) -> Result<ReplaySummary, ParseError>;
}

/// 虚拟时钟：只累计毫秒步进，不碰墙钟。
pub trait VirtualClock {
    /// 当前累计虚拟毫秒
    fn elapsed_ms(&self) -> u32;
    /// 前进 `ms` 个虚拟毫秒
    fn advance(&mut self, ms: u32);
}

/// 测试 / 自检用假时钟：从 0 开始按步累计。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FakeClock {
    elapsed_ms: u32,
}

impl VirtualClock for FakeClock {
    fn elapsed_ms(&self) -> u32 {
        self.elapsed_ms
    }

    fn advance(&mut self, ms: u32) {
        self.elapsed_ms = self.elapsed_ms.saturating_add(ms);
    }
}

/// 进入回放的落点证据：停在首回合首 tick。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayEntry {
    /// 停留 tick（必须等于摘要的首回合起点 tick）
    pub tick: u32,
    /// 回合下标（首回合固定为 0）
    pub round_index: u32,
}

/// 交接失败原因
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandoffFailure {
    /// 解析失败
    ParseFailed { reason: String },
    /// 总预算超 3000 ms
    BudgetExceeded { elapsed_ms: u32, budget_ms: u32 },
    /// tick 漂移超 ±2
    TickDrift {
        expected: u32,
        actual: u32,
        tolerance: u32,
    },
    /// 选手数不等于 10
    PlayerCountMismatch { actual: u8, expected: u8 },
}

impl std::fmt::Display for HandoffFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseFailed { reason } => write!(f, "解析失败：{reason}"),
            Self::BudgetExceeded {
                elapsed_ms,
                budget_ms,
            } => write!(f, "交接预算超支：{elapsed_ms} ms > 预算 {budget_ms} ms"),
            Self::TickDrift {
                expected,
                actual,
                tolerance,
            } => write!(
                f,
                "tick 漂移：清单 {expected} 与摘要 {actual} 相差超过 ±{tolerance}"
            ),
            Self::PlayerCountMismatch { actual, expected } => write!(
                f,
                "选手数 {actual} 不等于十人回放要求的 {expected}（Q-06 未决：单人降级不在本轮范围）"
            ),
        }
    }
}

impl std::error::Error for HandoffFailure {}

/// 交接状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandoffState {
    /// 录制完成，尚未开始解析
    RecordingDone,
    /// 解析进行中
    Parsing,
    /// 已进入回放，停在首回合首 tick
    ReplayEntered(ReplayEntry),
    /// 交接失败，带原因
    Failed(HandoffFailure),
}

/// 交接结果：成功落点 + 摘要 + 虚拟耗时，或失败原因 + 耗时
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandoffOutcome {
    /// 成功进入回放
    Entered {
        /// 落点证据
        entry: ReplayEntry,
        /// 解析摘要
        summary: ReplaySummary,
        /// 累计虚拟毫秒
        elapsed_ms: u32,
    },
    /// 交接失败
    Failed {
        /// 失败原因
        failure: HandoffFailure,
        /// 失败时已累计的虚拟毫秒
        elapsed_ms: u32,
    },
}

/// 交接状态机：绑定一个解析器和一个虚拟时钟，串行跑「录制完成 → 解析 → 进入回放」。
#[derive(Debug, Clone)]
pub struct HandoffMachine<P, C = FakeClock> {
    parser: P,
    clock: C,
    state: HandoffState,
}

impl<P: ReplayParser, C> HandoffMachine<P, C>
where
    C: VirtualClock,
{
    /// 绑定解析器与虚拟时钟；初始状态为 [`HandoffState::RecordingDone`]
    pub fn new(parser: P, clock: C) -> Self {
        Self {
            parser,
            clock,
            state: HandoffState::RecordingDone,
        }
    }

    /// 当前状态
    #[must_use]
    pub fn state(&self) -> &HandoffState {
        &self.state
    }

    /// 跑一遍交接。`parse_cost_ms` 是本次解析占用的虚拟毫秒（由调用方注入，不读墙钟）。
    pub fn run(&mut self, bundle: &RecordMetadata, parse_cost_ms: u32) -> HandoffOutcome {
        self.state = HandoffState::Parsing;

        let summary = match self.parser.parse(bundle) {
            Ok(summary) => summary,
            Err(error) => {
                let failure = HandoffFailure::ParseFailed {
                    reason: error.to_string(),
                };
                self.state = HandoffState::Failed(failure.clone());
                return HandoffOutcome::Failed {
                    failure,
                    elapsed_ms: self.clock.elapsed_ms(),
                };
            }
        };

        self.clock.advance(parse_cost_ms);
        let elapsed_ms = self.clock.elapsed_ms();

        if elapsed_ms > HANDOFF_BUDGET_MS {
            let failure = HandoffFailure::BudgetExceeded {
                elapsed_ms,
                budget_ms: HANDOFF_BUDGET_MS,
            };
            self.state = HandoffState::Failed(failure.clone());
            return HandoffOutcome::Failed {
                failure,
                elapsed_ms,
            };
        }

        if summary.player_count != EXPECTED_PLAYER_COUNT {
            let failure = HandoffFailure::PlayerCountMismatch {
                actual: summary.player_count,
                expected: EXPECTED_PLAYER_COUNT,
            };
            self.state = HandoffState::Failed(failure.clone());
            return HandoffOutcome::Failed {
                failure,
                elapsed_ms,
            };
        }

        let drift = bundle.start_tick.abs_diff(summary.first_round_start_tick);
        if drift > TICK_ALIGNMENT_TOLERANCE {
            let failure = HandoffFailure::TickDrift {
                expected: bundle.start_tick,
                actual: summary.first_round_start_tick,
                tolerance: TICK_ALIGNMENT_TOLERANCE,
            };
            self.state = HandoffState::Failed(failure.clone());
            return HandoffOutcome::Failed {
                failure,
                elapsed_ms,
            };
        }

        let entry = ReplayEntry {
            tick: summary.first_round_start_tick,
            round_index: 0,
        };
        // 落点断言：进入回放时必须停在摘要给出的首回合首 tick
        debug_assert_eq!(entry.tick, summary.first_round_start_tick);
        self.state = HandoffState::ReplayEntered(entry);
        HandoffOutcome::Entered {
            entry,
            summary,
            elapsed_ms,
        }
    }
}

/// 占位解析器：返回「尚未接入」错误；真实解析后端待后续阶段接入。
#[derive(Debug, Clone, Copy, Default)]
pub struct PendingParser;

impl ReplayParser for PendingParser {
    fn parse(&mut self, _bundle: &RecordMetadata) -> Result<ReplaySummary, ParseError> {
        Err(ParseError::NotImplemented)
    }
}

/// 脚本化假解析器：按预设结果返回，专供自检 / 单测注入；不是真实解析后端。
#[derive(Debug, Clone)]
pub struct ScriptedParser {
    next: Result<ReplaySummary, ParseError>,
}

impl ScriptedParser {
    /// 预设返回一份摘要
    #[must_use]
    pub fn returns(summary: ReplaySummary) -> Self {
        Self { next: Ok(summary) }
    }

    /// 预设解析失败，带说明
    #[must_use]
    pub fn fails(reason: impl Into<String>) -> Self {
        Self {
            next: Err(ParseError::Other(reason.into())),
        }
    }
}

impl ReplayParser for ScriptedParser {
    fn parse(&mut self, _bundle: &RecordMetadata) -> Result<ReplaySummary, ParseError> {
        self.next.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EXPECTED_PLAYER_COUNT, FakeClock, HANDOFF_BUDGET_MS, HandoffFailure, HandoffMachine,
        HandoffOutcome, ParseError, ReplayParser, ReplaySummary, ScriptedParser,
        TICK_ALIGNMENT_TOLERANCE,
    };
    use crate::interval::RecordingInterval;
    use crate::tasks::plan_round;

    /// 造一份假录像清单（阶段一编排产物，不含真实文件）
    fn bundle() -> crate::metadata::RecordMetadata {
        let interval = RecordingInterval::new(96, 320).unwrap();
        let plan = plan_round("de_mirage", 4, interval, 3, &[1, 5, 10], "out").unwrap();
        plan.to_metadata(64.0)
    }

    fn summary(first_tick: u32, players: u8) -> ReplaySummary {
        ReplaySummary {
            first_round_start_tick: first_tick,
            player_count: players,
            round_count: 1,
        }
    }

    #[test]
    fn happy_path_enters_on_first_tick_within_budget() {
        let bundle = bundle();
        let mut machine = HandoffMachine::new(
            ScriptedParser::returns(summary(bundle.start_tick, 10)),
            FakeClock::default(),
        );
        let outcome = machine.run(&bundle, 12);

        match outcome {
            HandoffOutcome::Entered {
                entry,
                summary,
                elapsed_ms,
            } => {
                assert!(elapsed_ms <= HANDOFF_BUDGET_MS, "耗时 {elapsed_ms} 超预算");
                assert_eq!(entry.tick, summary.first_round_start_tick);
                assert_eq!(entry.round_index, 0);
                assert_eq!(entry.tick, bundle.start_tick);
                assert_eq!(summary.player_count, 10);
            }
            HandoffOutcome::Failed { failure, .. } => panic!("正常路径不应失败：{failure}"),
        }
    }

    #[test]
    fn parse_failure_lands_in_failed_with_reason() {
        let bundle = bundle();
        let mut machine =
            HandoffMachine::new(ScriptedParser::fails("demo 损坏"), FakeClock::default());
        let outcome = machine.run(&bundle, 5);
        match outcome {
            HandoffOutcome::Failed {
                failure: HandoffFailure::ParseFailed { reason },
                ..
            } => assert!(reason.contains("demo 损坏")),
            other => panic!("应进 ParseFailed，实际 {other:?}"),
        }
    }

    #[test]
    fn tick_drift_three_and_five_both_fail() {
        let bundle = bundle();
        for drift in [3, 5] {
            let mut machine = HandoffMachine::new(
                ScriptedParser::returns(summary(bundle.start_tick + drift, 10)),
                FakeClock::default(),
            );
            let outcome = machine.run(&bundle, 5);
            assert!(
                matches!(
                    outcome,
                    HandoffOutcome::Failed {
                        failure: HandoffFailure::TickDrift { .. },
                        ..
                    }
                ),
                "漂移 {drift} 应判 TickDrift"
            );
        }
    }

    #[test]
    fn drift_within_tolerance_two_still_passes() {
        let bundle = bundle();
        let mut machine = HandoffMachine::new(
            ScriptedParser::returns(summary(bundle.start_tick + TICK_ALIGNMENT_TOLERANCE, 10)),
            FakeClock::default(),
        );
        let outcome = machine.run(&bundle, 5);
        assert!(matches!(outcome, HandoffOutcome::Entered { .. }));
    }

    #[test]
    fn nine_players_fail_player_count_mismatch() {
        let bundle = bundle();
        let mut machine = HandoffMachine::new(
            ScriptedParser::returns(summary(bundle.start_tick, 9)),
            FakeClock::default(),
        );
        let outcome = machine.run(&bundle, 5);
        match outcome {
            HandoffOutcome::Failed {
                failure: HandoffFailure::PlayerCountMismatch { actual, expected },
                ..
            } => {
                assert_eq!(actual, 9);
                assert_eq!(expected, EXPECTED_PLAYER_COUNT);
            }
            other => panic!("应判选手数不符，实际 {other:?}"),
        }
    }

    #[test]
    fn budget_exceeded_at_3001_ms_fails() {
        let bundle = bundle();
        let mut machine = HandoffMachine::new(
            ScriptedParser::returns(summary(bundle.start_tick, 10)),
            FakeClock::default(),
        );
        let outcome = machine.run(&bundle, HANDOFF_BUDGET_MS + 1);
        match outcome {
            HandoffOutcome::Failed {
                failure:
                    HandoffFailure::BudgetExceeded {
                        elapsed_ms,
                        budget_ms,
                    },
                ..
            } => {
                assert_eq!(elapsed_ms, 3001);
                assert_eq!(budget_ms, HANDOFF_BUDGET_MS);
            }
            other => panic!("应判预算超支，实际 {other:?}"),
        }
    }

    #[test]
    fn pending_parser_reports_not_implemented() {
        let bundle = bundle();
        let mut machine = HandoffMachine::new(super::PendingParser, FakeClock::default());
        let outcome = machine.run(&bundle, 0);
        assert!(matches!(
            outcome,
            HandoffOutcome::Failed {
                failure: HandoffFailure::ParseFailed { .. },
                ..
            }
        ));
        let mut parser = super::PendingParser;
        assert!(matches!(
            parser.parse(&bundle),
            Err(ParseError::NotImplemented)
        ));
    }
}
