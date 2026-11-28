use source2_demo::proto::CSvcMsgServerInfo;
use std::collections::BTreeMap;

/// 游戏事件里的整数取值
fn event_u8(ge: &GameEvent, key: &str) -> Option<u8> {
    ge.get_value(key).ok()?.try_into().ok()
}

fn event_u32(ge: &GameEvent, key: &str) -> Option<u32> {
    ge.get_value(key).ok()?.try_into().ok()
}

fn event_bool(ge: &GameEvent, key: &str) -> Option<bool> {
    ge.get_value(key).ok()?.try_into().ok()
}

/// 一帧里的一行原始数据：槽位、世界 x、世界 y、朝向、血量、是否存活
pub type RawRow = (u8, f32, f32, f32, i32, bool);

/// Source 2 格子边长（Hammer 单位）：2^9
pub const CELL_SIZE: f32 = 512.0;
/// 世界可寻址范围的一半：格子网格从 -WORLD_HALF 开始
pub const WORLD_HALF: f32 = 16384.0;

/// 把「格子编号 + 格内偏移」还原成世界坐标：cell * CELL_SIZE - WORLD_HALF + offset
pub fn cell_to_world(cell: u16, offset: f32) -> f32 {
    f32::from(cell) * CELL_SIZE - WORLD_HALF + offset
}

/// 花名册里的一名选手：控制器实体序号 → 槽位
#[derive(Clone)]
struct Roster {
    controller_key: u32,
    slot: u8,
    name: String,
    team: String,
}

#[derive(Default)]
struct RoundAcc {
    start_tick: u32,
    end_tick: u32,
    winner: Option<String>,
}

#[derive(Default)]
struct Probe {
    step: u32,
    tick_rate: f64,
    map: String,
    roster: Vec<Roster>,
    rounds: Vec<RoundAcc>,
    current_round: Option<usize>,
    events: Vec<KillEvent>,
    unmapped_kills: u32,
    world_kills: u32,
    /// 槽位 → 最后一次已知的世界坐标与朝向，死亡后角色实体解绑时沿用
    last_pos: BTreeMap<u8, (f32, f32, f32)>,
    bounds: Option<[f32; 4]>,
    /// 采样得到的原始世界坐标，等解析完拿到全局包围盒再归一化
    raw_frames: Vec<(u32, Vec<RawRow>)>,
    /// 调试用：只打印第一次见到的事件名
    seen_events: Vec<String>,
}

impl Probe {
    fn slot_for(&self, controller_key: u32) -> Option<u8> {        self.roster
            .iter()
            .find(|r| r.controller_key == controller_key)
            .map(|r| r.slot)
    }

