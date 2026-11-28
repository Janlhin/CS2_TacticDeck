import { test } from 'bun:test';

import {
  buildTrackIndex,
  createStaticSource,
  mountedVideoSlots,
  parseTrackManifest,
  selectTrack,
  sliceTracksByRange,
  visibleCoverRange,
  type TrackManifest,
} from '../src/replay/tracks.ts';
import { useAppStore } from '../src/store/appStore.ts';

/** 构造一份假清单：路径字符串均为占位，不指向仓库内任何真实封面或视频文件 */
function buildFakeManifest(count = 10, tickRate = 64): TrackManifest {
  return {
    roundIndex: 0,
    tickRate,
    tracks: Array.from({ length: count }, (_, index) => {
      const slot = index + 1;
      return {
        slot,
        cover: `covers/p${slot}.webp`,
        video: `tracks/p${slot}.mp4`,
        startTick: 0,
        endTick: 640,
      };
    }),
  };
}

function expectReject(raw: unknown, snippet: string): void {
  try {
    parseTrackManifest(raw);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    if (!message.includes(snippet)) {
      throw new Error(`错误信息应包含「${snippet}」，实际：${message}`);
    }
    return;
  }
  throw new Error(`应拒绝非法清单：${snippet}`);
}

test('tracks：合法 10 路清单解析通过，索引可 O(1) 查表', () => {
  const manifest = parseTrackManifest(buildFakeManifest(10));
  if (manifest.tracks.length !== 10) throw new Error('应解析出 10 路');

  const index = buildTrackIndex(manifest);
  if (index.size !== 10) throw new Error('索引应含 10 个槽位');
  const track = selectTrack(index, 7);
  if (!track || track.slot !== 7) throw new Error('P7 应能从索引取到');
  if (selectTrack(index, 42) !== undefined) throw new Error('越界槽位应查不到轨');
});

test('tracks：slot 重复被拒', () => {
  const raw = {
    roundIndex: 0,
    tickRate: 64,
    tracks: [
      { slot: 1, cover: 'a.webp', video: 'a.mp4', startTick: 0, endTick: 100 },
      { slot: 1, cover: 'b.webp', video: 'b.mp4', startTick: 0, endTick: 100 },
    ],
  };
  expectReject(raw, '重复');
});

test('tracks：slot 越界（0 与 11）被拒', () => {
  expectReject({ ...buildFakeManifest(1), tracks: [{ slot: 0, cover: 'a.webp', video: 'a.mp4', startTick: 0, endTick: 100 }] }, '1..10');
  expectReject({ ...buildFakeManifest(1), tracks: [{ slot: 11, cover: 'a.webp', video: 'a.mp4', startTick: 0, endTick: 100 }] }, '1..10');
});

test('tracks：N=0 与 N=11 被拒（N 只允许 1..10）', () => {
  expectReject(buildFakeManifest(0), '至少需要 1 路');
  expectReject(buildFakeManifest(11), '最多 10 路');
});

test('tracks：tickRate 非正、起止反序、路径为空均被拒', () => {
  expectReject(buildFakeManifest(2, 0), '正整数');
  const reversed = {
    roundIndex: 0,
    tickRate: 64,
    tracks: [{ slot: 1, cover: 'a.webp', video: 'a.mp4', startTick: 200, endTick: 100 }],
  };
  expectReject(reversed, '起止 tick 反了');
  expectReject(
    { roundIndex: 0, tickRate: 64, tracks: [{ slot: 1, cover: '', video: 'a.mp4', startTick: 0, endTick: 100 }] },
    '封面路径为空',
  );
  expectReject(
    { roundIndex: 0, tickRate: 64, tracks: [{ slot: 1, cover: 'a.webp', video: '', startTick: 0, endTick: 100 }] },
    '视频路径为空',
  );
});

test('tracks：静态来源直接返回同一份清单', async () => {
  const manifest = buildFakeManifest(3);
  const source = createStaticSource(manifest);
  const loaded = await source.list(0);
  if (loaded !== manifest) throw new Error('静态来源应原样返回注入的清单');
});

test('懒加载：小视口只请求可见区间内的封面，封面数少于 N', () => {
  const count = 10;
  // 每行 90px，视口高 180px → 首屏只放得下 2 行；无 overscan
  const range = visibleCoverRange(0, 90, 180, count, 0);
  const requested = range.end - range.start + 1;
  if (range.start !== 0 || range.end !== 1) throw new Error(`首屏区间应为 0..1，实际 ${range.start}..${range.end}`);
  if (requested >= count) throw new Error(`小视口请求封面数 ${requested} 应少于 N=${count}`);

  // 大视口应覆盖全部封面
  const full = visibleCoverRange(0, 90, 900, count, 0);
  if (full.start !== 0 || full.end !== count - 1) throw new Error('大视口应覆盖全部封面');

  // 空清单/非法尺寸返回空区间（end=-1）
  const empty = visibleCoverRange(0, 90, 180, 0, 0);
  if (empty.end !== -1) throw new Error('无清单时不应请求任何封面');
});

