import { clamp01 } from './geometry.ts';
import type { Frame, KillEvent, PlayerFrame, PlayerMeta, Replay, RoundInfo, TeamSide } from './types.ts';

/** 合成样本：3 个回合、每回合 640 tick（64 tick/s ≈ 10 秒），确定性生成，供 2D 板面与联动开发/测试用 */
export const SAMPLE_ROUND_TICKS = 640;
export const SAMPLE_ROUNDS = 3;
export const SAMPLE_TICK_RATE = 64;

/** 帧间隔（tick）——2 即 32 Hz，够画轨迹又不会撑大数组 */
const FRAME_STEP = 2;

const NAMES = ['Ace', 'Bolt', 'Cipher', 'Drift', 'Echo', 'Frost', 'Ghost', 'Haze', 'Ivy', 'Jolt'];

type Wander = {
  baseX: number;
  baseY: number;
  ampX: number;
  ampY: number;
  phase: number;
};

/** 前 5 名 T 从左上出生点展开，后 5 名 CT 从右下展面，路径交叉处即"交火区" */
const WANDERS: Wander[] = [
  { baseX: 0.2, baseY: 0.78, ampX: 0.28, ampY: 0.2, phase: 0.0 },
  { baseX: 0.18, baseY: 0.6, ampX: 0.34, ampY: 0.18, phase: 0.12 },
  { baseX: 0.26, baseY: 0.42, ampX: 0.3, ampY: 0.24, phase: 0.24 },
  { baseX: 0.32, baseY: 0.7, ampX: 0.26, ampY: 0.16, phase: 0.36 },
  { baseX: 0.24, baseY: 0.5, ampX: 0.22, ampY: 0.28, phase: 0.48 },
  { baseX: 0.78, baseY: 0.24, ampX: 0.24, ampY: 0.18, phase: 0.6 },
  { baseX: 0.8, baseY: 0.42, ampX: 0.2, ampY: 0.22, phase: 0.72 },
  { baseX: 0.72, baseY: 0.58, ampX: 0.26, ampY: 0.16, phase: 0.84 },
  { baseX: 0.68, baseY: 0.3, ampX: 0.22, ampY: 0.2, phase: 0.96 },
  { baseX: 0.76, baseY: 0.48, ampX: 0.3, ampY: 0.24, phase: 1.08 },
];

const KILL_OFFSETS = [200, 340, 500];

function wanderAt(wander: Wander, progress: number): { x: number; y: number; dir: number } {
  const rad = 2 * Math.PI;
  const x = clamp01(wander.baseX + wander.ampX * Math.sin(rad * (progress + wander.phase)));
  const y = clamp01(wander.baseY + wander.ampY * Math.cos(rad * (progress * 1.3 + wander.phase)));

  // 解析导数求朝向：x 对 progress 求导决定 dx，y 对 progress 求导决定 dy（画布 y 向下）
  const dx = wander.ampX * rad * Math.cos(rad * (progress + wander.phase));
  const dy = -wander.ampY * rad * Math.sin(rad * (progress * 1.3 + wander.phase));
  const dir = (Math.atan2(-dy, dx) * 180) / Math.PI;
  return { x, y, dir };
}

function victimOf(round: number, roundVictimIndex: number): number {
  return ((round * 3 + roundVictimIndex) % 10) + 1;
}

function killerOf(victim: number): number {
  return victim <= 5 ? ((victim % 5) + 1) : victim === 6 ? 1 : 6;
}

export function buildSampleReplay(): Replay {
  const players: PlayerMeta[] = NAMES.map((name, index) => ({
    slot: index + 1,
    name,
    team: (index < 5 ? 't' : 'ct') as TeamSide,
  }));

  const rounds: RoundInfo[] = [];
  const events: KillEvent[] = [];
  const frames: Frame[] = [];

  for (let round = 0; round < SAMPLE_ROUNDS; round += 1) {
    const startTick = round * SAMPLE_ROUND_TICKS;
    const endTick = startTick + SAMPLE_ROUND_TICKS - 1;
    rounds.push({
      index: round,
      startTick,
      endTick,
      winner: round % 2 === 0 ? 't' : 'ct',
    });

    const roundKills: KillEvent[] = KILL_OFFSETS.map((offset, killIndex) => {
      const victim = victimOf(round, killIndex);
      return {
        tick: startTick + offset,
        killer: killerOf(victim),
        victim,
        headshot: killIndex === 1,
      };
    });
    events.push(...roundKills);

    for (let localTick = 0; localTick < SAMPLE_ROUND_TICKS; localTick += FRAME_STEP) {
      const tick = startTick + localTick;
      const progress = localTick / SAMPLE_ROUND_TICKS;

      const playerFrames: PlayerFrame[] = WANDERS.map((wander, index) => {
        const slot = index + 1;
        const pos = wanderAt(wander, progress);
        const dying = roundKills.find((kill) => kill.victim === slot && tick >= kill.tick);
        return {
          slot,
          x: pos.x,
          y: pos.y,
          dir: pos.dir,
          hp: dying ? 0 : 100,
          alive: !dying,
        };
      });

      frames.push({ tick, players: playerFrames });
    }
  }

  return {
    map: 'de_mirage',
    tickRate: SAMPLE_TICK_RATE,
    players,
    rounds,
    frames,
    events,
  };
}

/** 缺一人的样本：用于"缺 1 人即失败"的断言 */
export function buildPartialReplay(): Replay {
  const replay = buildSampleReplay();
  return {
    ...replay,
    frames: replay.frames.map((frame) => ({
      ...frame,
      players: frame.players.filter((player) => player.slot !== 10),
    })),
  };
}
