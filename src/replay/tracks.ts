/**
 * 录像轨清单（T-07 视频条）：可注入的数据源抽象与配套纯逻辑。
 *
 * 设计边界：本轮只到可视化与逻辑层，不加载任何真实媒体。`cover` / `video` 只是路径或
 * URL 字符串，真实封面与录像素材的产出依赖 Q-01（本地 HLAE vs 服务端）与 Q-06（POV 单人
 * 契约）未决，详见 `docs/video-track-manifest.md` 的「当前限制」。
 *
 * 时间口径沿用 `src/replay/types.ts` 的单一时基：一切位置都用整数 tick，`tickRate` 只做
 * 参考，本模块不出现第二套「秒」换算。
 */

export const MIN_SLOT = 1;
export const MAX_SLOT = 10;

/** 单路录像轨：与花名册同一套 1..10 槽位编号 */
export type RecordingTrack = {
  /** 选手槽位 1..10，与 `PlayerMeta.slot` 同源 */
  slot: number;
  /** 封面图路径或 URL；本轮为假清单字符串，仓库内不落地真实封面素材 */
  cover: string;
  /** 录像文件路径或 URL；本轮不挂载真实视频流 */
  video: string;
  /** 该路在共享 tick 时间基上的起点（整数 tick） */
  startTick: number;
  /** 该路在共享 tick 时间基上的终点（整数 tick，含端点） */
  endTick: number;
};

/** 一回合对应的录像轨清单；N 可配 1..10，允许少于 10 路 */
export type TrackManifest = {
  /** 回合序号（0 起），与 `RoundInfo.index` 对齐 */
  roundIndex: number;
  /** 服务器 tickrate（tick/秒）；口径沿用回放元数据，不在本模块另造换算 */
  tickRate: number;
  tracks: RecordingTrack[];
};

/**
 * 可注入的清单来源。未来由 Rust 桥 / 本地文件实现；浏览器预览与单测用 `createStaticSource`
 * 直接喂一份内存清单，这样组件与 store 不绑定具体取数通道。
 */
export interface TrackSource {
  list(roundIndex: number): Promise<TrackManifest>;
}

/** 假清单来源：直接返回已构造好的清单，供浏览器预览与无 DOM 单测使用 */
export function createStaticSource(manifest: TrackManifest): TrackSource {
  return {
    list: async () => manifest,
  };
}

function isIntegerNumber(value: unknown): value is number {
  return typeof value === 'number' && Number.isInteger(value);
}

/**
 * 把外部 JSON / 未知结构校验成 `TrackManifest`。
 * 校验失败一律抛出带定位信息的可读错误，不静默吞成默认值——清单错了要在注入点就暴露。
 */
export function parseTrackManifest(raw: unknown): TrackManifest {
  if (typeof raw !== 'object' || raw === null) {
    throw new Error('录像轨清单必须是对象');
  }
  const root = raw as Record<string, unknown>;

  const roundIndex = root.roundIndex;
  if (!isIntegerNumber(roundIndex) || roundIndex < 0) {
    throw new Error('清单 roundIndex 必须是非负整数');
  }

  const tickRate = root.tickRate;
  if (!isIntegerNumber(tickRate) || tickRate <= 0) {
    throw new Error('清单 tickRate 必须是正整数');
  }

  const tracks = root.tracks;
  if (!Array.isArray(tracks)) {
    throw new Error('清单 tracks 必须是数组');
  }
  if (tracks.length === 0) {
    throw new Error('清单至少需要 1 路录像轨');
  }
  if (tracks.length > MAX_SLOT) {
    throw new Error(`录像轨最多 ${MAX_SLOT} 路，实际 ${tracks.length}`);
  }

  const seen = new Set<number>();
  const parsed: RecordingTrack[] = tracks.map((rawTrack, index) => {
    if (typeof rawTrack !== 'object' || rawTrack === null) {
      throw new Error(`第 ${index + 1} 路录像轨不是对象`);
    }
    const track = rawTrack as Record<string, unknown>;

    const slot = track.slot;
    if (!isIntegerNumber(slot) || slot < MIN_SLOT || slot > MAX_SLOT) {
      throw new Error(`第 ${index + 1} 路 slot 必须在 ${MIN_SLOT}..${MAX_SLOT}，实际 ${String(slot)}`);
    }
    if (seen.has(slot)) {
      throw new Error(`录像轨 slot 重复：P${slot}`);
    }
    seen.add(slot);

    const cover = track.cover;
    if (typeof cover !== 'string' || cover === '') {
      throw new Error(`P${slot} 封面路径为空`);
    }
    const video = track.video;
    if (typeof video !== 'string' || video === '') {
      throw new Error(`P${slot} 视频路径为空`);
    }

    const startTick = track.startTick;
    const endTick = track.endTick;
    if (!isIntegerNumber(startTick)) {
      throw new Error(`P${slot} startTick 必须是整数 tick`);
    }
    if (!isIntegerNumber(endTick)) {
      throw new Error(`P${slot} endTick 必须是整数 tick`);
    }
    if (startTick >= endTick) {
      throw new Error(`P${slot} 起止 tick 反了：${startTick} 不小于 ${endTick}`);
    }

    return { slot, cover, video, startTick, endTick };
  });

  return { roundIndex, tickRate, tracks: parsed };
}

