use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use source2_demo::prelude::*;
use std::fs::File;
use std::io::BufReader;

include!("schema.rs");
include!("probe.rs");

fn main() -> Result<()> {
    let handle = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(run)
        .context("起解析线程失败")?;
    handle
        .join()
        .map_err(|_| anyhow::anyhow!("解析线程崩了"))??;
    Ok(())
}

// 解析上下文含 8192 个实体槽，debug 下帧本身就有几百 KB，必须在 64 MiB 栈线程里跑
#[allow(clippy::large_unopt_stack_frames)]
fn run() -> Result<()> {
    let mut demo: Option<String> = None;
    let mut out: Option<String> = None;
    let mut validate: Option<String> = None;
    let mut step: u32 = 8;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--demo" => {
                i += 1;
                demo = args.get(i).cloned();
            }
            "--out" => {
                i += 1;
                out = args.get(i).cloned();
            }
            "--validate" => {
                i += 1;
                validate = args.get(i).cloned();
            }
            "--step" => {
                i += 1;
                if let Some(value) = args.get(i) {
                    step = value.parse().context("--step 必须是正整数")?;
                }
            }
            other => anyhow::bail!("未知参数 {other}"),
        }
        i += 1;
    }

    if let Some(path) = validate {
        return validate_replay(&path);
    }

    let demo = demo.context("缺少 --demo <file.dem>")?;
    let out = out.context("缺少 --out <replay.json>")?;

    let file = File::open(&demo).with_context(|| format!("打开 {demo} 失败"))?;
    let mut parser = Parser::from_reader(BufReader::new(file))?;
    let probe = parser.add_observer(Probe {
        step: step.max(1),
        ..Probe::default()
    });
    parser.run_to_end()?;

    let probe = probe.borrow_mut();
    if probe.roster.is_empty() {
        anyhow::bail!("没有识别到任何选手：demo 可能不是 CS2 比赛录像");
    }
    let bounds = probe.bounds.unwrap_or([0.0, 0.0, 1.0, 1.0]);
    let replay = Replay {
        map: if probe.map.is_empty() {
            "unknown".to_string()
        } else {
            probe.map.clone()
        },
        tick_rate: probe.tick_rate,
        players: probe
            .roster
            .iter()
            .map(|r| PlayerMeta {
                slot: r.slot,
                name: r.name.clone(),
                team: r.team.clone(),
            })
            .collect(),
        rounds: probe
            .rounds
            .iter()
            .enumerate()
            .map(|(index, r)| RoundInfo {
                index: index as u32,
                start_tick: r.start_tick,
                end_tick: r.end_tick,
                winner: r.winner.clone(),
            })
            .collect(),
        frames: probe.frames.clone(),
        events: {
            let mut events = probe.events.clone();
            events.sort_by_key(|e| e.tick);
            events
        },
        meta: ReplayMeta {
            source: demo.clone(),
            tick_step: probe.step,
            normalize: "bbox-uniform".to_string(),
            bounds,
            controllers: probe.roster.len() as u32,
            unmapped_kills: probe.unmapped_kills,
            world_kills: probe.world_kills,
        },
    };

    let json = serde_json::to_string(&replay).context("序列化 replay.json 失败")?;
    std::fs::write(&out, json).with_context(|| format!("写 {out} 失败"))?;
    println!(
        "回合 {} 个，帧 {} 条，击杀 {} 条（未映射 {}），选手 {} 人，地图 {}",
        replay.rounds.len(),
        replay.frames.len(),
        replay.events.len(),
        replay.meta.unmapped_kills,
        replay.players.len(),
        replay.map
    );
    Ok(())
}

fn validate_replay(path: &str) -> Result<()> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("读 {path} 失败"))?;
    let replay: Replay = serde_json::from_str(&raw).context("replay.json 结构不合法")?;

    anyhow::ensure!(!replay.map.is_empty(), "map 为空");
    anyhow::ensure!(replay.tick_rate > 0.0, "tickRate 必须为正");
    anyhow::ensure!(
        replay.players.len() == 10,
        "选手必须 10 人，实际 {}",
        replay.players.len()
    );
    for (index, player) in replay.players.iter().enumerate() {
        anyhow::ensure!(
            player.slot == index as u8 + 1,
            "槽位应连续 1..10，第 {index} 项是 {}",
            player.slot
        );
        anyhow::ensure!(!player.name.is_empty(), "选手 {} 名字为空", player.slot);
        anyhow::ensure!(
            player.team == "t" || player.team == "ct",
            "选手 {} 阵营非法",
            player.slot
        );
    }
    anyhow::ensure!(!replay.rounds.is_empty(), "没有回合");
    anyhow::ensure!(!replay.frames.is_empty(), "没有帧");
    let mut last_tick = 0;
    for frame in &replay.frames {
        anyhow::ensure!(
            frame.tick >= last_tick,
            "帧 tick 必须递增，{} 出现在 {} 之后",
            frame.tick,
            last_tick
        );
        last_tick = frame.tick;
        anyhow::ensure!(
            frame.players.len() == 10,
            "tick {} 只有 {} 名选手",
            frame.tick,
            frame.players.len()
        );
        for player in &frame.players {
            anyhow::ensure!(
                (0.0..=1.0).contains(&player.x) && (0.0..=1.0).contains(&player.y),
                "tick {} 槽位 {} 坐标越界 ({}, {})",
                frame.tick,
                player.slot,
                player.x,
                player.y
            );
        }
    }
    for round in &replay.rounds {
        anyhow::ensure!(
            round.start_tick <= round.end_tick,
            "回合 {} 起止 tick 反了",
            round.index
        );
    }
    for event in &replay.events {
        anyhow::ensure!(
            (1..=10).contains(&event.killer) && (1..=10).contains(&event.victim),
            "击杀事件槽位越界"
        );
    }
    println!(
        "校验通过：{} 个回合、{} 条帧、{} 条击杀、{} 名选手",
        replay.rounds.len(),
        replay.frames.len(),
        replay.events.len(),
        replay.players.len()
    );
    Ok(())
}
