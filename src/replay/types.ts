/**
 * 回放数据模型：与未来 `replay.json`（T-01）对齐的前端契约。
 * 坐标一律归一化到 0..1，左下角为原点，方向角单位为度、0 表示朝右、逆时针为正。
 */
export type TeamSide = 't' | 'ct';

export type PlayerMeta = {
  /** 1..10，与左栏 P1–P10 一一对应 */
  slot: number;
  name: string;
  team: TeamSide;
};

export type PlayerFrame = {
  slot: number;
  x: number;
  y: number;
  dir: number;
  hp: number;
  alive: boolean;
};

export type Frame = {
  tick: number;
  players: PlayerFrame[];
};

export type RoundInfo = {
  /** 0 起 */
  index: number;
  startTick: number;
  /** 含端点 */
  endTick: number;
  winner: TeamSide | null;
};

export type KillEvent = {
  tick: number;
  killer: number;
  victim: number;
  headshot: boolean;
};

export type ReplayMeta = {
  /** demo 文件路径（导出时写入，仅供排查） */
  source: string;
  /** 采样间隔 tick */
  tickStep: number;
  /** 归一化口径：目前是 bbox-uniform（等比例包围盒），T-02 换雷达校准 */
  normalize: string;
  /** 世界坐标包围盒 minX/minY/maxX/maxY */
  bounds: [number, number, number, number];
  controllers: number;
  unmappedKills: number;
  worldKills: number;
};

export type Replay = {
  map: string;
  tickRate: number;
  players: PlayerMeta[];
  rounds: RoundInfo[];
  /** 按 tick 升序，供二分查找 */
  frames: Frame[];
  events: KillEvent[];
  /** 解析来源与归一化信息；前端样本数据可以不带 */
  meta?: ReplayMeta;
};
