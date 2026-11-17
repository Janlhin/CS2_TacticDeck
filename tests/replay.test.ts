/// <reference lib="deno.ns" />
import { buildPartialReplay, buildSampleReplay } from '../src/replay/fixture.ts';
import { TEAM_COLOR } from '../src/replay/geometry.ts';
import {
  aliveCount,
  frameAt,
  killsUpTo,
  rosterAt,
  roundAt,
  roundIndexAt,
  seekTick,
  totalTicks,
  validateTenPlayers,
} from '../src/replay/selectors.ts';
import { buildViews, loadedCount, rotateView, SLOT_COUNT } from '../src/store/rail.ts';

const replay = buildSampleReplay();

Deno.test('2d：板面坐标与朝向可用', () => {
  const frame = frameAt(replay, 320);
  if (!frame) throw new Error('tick 320 应取到帧');
  if (frame.players.length !== 10) throw new Error('每帧应有 10 名选手');
  for (const player of frame.players) {
    if (!(player.x >= 0 && player.x <= 1)) throw new Error(`x 越界: ${player.x}`);
    if (!(player.y >= 0 && player.y <= 1)) throw new Error(`y 越界: ${player.y}`);
    if (!Number.isFinite(player.dir)) throw new Error('朝向必须是有限数值');
  }
  if (TEAM_COLOR.t === TEAM_COLOR.ct) throw new Error('两队配色不能相同');
});

Deno.test('2d：轨迹采样连续（相邻帧位移有限）', () => {
  const a = frameAt(replay, 100);
  const b = frameAt(replay, 102);
  if (!a || !b) throw new Error('相邻帧必须都存在');
  for (const player of a.players) {
    const next = b.players.find((item) => item.slot === player.slot);
    if (!next) throw new Error(`帧间丢失槽位 ${player.slot}`);
    const dist = Math.hypot(next.x - player.x, next.y - player.y);
    if (dist > 0.12) throw new Error(`槽位 ${player.slot} 单帧位移过大: ${dist.toFixed(3)}`);
  }
});

Deno.test('ten-player：任意 tick 都能取到 10 个点位', () => {
  const end = totalTicks(replay);
  for (let tick = 0; tick <= end; tick += 37) {
    const result = validateTenPlayers(replay, tick);
    if (!result.ok) throw new Error(`tick ${tick} 缺人: ${result.missing.join(',')}`);
  }
});

Deno.test('ten-player：缺 1 人即失败', () => {
  const partial = buildPartialReplay();
  const result = validateTenPlayers(partial, 200);
  if (result.ok) throw new Error('缺 P10 时不应判定通过');
  if (result.missing.length !== 1 || result.missing[0] !== 10) {
    throw new Error(`缺失槽位应为 [10]，实际 ${JSON.stringify(result.missing)}`);
  }
});

Deno.test('rail：11 个槽位与轮换、绑定计数', () => {
  const views = buildViews();
  const first = views[0];
  if (views.length !== SLOT_COUNT || SLOT_COUNT !== 11) throw new Error('应为 11 个槽位');
  if (!first || first.kind !== 'overview') throw new Error('第 1 槽必须是 2D 总览');
  if (loadedCount(views) !== 0) throw new Error('初始不应有已绑定槽位');

  let active = 1;
  const seen: number[] = [];
  for (let step = 0; step < 10; step += 1) {
    active = rotateView(views, active, 1);
    seen.push(active);
  }
  if (new Set(seen).size !== 10) throw new Error('轮换应遍历全部十人');
  const start = seen[0];
  if (start === undefined || rotateView(views, active, 1) !== start) throw new Error('轮换应回到起点');
});

Deno.test('panels：播放头前进后右栏数据同步', () => {
  const before = rosterAt(replay, 300);
  const after = rosterAt(replay, 520);

  if (before.length !== 10) throw new Error('右栏应总有 10 行');
  const moved = before.some((row, index) => {
    const b = after[index];
    const aFrame = row.frame;
    const bFrame = b?.frame;
    return aFrame && bFrame && (aFrame.x !== bFrame.x || aFrame.y !== bFrame.y);
  });
  if (!moved) throw new Error('播放头前进后点位应变化');

  if (roundAt(replay, 300)?.index !== roundAt(replay, 520)?.index) {
    throw new Error('同一回合内回合号不应变化');
  }
  if (roundIndexAt(replay, 640) !== 1) throw new Error('640 tick 应进入第 2 回合');

  const kills = killsUpTo(replay, 620);
  if (kills.length !== 3) throw new Error(`第 1 回合到 620 tick 应有 3 次击杀，实际 ${kills.length}`);
  if (aliveCount(replay, 620, 't') + aliveCount(replay, 620, 'ct') >= 10) {
    throw new Error('已有击杀时存活人数应少于 10');
  }
  // 换回合后满员复活
  if (aliveCount(replay, 640, 't') + aliveCount(replay, 640, 'ct') !== 10) {
    throw new Error('新回合开始应十人全员存活');
  }
});

Deno.test('timeline：跳转耗时 < 200 ms', () => {
  const end = totalTicks(replay);
  let worst = 0;
  for (let i = 0; i < 50; i += 1) {
    const target = (i * 7919) % (end + 1);
    const t0 = performance.now();
    const frame = seekTick(replay, target);
    const cost = performance.now() - t0;
    if (!frame) throw new Error(`跳转到 ${target} 失败`);
    worst = Math.max(worst, cost);
  }
  if (worst >= 200) throw new Error(`最慢跳转 ${worst.toFixed(2)} ms，超过 200 ms 预算`);
});