    fn team_name(team: u8) -> Option<&'static str> {
        match team {
            2 => Some("t"),
            3 => Some("ct"),
            _ => None,
        }
    }

    fn build_roster(&mut self, ctx: &Context) -> bool {
        let mut t_side = Vec::new();
        let mut ct_side = Vec::new();
        for entity in ctx.entities().iter() {
            if entity.class().name() != "CCSPlayerController" {
                continue;
            }
            let name: String =
                try_property!(entity, String, "m_iszPlayerName").unwrap_or_default();
            if name.trim().is_empty() {
                continue;
            }
            let Some(team_byte) = try_property!(entity, u8, "m_iTeamNum") else {
                continue;
            };
            let Some(team) = Probe::team_name(team_byte) else {
                continue;
            };
            let entry = (entity.index(), name, team);
            if team == "t" {
                t_side.push(entry);
            } else {
                ct_side.push(entry);
            }
        }
        t_side.sort_by_key(|e| e.0);
        ct_side.sort_by_key(|e| e.0);
        if std::env::var("TACTICDECK_DEBUG_ROSTER").is_ok() {
            eprintln!("T 侧 {} 人：{:?}", t_side.len(), t_side);
            eprintln!("CT 侧 {} 人：{:?}", ct_side.len(), ct_side);
        }

        for (offset, (key, name, team)) in t_side
            .into_iter()
            .chain(ct_side)
            .enumerate()
            .take(10)
        {
            let slot = offset as u8 + 1;
            self.roster.push(Roster {
                controller_key: key,
                slot,
                name,
                team: team.to_string(),
            });
        }
        !self.roster.is_empty()
    }

    fn sample(&mut self, ctx: &Context) {
        let roster = self.roster.clone();
        let mut row = Vec::with_capacity(10);
        for member in &roster {
            let Some(controller) = ctx
                .entities()
                .iter()
                .find(|e| e.index() == member.controller_key)
            else {
                continue;
            };
            let pawn_handle: u32 = try_property!(controller, u32, "m_hPlayerPawn").unwrap_or(0);
            // 个别 demo 的 controller→pawn 句柄是空的（例如某两名选手整局都没绑定上），
            // 退一步扫 pawn 实体，用它的 m_hController 反查回这名控制器
            let pawn = ctx
                .entities()
                .get_by_handle(pawn_handle as usize)
                .ok()
                .or_else(|| find_pawn_by_controller(ctx, member.controller_key));
            let Some(pawn) = pawn else {
                // 角色实体解绑（一般是已经阵亡），沿用最后已知位置
                if let Some((x, y, dir)) = self.last_pos.get(&member.slot).copied() {
                    row.push((member.slot, x, y, dir, 0, false));
                }
                continue;
            };
        let (world_x, world_y, yaw, hp, alive) = match std::panic::catch_unwind(
            std::panic::AssertUnwindSafe(|| read_pawn(pawn)),
        ) {
            Ok(values) => values,
            // 某个字段在这个 demo 里读不通：这一帧跳过这名选手，别把整个解析线程带崩
            Err(_) => continue,
        };
            self.last_pos
                .insert(member.slot, (world_x, world_y, yaw));
            row.push((member.slot, world_x, world_y, yaw, hp, alive));
        }
        if row.is_empty() {
            return;
        }
        if row.len() != 10 && std::env::var("TACTICDECK_DEBUG_ROWS").is_ok() {
            let have: Vec<u8> = row.iter().map(|(slot, ..)| *slot).collect();
            eprintln!("tick {} 只有 {} 人：{have:?}", ctx.tick(), row.len());
        }
        let bounds = self.bounds.get_or_insert([f32::MAX, f32::MAX, f32::MIN, f32::MIN]);
        for (_, x, y, _, _, _) in &row {
            bounds[0] = bounds[0].min(*x);
            bounds[1] = bounds[1].min(*y);
            bounds[2] = bounds[2].max(*x);
            bounds[3] = bounds[3].max(*y);
        }
        self.raw_frames.push((ctx.tick(), row));
    }
}

/// 从 pawn 实体读出世界坐标、朝向、血量、存活与否
fn read_pawn(pawn: &Entity) -> (f32, f32, f32, i32, bool) {
    let cell_x: u16 =
        try_property!(pawn, u16, "CBodyComponent.m_skeletonInstance.m_vecOrigin.m_cellX")
            .unwrap_or_default();
    let vec_x: f32 =
        try_property!(pawn, f32, "CBodyComponent.m_skeletonInstance.m_vecOrigin.m_vecX")
            .unwrap_or_default();
    let cell_y: u16 =
        try_property!(pawn, u16, "CBodyComponent.m_skeletonInstance.m_vecOrigin.m_cellY")
            .unwrap_or_default();
    let vec_y: f32 =
        try_property!(pawn, f32, "CBodyComponent.m_skeletonInstance.m_vecOrigin.m_vecY")
            .unwrap_or_default();
    let hp: i32 = try_property!(pawn, i32, "m_iHealth").unwrap_or(0);
    let life: u8 = try_property!(pawn, u8, "m_lifeState").unwrap_or(0);
    let angles: [f32; 3] =
        try_property!(pawn, [f32; 3], "m_angEyeAngles").unwrap_or([0.0, 0.0, 0.0]);

    // Source 2 的世界坐标是「格子 + 格内偏移」两段式存的：格子边长 512，
    // 世界原点在 -16384（格子网格以世界原点为中心）。少了这一步坐标会整体偏掉。
    let world_x = cell_to_world(cell_x, vec_x);
    let world_y = cell_to_world(cell_y, vec_y);
    (world_x, world_y, angles[1], hp, life == 0 && hp > 0)
}

/// 实体句柄低 14 位才是实体序号
fn handle_entity_index(handle: u32) -> usize {
    (handle as usize) & 0x3fff
}

/// 扫全场的 player pawn，找出 `m_hController` 指向这名控制器的那个
///
/// 不同版本的 demo 里 pawn 的字段名不完全一致，缺字段时上游库会 panic，
/// 所以这里把单次属性读取包进 `catch_unwind`，读不到就当它不是目标。
fn find_pawn_by_controller(ctx: &Context, controller_key: u32) -> Option<&Entity> {
    ctx.entities().iter().find(|e| {
        matches!(e.class().name(), "C_CSPlayerPawn" | "CCSPlayerPawn")
            && controller_handle_of(e).is_some_and(|h| handle_entity_index(h) == controller_key as usize)
    })
}

