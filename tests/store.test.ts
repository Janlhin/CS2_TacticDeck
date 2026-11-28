import { test } from 'bun:test';


import { useAppStore } from '../src/store/appStore.ts';
import { loadedCount } from '../src/store/rail.ts';
import { buildSampleReplay } from '../src/replay/fixture.ts';
import { totalTicks } from '../src/replay/selectors.ts';

test('store：装载回放后十槽全部绑定，播放头落在首回合起点', () => {
  const replay = buildSampleReplay();
  useAppStore.getState().loadReplay(replay, 'sample');

  const state = useAppStore.getState();
  if (state.replay !== replay) throw new Error('回放未写进 store');
  if (loadedCount(state.views) !== 10) {
    throw new Error('应绑定 10 个选手槽');
  }
  if (state.tick !== (replay.rounds[0]?.startTick ?? 0)) throw new Error('播放头应位于首回合起点');
  if (state.playing) throw new Error('装载后不应自动播放');
});

test('store：暂停态下 advance 不推进播放头', () => {
  const replay = buildSampleReplay();
  useAppStore.getState().loadReplay(replay, 'sample');
  const before = useAppStore.getState().tick;

  useAppStore.getState().advance(1000);
  if (useAppStore.getState().tick !== before) throw new Error('暂停时不应推进');
});

test('store：播放态下 advance 按 tickRate×倍速推进，到尾自动停', () => {
  const replay = buildSampleReplay();
  useAppStore.getState().loadReplay(replay, 'sample');
  useAppStore.getState().togglePlay();
  if (!useAppStore.getState().playing) throw new Error('togglePlay 后应处于播放态');

  const start = useAppStore.getState().tick;
  useAppStore.getState().advance(1000);
  const oneSecond = Math.round(start + replay.tickRate);
  if (Math.abs(useAppStore.getState().tick - oneSecond) > 1) {
    throw new Error(`1 秒应按 64 tick 推进，实际 ${useAppStore.getState().tick - start}`);
  }

  useAppStore.getState().setSpeed(2);
  const mid = useAppStore.getState().tick;
  useAppStore.getState().advance(1000);
  if (Math.abs(useAppStore.getState().tick - (mid + replay.tickRate * 2)) > 1) {
    throw new Error('2× 倍速应推进两倍 tick');
  }

  const end = totalTicks(replay);
  useAppStore.getState().advance(60_000);
  if (useAppStore.getState().tick !== end) throw new Error('越过末尾应停在总 tick');
  if (useAppStore.getState().playing) throw new Error('到尾应自动暂停');
});

test('store：seekTick 夹在合法区间', () => {
  const replay = buildSampleReplay();
  useAppStore.getState().loadReplay(replay, 'sample');

  useAppStore.getState().seekTick(-100);
  if (useAppStore.getState().tick !== 0) throw new Error('负 tick 应夹到 0');

  useAppStore.getState().seekTick(totalTicks(replay) + 500);
  if (useAppStore.getState().tick !== totalTicks(replay)) throw new Error('超出总 tick 应夹到末尾');

  useAppStore.getState().seekTick(620);
  if (useAppStore.getState().tick !== 620) throw new Error('合法 tick 应原样生效');
});
