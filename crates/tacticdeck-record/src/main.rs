//! `tacticdeck-record` 可执行入口：阶段一只提供 `--self-test`，在不触碰真实录像、
//! 不发起任何真实采集的前提下，跑一遍编排与元数据的纯逻辑自检。
//!
//! 退出码：全部通过 0；任一项失败 1；未知参数 2。

use std::process::ExitCode;
use tacticdeck_record::{
    Collector, CollectorError, FakeClock, HANDOFF_BUDGET_MS, HandoffFailure, HandoffMachine,
    HandoffOutcome, PassRequest, RecordMetadata, RecordStep, RecordingInterval, ReplaySummary,
    ScriptedParser, TICK_ALIGNMENT_TOLERANCE, UndecidedCollector, check_first_frame_alignment,
    metadata_file_name, pass_specs, plan_round, video_file_name,
};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => {
            eprintln!("用法：tacticdeck-record --self-test");
            ExitCode::from(2)
        }
        [first] if first == "--self-test" => self_test(),
        [first, ..] => {
            eprintln!("未知参数：{first}");
            ExitCode::from(2)
        }
    }
}

/// 打印一条 PASS/FAIL 结果；返回是否通过，调用方据此累计失败数
fn check(label: &str, ok: bool, detail: &str) -> bool {
    if ok {
        println!("PASS: {label} — {detail}");
    } else {
        println!("FAIL: {label} — {detail}");
    }
    ok
}