/// 读 pawn 的 `m_hController`，读不到或字库对不上时返回 None 而不是让解析线程崩掉
fn controller_handle_of(entity: &Entity) -> Option<u32> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        try_property!(entity, u32, "m_hController")
    }))
    .ok()
    .flatten()
}

impl Observer for Probe {
    fn interests(&self) -> Interests {
        Interests::TICK_START
            | Interests::ENTITY_STATE
            | Interests::BASE_GAME_EVENT
            | Interests::SVC_MESSAGE
            | Interests::REPLAY_END
    }

    fn on_svc_message(
        &mut self,
        _ctx: &Context,
        msg_type: SvcMessages,
        msg: &[u8],
    ) -> ObserverResult {
        if msg_type == SvcMessages::SvcServerInfo {
            match CSvcMsgServerInfo::decode(msg) {
                Ok(info) => {
                    if let Some(map) = info.map_name {
                        self.map = map;
                    }
                    if let Some(interval) = info.tick_interval.filter(|value| *value > 0.0) {
                        self.tick_rate = 1.0 / interval as f64;
                    }
                }
                Err(error) => eprintln!("服务端信息解析失败: {error}"),
            }
        }
        Ok(())
    }

    fn on_tick_start(&mut self, ctx: &Context) -> ObserverResult {
        let tick = ctx.tick();
        if tick == u32::MAX {
            return Ok(());
        }
        if self.step == 0 {
            self.step = 8;
        }
        if self.roster.is_empty() {
            // 个别 demo 的实体字段与上游库的 schema 不完全一致，读属性会 panic；
            // 兜一层，崩了就跳过这一 tick，别整份录像都解析不出来
            let ok = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.build_roster(ctx)))
                .unwrap_or(false);
            if !ok {
                return Ok(());
            }
        }
        if self.tick_rate <= 0.0 {
            self.tick_rate = 64.0;
        }
        if self.current_round.is_none() || !tick.is_multiple_of(self.step) {
            return Ok(());
        }
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.sample(ctx)));
        Ok(())
    }

    fn on_game_event(&mut self, ctx: &Context, ge: &GameEvent) -> ObserverResult {
        let tick = ctx.tick();
        if std::env::var("TACTICDECK_DEBUG_EVENTS").is_ok() {
            let name = ge.name().to_string();
            if !self.seen_events.contains(&name) {
                eprintln!("事件 {name} @ {tick}");
                self.seen_events.push(name);
            }
        }
        match ge.name() {
            // CS2 用 round_prestart / round_officially_ended；CS:GO 老名字一起接住
            "round_start" | "round_prestart" => {
                // 上一回合如果没等到结束事件（中途重启、录像截断），就在这里收口
                if let Some(idx) = self.current_round
                    && self.rounds[idx].end_tick == 0
                {
                    self.rounds[idx].end_tick = tick;
                }
                self.rounds.push(RoundAcc {
                    start_tick: tick,
                    end_tick: 0,
                    winner: None,
                });
                self.current_round = Some(self.rounds.len() - 1);
            }
            "round_end" | "round_officially_ended" => {
                let winner = event_u8(ge, "winner")
                    .and_then(Probe::team_name)
                    .map(|w| w.to_string());
                if let Some(idx) = self.current_round {
                    self.rounds[idx].end_tick = tick;
                    self.rounds[idx].winner = winner;
                }
                self.current_round = None;
            }
            "player_death" => {
                let Some(victim_key) = event_u32(ge, "userid") else {
                    return Ok(());
                };
                let attacker_key = event_u32(ge, "attacker").unwrap_or(0);
                if attacker_key == 0 || attacker_key == victim_key {
                    // 环境致死或自杀：本来就没有击杀者，单独计数
                    self.world_kills += 1;
                    return Ok(());
                }
                let headshot = event_bool(ge, "headshot").unwrap_or(false);
                match (self.slot_for(attacker_key), self.slot_for(victim_key)) {
                    (Some(killer), Some(victim)) => self.events.push(KillEvent {
                        tick,
                        killer,
                        victim,
                        headshot,
                    }),
                    _ => self.unmapped_kills += 1,
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn on_stop(&mut self, _ctx: &Context) -> ObserverResult {
        // 归一化放到 main 里做：那里才知道有没有地图标定文件
        self.raw_frames.sort_by_key(|(tick, _)| *tick);
        if let Some(last) = self.rounds.last_mut()
            && last.end_tick == 0
        {
            last.end_tick = self
                .raw_frames
                .last()
                .map(|(tick, _)| *tick)
                .unwrap_or(last.start_tick);
        }
        Ok(())
    }
}
