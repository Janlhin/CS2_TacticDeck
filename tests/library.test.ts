/// <reference lib="deno.ns" />

import { useAppStore } from '../src/store/appStore.ts';
import { buildViews, loadedCount, PLAYER_COUNT, rotateView, SLOT_COUNT } from '../src/store/rail.ts';
import { buildSampleReplay } from '../src/replay/fixture.ts';
import {
  entryLabel,
  firstReplay,
  formatBytes,
  isUsableEntry,
  parseReplayPayload,
  type LibraryEntry,
} from '../src/replay/library.ts';

function entry(overrides: Partial<LibraryEntry> = {}): LibraryEntry {
  return {
    path: 'D:/demos/replay.json',
    name: 'replay.json',
    kind: 'replay',
    map: 'de_mirage',
    rounds: 10,
    ticks: 56705,
    bytes: 5_273_600,
    ...overrides,
  };
}

Deno.test('rail：左栏是 1 个固定 2D 总览 + 10 人，共 11 槽', () => {
  const views = buildViews();
  if (views.length !== SLOT_COUNT) throw new Error(`槽位总数应为 ${SLOT_COUNT}`);
  if (views.filter((view) => view.kind === 'overview').length !== 1) throw new Error('只应有一个总览槽');
  if (views.filter((view) => view.kind === 'player').length !== PLAYER_COUNT) throw new Error('应有十个人槽');
  const teams = views.filter((view) => view.kind === 'player').map((view) => view.team);
  if (teams.filter((team) => team === 't').length !== 5) throw new Error('T 侧应 5 人');
  if (teams.filter((team) => team === 'ct').length !== 5) throw new Error('CT 侧应 5 人');
});

Deno.test('rail：轮换只走十个人槽并绕回，不动总览', () => {
  const views = buildViews();
  const firstPlayer = 1;
  let active = firstPlayer;
  for (let step = 0; step < PLAYER_COUNT; step += 1) {
    active = rotateView(views, active, 1);
    if (views[active]?.kind !== 'player') throw new Error('轮换落到了非人槽');
  }
  if (active !== firstPlayer) throw new Error('转一圈应回到起点');
  const back = rotateView(views, firstPlayer, -1);
  if (back !== PLAYER_COUNT) throw new Error(`反向应绕到 P${PLAYER_COUNT}，实际 ${back}`);
});

Deno.test('library：只把「能直接载入的回放」算作可用条目', () => {
  const replay = entry();
  const demo = entry({ name: 'test_demo.dem', kind: 'demo', map: null, rounds: null, ticks: null });
  if (!isUsableEntry(replay)) throw new Error('replay 条目应可用');
  if (isUsableEntry(demo)) throw new Error('未导出的 .dem 不应算可用');
  const list = [demo, replay];
  const first = firstReplay(list);
  if (!first || first.path !== replay.path) throw new Error('应跳过 demo 选到 replay');
  if (firstReplay([demo]) !== null) throw new Error('没有可用回放时应返回 null');
});

Deno.test('library：条目展示拼地图 / 回合数 / 体积', () => {
  if (entryLabel(entry()) !== 'replay.json · de_mirage · R10 · 5.0 MB') {
    throw new Error(`展示文案不对：${entryLabel(entry())}`);
  }
  if (formatBytes(1023) !== '1023 B') throw new Error('小于 1 KB 应显示 B');
  if (formatBytes(2048) !== '2 KB') throw new Error('KB 段应按整数显示');
});

Deno.test('library：真实 replay.json 字段能通过校验', () => {
  const payload = JSON.parse(JSON.stringify(buildSampleReplay())) as Record<string, unknown>;
  payload.map = 'de_mirage';
  payload.tickRate = 64;
  const replay = parseReplayPayload(payload);
  if (replay.map !== 'de_mirage' || replay.tickRate !== 64) throw new Error('正常回放被改坏');

  const bad = [
    { ...payload, frames: [] },
    { ...payload, rounds: [] },
    { ...payload, players: [] },
    { ...payload, tickRate: 0 },
    { ...payload, map: '' },
  ];
  for (const item of bad) {
    let rejected = false;
    try {
      parseReplayPayload(item);
    } catch {
      rejected = true;
    }
    if (!rejected) throw new Error(`这份回放应被拦下：${JSON.stringify(item).slice(0, 60)}`);
  }

  const dup = JSON.parse(JSON.stringify(payload)) as { players: { slot: number }[] };
  if (dup.players.length > 1) {
    dup.players[1]!.slot = dup.players[0]!.slot;
    let rejected = false;
    try {
      parseReplayPayload(dup);
    } catch {
      rejected = true;
    }
    if (!rejected) throw new Error('槽位重复应被拦下');
  }
});

Deno.test('library：载入真实来源后十槽绑定到该路径', () => {
  const replay = buildSampleReplay();
  const source = 'D:/demos/de_mirage.json';
  useAppStore.getState().loadReplay(replay, source);

  const state = useAppStore.getState();
  const players = state.views.filter((view) => view.kind === 'player');
  if (players.length !== PLAYER_COUNT) throw new Error('应有十个人槽');
  if (players.some((view) => view.source !== source)) throw new Error('十槽都应绑定同一来源');
  if (loadedCount(state.views) !== PLAYER_COUNT) throw new Error(`绑定计数应为 ${PLAYER_COUNT}`);
  if (state.views[0]?.source !== '') throw new Error('总览槽不应记来源');
});