test('懒加载：切片子集路数少于 N，首尾下标与可见区间一致', () => {
  const manifest = buildFakeManifest(10);
  // 单项宽 120px，视口宽 240px → 首屏 2 项，加 1 项 overscan → 区间 0..2
  const range = visibleCoverRange(0, 120, 240, manifest.tracks.length, 1);
  const visible = sliceTracksByRange(manifest.tracks, range);

  if (visible.length >= manifest.tracks.length) {
    throw new Error(`小视口渲染封面数 ${visible.length} 应少于 N=${manifest.tracks.length}`);
  }
  if (visible.length !== range.end - range.start + 1) throw new Error('切片长度应与区间闭合长度一致');
  if (visible[0]?.slot !== manifest.tracks[range.start]?.slot) throw new Error('切片首项下标应与区间起点一致');
  if (visible[visible.length - 1]?.slot !== manifest.tracks[range.end]?.slot) throw new Error('切片末项下标应与区间终点一致');
});

test('懒加载：空区间（end=-1）切片返回 0 路且不崩', () => {
  const manifest = buildFakeManifest(10);
  const emptyRange = visibleCoverRange(0, 120, 240, 0, 1);
  const visible = sliceTracksByRange(manifest.tracks, emptyRange);
  if (visible.length !== 0) throw new Error('空区间不应返回任何封面');
  if (sliceTracksByRange(manifest.tracks, { start: 0, end: -1 }).length !== 0) {
    throw new Error('显式空区间应返回 0 路');
  }
});

test('挂载：只挂主视图一路视频；切主后仍为 1 路且换到新 slot', () => {
  const index = buildTrackIndex(buildFakeManifest(10));

  const first = mountedVideoSlots(index, 1);
  if (first.length !== 1 || first[0] !== 1) throw new Error('初始应只挂载 P1 一路视频');

  const afterSwitch = mountedVideoSlots(index, 6);
  if (afterSwitch.length !== 1 || afterSwitch[0] !== 6) throw new Error('切主后应只挂载 P6 一路视频');

  // 未注入清单（空索引）、overview 主槽（0）、越界槽位都不挂视频
  if (mountedVideoSlots(new Map(), 1).length !== 0) throw new Error('空索引不应挂载视频');
  if (mountedVideoSlots(index, 0).length !== 0) throw new Error('总览槽位不应挂载视频');
  if (mountedVideoSlots(index, 11).length !== 0) throw new Error('越界槽位不应挂载视频');
});

test('切主耗时：选择 + store 切主 + 取选择器，100 次单次均 < 1000 ms', () => {
  const manifest = buildFakeManifest(10);
  useAppStore.getState().setTrackManifest(manifest);
  const index = useAppStore.getState().trackIndex;

  const ITERATIONS = 100;
  let maxCost = 0;

  for (let i = 0; i < ITERATIONS; i += 1) {
    const slot = (i % 10) + 1;

    const started = performance.now();
    const picked = selectTrack(index, slot);
    useAppStore.getState().setActiveView(slot);
    const active = useAppStore.getState().activeTrack();
    const ended = performance.now();

    if (!picked || picked.slot !== slot) throw new Error(`纯查表应取到 P${slot}`);
    if (!active || active.slot !== slot) throw new Error(`切主后选择器应返回 P${slot}`);

    const cost = ended - started;
    if (cost > maxCost) maxCost = cost;
    if (cost >= 1000) throw new Error(`单次切主 ${cost.toFixed(3)} ms 达到或超过 1000 ms`);
  }

  // 切主路径只做 Map 查表 + zustand 置位，不重解析清单、不重建索引；这里打印实测最大值供人工对账
  console.log(`切主耗时最大值（${ITERATIONS} 次，单位 ms）：${maxCost.toFixed(3)}`);
  if (useAppStore.getState().trackIndex.size !== 10) throw new Error('切主过程中索引不应被重建或清空');
});

test('空态：未注入清单时纯函数不崩', () => {
  expectReject(null, '对象');
  expectReject(undefined, '对象');

  const emptyIndex = buildTrackIndex(parseTrackManifest(buildFakeManifest(1)));
  if (selectTrack(emptyIndex, 99) !== undefined) throw new Error('查不到的槽位应返回 undefined');
  if (mountedVideoSlots(emptyIndex, 99).length !== 0) throw new Error('越界槽位不应挂载视频');

  const range = visibleCoverRange(0, 90, 180, 0, 2);
  if (range.end !== -1) throw new Error('空清单封面区间应为空');
});