/** slot → 录像轨 的 O(1) 索引；只在清单注入时构建一次，切主路径直接查表，不重解析、不重排 */
export type TrackIndex = Map<number, RecordingTrack>;

export function buildTrackIndex(manifest: TrackManifest): TrackIndex {
  const index: TrackIndex = new Map();
  for (const track of manifest.tracks) {
    index.set(track.slot, track);
  }
  return index;
}

/** 封面懒加载区间（数组下标闭区间）；`end = -1` 表示当前无需请求任何封面 */
export type CoverRange = { start: number; end: number };

/**
 * 按滚动位置算出本轮应请求封面的下标区间（含上下 overscan 缓冲行）。
 * 纯函数：滚动容器把 `scrollTop / viewportHeight` 传进来即可，组件本身不读 DOM，便于单测。
 */
export function visibleCoverRange(
  scrollTop: number,
  itemHeight: number,
  viewportHeight: number,
  count: number,
  overscan: number,
): CoverRange {
  if (count <= 0 || itemHeight <= 0 || viewportHeight <= 0) {
    return { start: 0, end: -1 };
  }
  const first = Math.floor(scrollTop / itemHeight);
  const last = Math.ceil((scrollTop + viewportHeight) / itemHeight) - 1;
  const start = Math.max(0, first - overscan);
  const end = Math.min(count - 1, last + overscan);
  if (start > end) {
    return { start: 0, end: -1 };
  }
  return { start, end };
}

/**
 * 按封面区间从轨列表切出本帧应渲染的子集。区间为数组下标闭区间；空区间（`end = -1`）
 * 或越界时返回空数组。组件只渲染这里返回的路数，区间外的封面不挂载 DOM。
 */
export function sliceTracksByRange<T extends { slot: number }>(tracks: T[], range: CoverRange): T[] {
  if (range.end < 0 || tracks.length === 0) return [];
  const start = Math.max(0, range.start);
  const end = Math.min(tracks.length - 1, range.end);
  if (start > end) return [];
  return tracks.slice(start, end + 1);
}

/**
 * 本轮应挂载 `<video>` 的 slot 列表：只允许主视图那一路，最多 1 个。
 * 未注入清单或主槽不在清单里时返回空数组，组件据此决定是否渲染视频元素。
 */
export function mountedVideoSlots(index: TrackIndex, activeSlot: number): number[] {
  if (!isIntegerNumber(activeSlot) || activeSlot < MIN_SLOT || activeSlot > MAX_SLOT) {
    return [];
  }
  return index.has(activeSlot) ? [activeSlot] : [];
}

/** 按 slot 从索引取轨；查不到返回 `undefined`。纯查表，不在这里读 `performance.now()` 做业务计时 */
export function selectTrack(index: TrackIndex, slot: number): RecordingTrack | undefined {
  return index.get(slot);
}
