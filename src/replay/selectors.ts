import type { Frame, KillEvent, PlayerFrame, PlayerMeta, Replay, RoundInfo } from './types.ts';

export type RosterRow = {
  meta: PlayerMeta;
  frame: PlayerFrame | null;
};

/** 按 tick 二分查找最近的帧（取 ≤ tick 的最后一帧） */
export function frameIndexAt(replay: Replay, tick: number): number {
  const frames = replay.frames;
  const first = frames[0];
  const last = frames[frames.length - 1];
  if (!first || !last) return -1;
  if (tick <= first.tick) return 0;
  if (tick >= last.tick) return frames.length - 1;

  let low = 0;
  let high = frames.length - 1;
  while (low <= high) {
    const mid = (low + high) >> 1;
    const current = frames[mid];
    const next = frames[mid + 1];
    if (!current) return Math.max(0, low);
    if (current.tick <= tick) {
      if (!next || next.tick > tick) return mid;
      low = mid + 1;
    } else {
      high = mid - 1;
    }
  }
  return Math.max(0, low);
}

export function frameAt(replay: Replay, tick: number): Frame | null {
  const index = frameIndexAt(replay, tick);
  if (index < 0) return null;
  return replay.frames[index] ?? null;
}

export function totalTicks(replay: Replay): number {
  const last = replay.frames[replay.frames.length - 1];
  return last ? last.tick : 0;
}

export function roundIndexAt(replay: Replay, tick: number): number {
  const rounds = replay.rounds;
  const first = rounds[0];
  const last = rounds[rounds.length - 1];
  if (!first || !last) return -1;
  for (const round of rounds) {
    if (tick >= round.startTick && tick <= round.endTick) return round.index;
  }
  return tick < first.startTick ? first.index : last.index;
}

export function roundAt(replay: Replay, tick: number): RoundInfo | null {
  const index = roundIndexAt(replay, tick);
  if (index < 0) return null;
  return replay.rounds.find((round) => round.index === index) ?? null;
}

export function tickToSeconds(replay: Replay, tick: number): number {
  const round = roundAt(replay, tick);
  const relative = round ? tick - round.startTick : tick;
  return relative / replay.tickRate;
}

export function formatSeconds(seconds: number): string {
  const clamped = Math.max(0, seconds);
  return `${clamped.toFixed(1)}s`;
}

/** 十人状态：meta 与当前帧合并，缺帧时 frame=null（不隐藏槽位） */
export function rosterAt(replay: Replay, tick: number): RosterRow[] {
  const frame = frameAt(replay, tick);
  return replay.players.map((meta) => ({
    meta,
    frame: frame?.players.find((player) => player.slot === meta.slot) ?? null,
  }));
}

/** 当前 tick 之前（含）的击杀，按时间倒序；window>0 时只取最近 window tick 内 */
export function killsUpTo(replay: Replay, tick: number, windowTicks = 0): KillEvent[] {
  const floor = windowTicks > 0 ? tick - windowTicks : -Infinity;
  return replay.events
    .filter((event) => event.tick <= tick && event.tick >= floor)
    .sort((a, b) => b.tick - a.tick);
}

/** 队伍存活人数 */
export function aliveCount(replay: Replay, tick: number, team: PlayerMeta['team']): number {
  return rosterAt(replay, tick).filter((row) => row.meta.team === team && row.frame?.alive).length;
}

/** 十人完整性校验：任意 tick 必须取到 10 个不同槽位，缺一人即失败 */
export function validateTenPlayers(replay: Replay, tick: number): { ok: boolean; missing: number[] } {
  const frame = frameAt(replay, tick);
  if (!frame) return { ok: false, missing: replay.players.map((player) => player.slot) };
  const expected = replay.players.map((player) => player.slot);
  const present = new Set(frame.players.map((player) => player.slot));
  const missing = expected.filter((slot) => !present.has(slot));
  return { ok: missing.length === 0, missing };
}

/** 时间轴跳转：clamp + 取帧，供 < 200 ms 断言使用 */
export function seekTick(replay: Replay, tick: number): Frame | null {
  const clamped = Math.min(Math.max(0, tick), totalTicks(replay));
  return frameAt(replay, clamped);
}

export type RoundEventRow = {
  event: KillEvent;
  /** 相对本回合起点的秒数 */
  elapsed: number;
  /** 是否已过播放头 */
  passed: boolean;
};

/** 当前回合事件列表：按时间正序，未到的事件也列出（置灰），便于回看"接下来会死谁" */
export function roundEvents(replay: Replay, tick: number): RoundEventRow[] {
  const round = roundAt(replay, tick);
  if (!round) return [];
  return replay.events
    .filter((event) => event.tick >= round.startTick && event.tick <= round.endTick)
    .sort((a, b) => a.tick - b.tick)
    .map((event) => ({
      event,
      elapsed: (event.tick - round.startTick) / replay.tickRate,
      passed: event.tick <= tick,
    }));
}

export type KillSpot = {
  tick: number;
  victim: number;
  x: number;
  y: number;
};

/** 击杀落点：被杀者死亡瞬间的坐标；scope=round 只看当前回合 */
export function killSpots(replay: Replay, tick: number, scope: 'round' | 'all' = 'round'): KillSpot[] {
  const round = roundAt(replay, tick);
  const spots: KillSpot[] = [];
  for (const event of replay.events) {
    if (event.tick > tick) continue;
    if (scope === 'round' && (!round || event.tick < round.startTick || event.tick > round.endTick)) continue;
    const frame = frameAt(replay, event.tick);
    const victim = frame?.players.find((player) => player.slot === event.victim);
    if (!victim) continue;
    spots.push({ tick: event.tick, victim: event.victim, x: victim.x, y: victim.y });
  }
  return spots;
}

export type HeatCell = {
  col: number;
  row: number;
  count: number;
  /** 0..1 相对热度，最热格为 1 */
  level: number;
};

/**
 * 落点聚合成 cols×rows 网格。行号自上而下（与板面翻转 y 轴后的视觉一致），
 * 这样把热图贴到地图上时位置不会上下颠倒。
 */
export function heatGrid(spots: KillSpot[], cols: number, rows: number): HeatCell[] {
  const counts = new Array<number>(cols * rows).fill(0);
  for (const spot of spots) {
    const col = Math.min(cols - 1, Math.max(0, Math.floor(spot.x * cols)));
    const row = Math.min(rows - 1, Math.max(0, Math.floor((1 - spot.y) * rows)));
    const index = row * cols + col;
    counts[index] = (counts[index] ?? 0) + 1;
  }
  const max = counts.reduce((best, value) => (value > best ? value : best), 0);

  const cells: HeatCell[] = [];
  for (let row = 0; row < rows; row += 1) {
    for (let col = 0; col < cols; col += 1) {
      const count = counts[row * cols + col] ?? 0;
      cells.push({ col, row, count, level: max > 0 ? count / max : 0 });
    }
  }
  return cells;
}
