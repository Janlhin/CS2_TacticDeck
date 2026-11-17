import { create } from 'zustand';
import type { Replay } from '../replay/types.ts';
import { totalTicks } from '../replay/selectors.ts';
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
};

export const useAppStore = create<AppState>((set, get) => ({
  views: buildViews(),
  activeView: OVERVIEW_ID,
  replay: null,
  tick: 0,
  playing: false,
  speed: 1,

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
}));
