use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use source2_demo::prelude::*;
use std::fs::File;
use std::io::BufReader;

include!("schema.rs");
include!("probe.rs");

use tacticdeck_map::map_profile::{self, MapProfile, MapProfiles};

/// 把采到的世界坐标按「地图雷达标定」或「包围盒」压到 0..1
fn build_frames(probe: &Probe, profile: Option<&MapProfile>) -> Vec<Frame> {
    let [min_x, min_y, max_x, max_y] = probe.bounds.unwrap_or([0.0, 0.0, 1.0, 1.0]);
    let span = (max_x - min_x).max(max_y - min_y).max(1.0);
    let center_x = (min_x + max_x) * 0.5;
    let center_y = (min_y + max_y) * 0.5;

    probe
        .raw_frames
        .iter()
        .filter_map(|(tick, rows)| {
            let players: Vec<PlayerFrame> = rows
                .iter()
                .map(|(slot, x, y, yaw, hp, alive)| {
                    let (nx, ny) = match profile {
                        Some(profile) => profile.world_to_norm(*x, *y),
                        // 没有标定时退化成等比例包围盒：不拉变形，但对不上雷达图
                        None => (0.5 + (x - center_x) / span, 0.5 - (y - center_y) / span),
                    };
                    PlayerFrame {
                        slot: *slot,
                        x: nx,
                        y: ny,
                        dir: *yaw,
                        hp: *hp,
                        alive: *alive,
                    }
                })
                .collect();
            // 只留「十人齐全、坐标都落在 0..1」的帧：热身与掉线帧、个别实体读歪的帧直接丢
            if players.len() != 10
                || !players
                    .iter()
                    .all(|p| (0.0..=1.0).contains(&p.x) && (0.0..=1.0).contains(&p.y))
            {
                return None;
            }
            Some(Frame {
                tick: *tick,
                players,
            })
        })
        .collect()
}

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
#[allow(unknown_lints, clippy::large_unopt_stack_frames)]
fn run() -> Result<()> {
    let mut demo: Option<String> = None;
    let mut out: Option<String> = None;
    let mut validate: Option<String> = None;
    let mut step: u32 = 8;
    let mut extract_maps = false;
    let mut extract_radar: Option<String> = None;
    let mut check_radar: Option<String> = None;
    let mut cs2_dir: Option<String> = None;
    let mut map_profile_path: Option<String> = None;

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
            "--extract-maps" => extract_maps = true,
            "--extract-radar" => {
                i += 1;
                extract_radar = args.get(i).cloned();
            }
            "--check-radar" => {
                i += 1;
                check_radar = args.get(i).cloned();
            }
            "--cs2" => {
                i += 1;
                cs2_dir = args.get(i).cloned();
            }
            "--map-profile" => {
                i += 1;
                map_profile_path = args.get(i).cloned();
            }
            other => anyhow::bail!("未知参数 {other}"),
        }
        i += 1;
    }

    if let Some(path) = validate {
        return validate_replay(&path);
    }

    if extract_maps {
        let cs2 = map_profile::discover_cs2(cs2_dir.as_deref().map(std::path::Path::new))?;
        let vpk_path = cs2.join("pak01_dir.vpk");
        println!("CS2 目录：{}", cs2.display());
        let profiles = map_profile::extract_from_vpk(&vpk_path)?;
        let out = out.unwrap_or_else(|| "map-profile.json".to_string());
        let json = serde_json::to_string(&profiles).context("序列化 map-profile.json 失败")?;
        std::fs::write(&out, json).with_context(|| format!("写 {out} 失败"))?;
        println!(
            "地图 {} 张 → {out}（来源 {}）",
            profiles.maps.len(),
            profiles.source
        );
        if let Some(mirage) = profiles.maps.iter().find(|map| map.map == "de_mirage") {
            println!(
                "de_mirage: pos_x={} pos_y={} scale={} rotate={}",
                mirage.pos_x, mirage.pos_y, mirage.scale, mirage.rotate
            );
        }
        return Ok(());
    }

    if let Some(map) = extract_radar {
        let out = out.unwrap_or_else(|| format!("{map}_radar.png"));
        let png =
            tacticdeck_map::radar::radar_png(&map, cs2_dir.as_deref().map(std::path::Path::new))?;
        std::fs::write(&out, &png).with_context(|| format!("写 {out} 失败"))?;
        println!("{map} 雷达底图 → {out}（{} 字节）", png.len());
        return Ok(());
    }

    if let Some(path) = check_radar {
        let cs2 = cs2_dir.as_deref().map(std::path::Path::new);
        return check_radar_cover(&path, cs2);
    }

    let demo = demo.context("缺少 --demo <file.dem> 或 --extract-maps")?;
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
    let map_name = if probe.map.is_empty() {
        "unknown".to_string()
    } else {
        probe.map.clone()
    };
    let profiles = match map_profile_path.as_deref() {
        Some(path) => Some(
            serde_json::from_str::<MapProfiles>(
                &std::fs::read_to_string(path).with_context(|| format!("读 {path} 失败"))?,
            )
            .with_context(|| format!("解析 {path} 失败"))?,
        ),
        None => None,
    };
    let profile = profiles
        .as_ref()
        .and_then(|all| all.maps.iter().find(|item| item.map == map_name));
    if profiles.is_some() && profile.is_none() {
        eprintln!("地图 {map_name} 没有标定参数，退回包围盒归一化");
    }
    if profile.is_some() {
        eprintln!("坐标换算走雷达标定（pos_x / pos_y / scale）");
    }
    let frames = build_frames(&probe, profile);
    let bounds = probe.bounds.unwrap_or([0.0, 0.0, 1.0, 1.0]);
    let replay = Replay {
        map: map_name.clone(),
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
        frames,
        events: {
            let mut events = probe.events.clone();
            events.sort_by_key(|e| e.tick);
            events
        },
        meta: ReplayMeta {
            source: demo.clone(),
            tick_step: probe.step,
            normalize: if profile.is_some() {
                "radar".to_string()
            } else {
                "bbox-uniform".to_string()
            },
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

/// 把回放里每个点位投到雷达图上，算「落在不透明像素上的比例」
fn check_radar_cover(path: &str, cs2_dir: Option<&std::path::Path>) -> Result<()> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("读 {path} 失败"))?;
    let replay: Replay = serde_json::from_str(&raw).context("replay.json 结构不合法")?;
    let image = tacticdeck_map::radar::radar_rgba(&replay.map, cs2_dir)?;
    let width = image.width as usize;
    let height = image.height as usize;
    let solid = |x: usize, y: usize| image.rgba[(y * width + x) * 4 + 3] > 0;

    let mut inside = 0usize;
    let mut total = 0usize;
    let mut outside = Vec::new();
    for frame in &replay.frames {
        for player in &frame.players {
            let px = (player.x * width as f32).round() as isize;
            let py = ((1.0 - player.y) * height as f32).round() as isize;
            total += 1;
            let hit = px >= 0
                && py >= 0
                && (px as usize) < width
                && (py as usize) < height
                && solid(px as usize, py as usize);
            if hit {
                inside += 1;
            } else if outside.len() < 5 {
                outside.push((frame.tick, player.slot, player.x, player.y));
            }
        }
    }
    let rate = if total == 0 {
        0.0
    } else {
        inside as f32 / total as f32 * 100.0
    };
    println!(
        "{}：{} 帧、{} 个点位，落在雷达可走区 {} 个（{:.1}%）",
        replay.map,
        replay.frames.len(),
        total,
        inside,
        rate
    );
    if !outside.is_empty() {
        println!("抽样越界：{outside:?}");
    }
    anyhow::ensure!(
        rate >= 98.0,
        "命中率 {rate:.1}% 低于门槛 98%，坐标口径可能不对"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CELL_SIZE, WORLD_HALF, cell_to_world};

    /// 校准用例：Source 2 的格子边长与世界原点必须是 512 / 16384
    #[test]
    fn coords_cell_to_world_matches_source2_grid() {
        assert_eq!(CELL_SIZE, 512.0);
        assert_eq!(WORLD_HALF, 16384.0);
        // 实测：tick 3200 槽位 1 读到 cellX=30 vecX=223、cellY=30 vecY=367
        assert!((cell_to_world(30, 223.0) - -801.0).abs() < 0.001);
        assert!((cell_to_world(30, 367.0) - -657.0).abs() < 0.001);
        // 负格子也要能还原（世界原点左侧的区域）
        assert!((cell_to_world(0, 100.0) - -16284.0).abs() < 0.001);
    }
}
