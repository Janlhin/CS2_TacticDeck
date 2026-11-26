import { create } from 'zustand';
import type { Replay } from '../replay/types.ts';
import { totalTicks } from '../replay/selectors.ts';
import {
  buildTrackIndex,
  parseTrackManifest,
  selectTrack,
  type RecordingTrack,
  type TrackIndex,
  type TrackManifest,
  type TrackSource,
} from '../replay/tracks.ts';
import {
  OVERVIEW_ID,
  buildViews,
  bindAllSources,
  loadedCount as countLoaded,
  rotateView,
  type ViewSlot,
} from './rail.ts';

type AppState = {
  views: ViewSlot[];
  activeView: number;
  replay: Replay | null;
  /** 当前播放头（tick） */
  tick: number;
  playing: boolean;
  /** 播放倍率：0.5 / 1 / 2 / 4 */
  speed: number;
  bindSource: (id: number, source: string) => void;
  setActiveView: (id: number) => void;
  rotate: (step: number) => void;
  loadedCount: () => number;
  loadReplay: (replay: Replay, source: string) => void;
  seekTick: (tick: number) => void;
  advance: (deltaMs: number) => void;
  togglePlay: () => void;
  setSpeed: (speed: number) => void;
  /** 当前回合已注入的录像轨清单；未注入为 null */
  trackManifest: TrackManifest | null;
  /** slot → 轨 的索引，清单注入时建一次；切主只查表，不重建 */
  trackIndex: TrackIndex;
  /** 可注入的清单来源；null 表示尚未接线（本轮测试用静态源喂假清单） */
  trackSource: TrackSource | null;
  setTrackSource: (source: TrackSource | null) => void;
  setTrackManifest: (manifest: TrackManifest) => void;
  /** 按回合序号从已注入来源拉取清单，校验后落库 */
  fetchTracks: (roundIndex: number) => Promise<void>;
  /** 当前主视图对应的录像轨；主槽不在清单里时为 undefined */
  activeTrack: () => RecordingTrack | undefined;
};

export const useAppStore = create<AppState>((set, get) => ({
  views: buildViews(),
  activeView: OVERVIEW_ID,
  replay: null,
  tick: 0,
  playing: false,
  speed: 1,
  trackManifest: null,
  trackIndex: new Map(),
  trackSource: null,

  bindSource: (id, source) =>
    set((state) => ({
      views: state.views.map((view) => (view.id === id ? { ...view, source } : view)),
    })),

  setActiveView: (id) => set({ activeView: id }),

  rotate: (step) => set((state) => ({ activeView: rotateView(state.views, state.activeView, step) })),

  loadedCount: () => countLoaded(get().views),

  loadReplay: (replay, source) =>
    set((state) => ({
      replay,
      tick: replay.rounds[0]?.startTick ?? 0,
      playing: false,
      views: bindAllSources(state.views, source),
    })),

  seekTick: (tick) => {
    const replay = get().replay;
    if (!replay) return;
    set({ tick: Math.min(Math.max(0, Math.round(tick)), totalTicks(replay)) });
  },

  advance: (deltaMs) => {
    const { replay, tick, playing, speed } = get();
    if (!replay || !playing) return;
    const next = tick + (deltaMs / 1000) * replay.tickRate * speed;
    const end = totalTicks(replay);
    if (next >= end) {
      set({ tick: end, playing: false });
      return;
    }
    set({ tick: next });
  },

  togglePlay: () => set((state) => ({ playing: !state.playing })),

  setSpeed: (speed) => set({ speed }),

  setTrackSource: (source) => set({ trackSource: source }),

  // 索引在清单注入这一刻构建一次；切主路径只做 Map 查表，不再重解析或重排清单
  setTrackManifest: (manifest) =>
    set({ trackManifest: manifest, trackIndex: buildTrackIndex(manifest) }),

  fetchTracks: async (roundIndex) => {
    const source = get().trackSource;
    if (!source) return;
    const raw = await source.list(roundIndex);
    get().setTrackManifest(parseTrackManifest(raw));
  },

  activeTrack: () => selectTrack(get().trackIndex, get().activeView),
}));
