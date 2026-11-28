/// 与前端 `src/replay/types.ts` 对齐的回放契约：坐标 0..1，左下角为原点。
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerMeta {
    pub slot: u8,
    pub name: String,
    pub team: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerFrame {
    pub slot: u8,
    pub x: f32,
    pub y: f32,
    pub dir: f32,
    pub hp: i32,
    pub alive: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Frame {
    pub tick: u32,
    pub players: Vec<PlayerFrame>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoundInfo {
    pub index: u32,
    pub start_tick: u32,
    pub end_tick: u32,
    pub winner: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KillEvent {
    pub tick: u32,
    pub killer: u8,
    pub victim: u8,
    pub headshot: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReplayMeta {
    pub source: String,
    pub tick_step: u32,
    pub normalize: String,
    pub bounds: [f32; 4],
    pub controllers: u32,
    pub unmapped_kills: u32,
    pub world_kills: u32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Replay {
    pub map: String,
    pub tick_rate: f64,
    pub players: Vec<PlayerMeta>,
    pub rounds: Vec<RoundInfo>,
    pub frames: Vec<Frame>,
    pub events: Vec<KillEvent>,
    pub meta: ReplayMeta,
}
