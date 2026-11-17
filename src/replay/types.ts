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

export type Replay = {
  map: string;
  tickRate: number;
  players: PlayerMeta[];
  rounds: RoundInfo[];
  /** 按 tick 升序，供二分查找 */
  frames: Frame[];
  events: KillEvent[];
};