fn self_test() -> ExitCode {
    let mut failures: u32 = 0;
    let mut mark = |ok: bool| {
        if !ok {
            failures += 1;
        }
    };

    // 1. 区间切分：N=3，区间 [96, 320]，锁 slots 1 / 5 / 10
    let interval = match RecordingInterval::new(96, 320) {
        Ok(interval) => interval,
        Err(error) => {
            mark(check("区间构造", false, &error.to_string()));
            return finish(failures);
        }
    };
    let specs = match pass_specs(interval, 3, &[1, 5, 10]) {
        Ok(specs) => specs,
        Err(error) => {
            mark(check("区间切分", false, &error.to_string()));
            return finish(failures);
        }
    };
    let slot_list: Vec<u8> = specs.iter().map(|spec| spec.slot).collect();
    mark(check(
        "区间切分",
        specs.len() == 3,
        &format!(
            "N={} 遍，区间 [{}, {}]，slots={slot_list:?}",
            specs.len(),
            interval.start_tick,
            interval.end_tick
        ),
    ));
    let order_ok = specs.iter().all(|spec| {
        let seek_pos = spec
            .steps
            .iter()
            .position(|step| matches!(step, RecordStep::SeekTick { .. }));
        let camera_pos = spec
            .steps
            .iter()
            .position(|step| matches!(step, RecordStep::SpecPlayer { .. }));
        matches!((seek_pos, camera_pos), (Some(i), Some(j)) if i < j)
    });
    mark(check(
        "时序：seek 早于 camera",
        order_ok,
        "每遍步骤均为 暂停4 → seek 96 → spec_mode → spec_player → 录到 320",
    ));

    // 2. 任务列表：整回合 N 遍任务（步骤 + 输出路径 + 元数据条目）
    let plan = match plan_round(
        "de_mirage",
        4,
        interval,
        3,
        &[1, 5, 10],
        "recordings/mirage-r4",
    ) {
        Ok(plan) => plan,
        Err(error) => {
            mark(check("任务列表", false, &error.to_string()));
            return finish(failures);
        }
    };
    let task_lines: Vec<String> = plan
        .tasks
        .iter()
        .map(|task| format!("slot{} → {}", task.slot, task.video_path))
        .collect();
    mark(check(
        "任务列表",
        plan.tasks.len() == 3,
        &task_lines.join("; "),
    ));
    match check_first_frame_alignment(&plan) {
        Ok(()) => mark(check(
            "首帧对齐 ±2",
            true,
            &format!(
                "{} 遍首帧均落在 tick {}（±2）",
                plan.tasks.len(),
                interval.start_tick
            ),
        )),
        Err(error) => mark(check("首帧对齐 ±2", false, &error.to_string())),
    }

    // 3. 命名规则：恒定、补零、非法字符清洗
    let video_slot1 = video_file_name("de_mirage", 4, 1);
    let video_slot10 = video_file_name("de_mirage", 4, 10);
    let metadata_name = metadata_file_name("de_mirage", 4);
    let dirty = video_file_name("de/mirage:ct?|x", 2, 8);
    let determinism = video_slot1 == video_file_name("de_mirage", 4, 1);
    let no_ambiguity = video_slot1 < video_slot10;
    let clean = ['<', '>', ':', '"', '/', '\\', '|', '?', '*']
        .iter()
        .all(|illegal| !dirty.contains(*illegal));
    mark(check("命名恒定", determinism, &video_slot1));
    mark(check(
        "命名补零无排序歧义",
        no_ambiguity,
        &format!("{video_slot1} < {video_slot10}"),
    ));
    mark(check("命名清洗非法字符", clean, &dirty));
    mark(check(
        "元数据文件名",
        metadata_name == "de_mirage_round04_meta.json",
        &metadata_name,
    ));

    // 4. 元数据 schema：校验 + JSON 往返
    let metadata = plan.to_metadata(64.0);
    match metadata.validate() {
        Ok(()) => mark(check(
            "元数据校验",
            true,
            &format!(
                "schema v{}，pass_count={}，tracks={}，区间 [{}, {}]",
                metadata.schema_version,
                metadata.pass_count,
                metadata.tracks.len(),
                metadata.start_tick,
                metadata.end_tick
            ),
        )),
        Err(error) => mark(check("元数据校验", false, &error.to_string())),
    }
    let json = serde_json::to_string_pretty(&metadata).unwrap_or_default();
    match serde_json::from_str::<RecordMetadata>(&json) {
        Ok(back) => mark(check(
            "元数据 JSON 往返",
            back == metadata,
            "序列化后读回结构一致",
        )),
        Err(error) => mark(check("元数据 JSON 往返", false, &error.to_string())),
    }
    println!("样例 tick 元数据 JSON：\n{json}");

    // 5. 非法输入必须被拒绝（不 panic）
    let rejects: Vec<(&str, bool)> = vec![
        ("N=0", pass_specs(interval, 0, &[]).is_err()),
        (
            "N=11",
            pass_specs(interval, 11, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 1]).is_err(),
        ),
        ("slot 0", pass_specs(interval, 1, &[0]).is_err()),
        ("slot 11", pass_specs(interval, 1, &[11]).is_err()),
        ("重复 slot", pass_specs(interval, 2, &[3, 3]).is_err()),
        ("起点 <96", RecordingInterval::new(80, 320).is_err()),
        ("终点 ≤ 起点", RecordingInterval::new(100, 100).is_err()),
    ];
    for (label, rejected) in rejects {
        mark(check(
            &format!("非法输入拒绝：{label}"),
            rejected,
            "返回 Err，未 panic",
        ));
    }
    let mut duplicate = metadata.clone();
    duplicate.tracks[1].slot = duplicate.tracks[0].slot;
    mark(check(
        "元数据拒绝重复 slot",
        duplicate.validate().is_err(),
        "同 slot 两条轨应校验失败",
    ));

    // 6. 采集器占位：全部返回未决错误，文案写明待 Q-01 裁决
    let collector = UndecidedCollector;
    let capabilities = collector.capabilities();
    let request = PassRequest {
        slot: 1,
        seek_tick: interval.start_tick,
        end_tick: interval.end_tick,
        video_path: video_slot1.clone(),
    };
    let outcome = collector.record_pass(&request);
    let undecided = matches!(
        (&capabilities, &outcome),
        (
            Err(CollectorError::Undecided { .. }),
            Err(CollectorError::Undecided { .. })
        )
    );
    let message = outcome
        .err()
        .map(|error| error.to_string())
        .unwrap_or_default();
    mark(check(
        "采集器占位待 Q-01",
        undecided && message.contains("待 Q-01 裁决"),
        &message,
    ));

    // 7. 交接状态机（T-16）：假清单 + 假解析 + 虚拟时钟，不读墙钟、不碰真实录像
    let happy = ReplaySummary {
        first_round_start_tick: metadata.start_tick,
        player_count: 10,
        round_count: 1,
    };
    let mut machine = HandoffMachine::new(ScriptedParser::returns(happy), FakeClock::default());
    let outcome = machine.run(&metadata, 12);
    match outcome {
        HandoffOutcome::Entered {
            entry,
            summary,
            elapsed_ms,
        } => {
            let drift = metadata.start_tick.abs_diff(entry.tick);
            let ok = elapsed_ms <= HANDOFF_BUDGET_MS
                && drift <= TICK_ALIGNMENT_TOLERANCE
                && entry.tick == summary.first_round_start_tick
                && entry.round_index == 0;
            mark(check(
                "交接 — 正常路径",
                ok,
                &format!(
                    "假清单下 {elapsed_ms} ms 进入回放，tick 差 {drift}，停在 tick {}（回合 {}）",
                    entry.tick, entry.round_index
                ),
            ));
        }
        HandoffOutcome::Failed { failure, .. } => {
            mark(check("交接 — 正常路径", false, &failure.to_string()));
        }
    }

    // 7a. 解析失败 → Failed，原因原样带入
    let mut machine = HandoffMachine::new(ScriptedParser::fails("demo 损坏"), FakeClock::default());
    let outcome = machine.run(&metadata, 5);
    let parse_failed = matches!(
        outcome,
        HandoffOutcome::Failed {
            failure: HandoffFailure::ParseFailed { .. },
            ..
        }
    );
    mark(check(
        "交接 — 解析失败进 Failed",
        parse_failed,
        "解析错误原样带入失败原因",
    ));

    // 7b. tick 漂移 3 / 5 → Failed（±2 含端点之外）
    for (label, drift) in [("漂移 3", 3u32), ("漂移 5", 5u32)] {
        let summary = ReplaySummary {
            first_round_start_tick: metadata.start_tick + drift,
            player_count: 10,
            round_count: 1,
        };
        let mut machine =
            HandoffMachine::new(ScriptedParser::returns(summary), FakeClock::default());
        let outcome = machine.run(&metadata, 5);
        let bad = matches!(
            outcome,
            HandoffOutcome::Failed {
                failure: HandoffFailure::TickDrift { .. },
                ..
            }
        );
        mark(check(
            &format!("交接 — tick {label} 进 Failed"),
            bad,
            &format!("摘要 tick = 清单 tick + {drift}"),
        ));
    }

    // 7c. 9 人 → Failed（Q-06 未决：单人降级不在本轮范围）
    let nine = ReplaySummary {
        first_round_start_tick: metadata.start_tick,
        player_count: 9,
        round_count: 1,
    };
    let mut machine = HandoffMachine::new(ScriptedParser::returns(nine), FakeClock::default());
    let outcome = machine.run(&metadata, 5);
    let bad = matches!(
        outcome,
        HandoffOutcome::Failed {
            failure: HandoffFailure::PlayerCountMismatch { .. },
            ..
        }
    );
    mark(check(
        "交接 — 9 人进 Failed",
        bad,
        "十人回放前提不满足（Q-06 单人降级未决）",
    ));

    // 7d. 预算超 3000 ms（假解析耗时 3001）→ Failed
    let mut machine = HandoffMachine::new(ScriptedParser::returns(happy), FakeClock::default());
    let outcome = machine.run(&metadata, HANDOFF_BUDGET_MS + 1);
    let bad = matches!(
        outcome,
        HandoffOutcome::Failed {
            failure: HandoffFailure::BudgetExceeded { .. },
            ..
        }
    );
    mark(check(
        "交接 — 预算超 3000 ms 进 Failed",
        bad,
        "假解析耗时 3001 ms",
    ));

    finish(failures)
}

fn finish(failures: u32) -> ExitCode {
    if failures == 0 {
        println!("self-test 全部通过");
        ExitCode::from(0)
    } else {
        println!("self-test 有 {failures} 项失败");
        ExitCode::from(1)
    }
}
