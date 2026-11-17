/// <reference lib="deno.ns" />

import { buildSampleReplay } from '../src/replay/fixture.ts';
import { heatGrid, killSpots, roundEvents } from '../src/replay/selectors.ts';

Deno.test('panels：本回合事件按时间正序，未到的置灰', () => {
  const replay = buildSampleReplay();

  const early = roundEvents(replay, 100);
  if (early.length !== 3) throw new Error(`本回合应有 3 条事件，实际 ${early.length}`);
  if (early.some((row) => row.passed)) throw new Error('tick 100 时三条都应未到');
  if (early[0]?.event.tick !== 200 || early[1]?.event.tick !== 340 || early[2]?.event.tick !== 500) {
    throw new Error('事件顺序应为 tick 升序');
  }

  const late = roundEvents(replay, 620);
  if (late.some((row) => !row.passed)) throw new Error('tick 620 时三条都应已过');
  const firstElapsed = late[0]?.elapsed ?? 0;
  if (Math.abs(firstElapsed - 200 / replay.tickRate) > 1e-6) {
    throw new Error(`首条事件相对秒数应为 ${200 / replay.tickRate}`);
  }
});

Deno.test('panels：回合切换后事件列表随之切换', () => {
  const replay = buildSampleReplay();
  const rows = roundEvents(replay, 700);
  if (rows.length !== 3) throw new Error('第 2 回合应有 3 条事件');
  if (rows[0]?.event.tick !== 840) throw new Error('第 2 回合首条事件应在 840 tick');
});

Deno.test('panels：击杀落点取自死亡瞬间坐标且限定回合', () => {
  const replay = buildSampleReplay();
  const spots = killSpots(replay, 620, 'round');
  if (spots.length !== 3) throw new Error(`第 1 回合应有 3 个落点，实际 ${spots.length}`);
  for (const spot of spots) {
    if (spot.x < 0 || spot.x > 1 || spot.y < 0 || spot.y > 1) throw new Error('落点坐标应归一化在 0..1');
  }
  if (killSpots(replay, 100, 'round').length !== 0) throw new Error('播放头之前不应有落点');
  if (killSpots(replay, 1300, 'all').length !== 6) throw new Error('tick 1300 时全场累计应有 6 个落点');
});

Deno.test('panels：热图网格数量与热度归一化', () => {
  const replay = buildSampleReplay();
  const spots = killSpots(replay, 620, 'round');
  const cols = 20;
  const rows = 12;
  const cells = heatGrid(spots, cols, rows);

  if (cells.length !== cols * rows) throw new Error('格子数应为 列×行');
  const total = cells.reduce((sum, cell) => sum + cell.count, 0);
  if (total !== spots.length) throw new Error('格子计数之和应等于落点数');
  const hottest = cells.reduce((best, cell) => (cell.count > best ? cell.count : best), 0);
  if (hottest <= 0) throw new Error('有落点时最热格应大于 0');
  for (const cell of cells) {
    if (cell.level < 0 || cell.level > 1) throw new Error('热度应归一化到 0..1');
  }
});

Deno.test('panels：热图把坐标贴到正确格子（含边界裁剪）', () => {
  const cells = heatGrid(
    [
      { tick: 1, victim: 1, x: 0, y: 0 },
      { tick: 2, victim: 2, x: 1, y: 1 },
    ],
    4,
    2,
  );

  // (0,0) 是左下角 → 末行首列；(1,1) 是右上角 → 首行末列
  const bottomLeft = cells.find((cell) => cell.row === 1 && cell.col === 0);
  const topRight = cells.find((cell) => cell.row === 0 && cell.col === 3);
  if (!bottomLeft || bottomLeft.count !== 1) throw new Error('左下角落点应落在末行首列');
  if (!topRight || topRight.count !== 1) throw new Error('右上角落点应落在首行末列');

  const empty = heatGrid([], 3, 3);
  if (empty.some((cell) => cell.level !== 0)) throw new Error('无落点时热度应全为 0');
});
