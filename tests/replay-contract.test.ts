import { test } from 'bun:test';


import type { Replay } from '../src/replay/types.ts';
import {
  aliveCount,
  frameAt,
  heatGrid,
  killSpots,
  killsUpTo,
  rosterAt,
  roundEvents,
  roundIndexAt,
  seekTick,
  totalTicks,
  validateTenPlayers,
} from '../src/replay/selectors.ts';

const REPLAY_PATH = new URL('../replay.json', import.meta.url);

async function loadReplay(): Promise<Replay | null> {
  try {
    const text = await Deno.readTextFile(REPLAY_PATH);
    return JSON.parse(text) as Replay;
  } catch {
    return null;
  }
}

const replay = await loadReplay();

/**
 * 真实 demo 导出的 replay.json 契约回归。
 * 本地跑过 probe 导出后才生效；没导出时跳过，避免 CI 依赖私有大文件。
 */
(replay === null ? test.skip : test)('replay 契约：probe 导出的 replay.json 满足前端结构', () => {
    if (!replay) throw new Error('未生成 replay.json');
    if (!replay.map) throw new Error('map 为空');
    if (replay.tickRate <= 0) throw new Error('tickRate 非法');
    if (replay.players.length !== 10) throw new Error(`选手应 10 人，实际 ${replay.players.length}`);
    replay.players.forEach((player, index) => {
      if (player.slot !== index + 1) throw new Error(`槽位应连续，第 ${index} 项是 ${player.slot}`);
      if (!player.name) throw new Error(`选手 ${player.slot} 名字为空`);
      if (player.team !== 't' && player.team !== 'ct') throw new Error(`选手 ${player.slot} 阵营非法`);
    });
    if (replay.rounds.length === 0) throw new Error('没有回合');
    if (replay.frames.length === 0) throw new Error('没有帧');

    let lastTick = 0;
    for (const frame of replay.frames) {
      if (frame.tick < lastTick) throw new Error(`帧 tick 未递增：${frame.tick} < ${lastTick}`);
      lastTick = frame.tick;
      if (frame.players.length !== 10) throw new Error(`tick ${frame.tick} 只有 ${frame.players.length} 人`);
      for (const player of frame.players) {
        if (player.x < 0 || player.x > 1 || player.y < 0 || player.y > 1) {
          throw new Error(`tick ${frame.tick} 槽位 ${player.slot} 坐标越界`);
        }
      }
    }

    for (const round of replay.rounds) {
      if (round.startTick > round.endTick) throw new Error(`回合 ${round.index} 起止 tick 反了`);
    }
    for (const event of replay.events) {
      if (event.killer < 1 || event.killer > 10 || event.victim < 1 || event.victim > 10) {
        throw new Error('击杀事件槽位越界');
      }
    }
});

(replay === null ? test.skip : test)('replay 契约：选择器能吃下真实 demo 数据', () => {
    if (!replay) throw new Error('未生成 replay.json');
    const total = totalTicks(replay);
    const middle = Math.floor(total / 2);

    const frame = frameAt(replay, middle);
    if (!frame) throw new Error(`tick ${middle} 取不到帧`);
    if (seekTick(replay, total + 1000)?.tick !== total) throw new Error('seekTick 未 clamp 到末尾');

    if (validateTenPlayers(replay, middle).ok !== true) throw new Error('十人完整性校验失败');
    const roster = rosterAt(replay, middle);
    if (roster.length !== 10) throw new Error(`十人列表应 10 行，实际 ${roster.length}`);
    if (roster.some((row) => row.frame === null)) throw new Error('十人列表存在空帧');

    const roundIndex = roundIndexAt(replay, middle);
    if (roundIndex < 0) throw new Error('取不到当前回合');
    const events = roundEvents(replay, middle);
    if (events.length === 0) throw new Error('本回合没有事件——真实 demo 应有击杀');
    if (events.some((row, index) => index > 0 && row.event.tick < events[index - 1]!.event.tick)) {
      throw new Error('事件应按时间正序');
    }

    const kills = killsUpTo(replay, middle);
    if (kills.length === 0) throw new Error('播放头之前应有击杀');
    if (kills.some((event, index) => index > 0 && event.tick > kills[index - 1]!.tick)) {
      throw new Error('击杀应倒序');
    }

    const spots = killSpots(replay, total, 'all');
    if (spots.length === 0) throw new Error('应能算出击杀落点');
    const cells = heatGrid(spots, 20, 12);
    if (cells.length !== 240) throw new Error(`热图应 240 格，实际 ${cells.length}`);
    if (Math.max(...cells.map((cell) => cell.level)) !== 1) throw new Error('最热格应为 1');

    const alive = aliveCount(replay, middle, 't') + aliveCount(replay, middle, 'ct');
    if (alive > 10) throw new Error('存活人数超过 10');
});
