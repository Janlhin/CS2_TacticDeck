import { create } from 'zustand';

export type ViewKind = 'overview' | 'player';
export type Team = 'none' | 't' | 'ct';

export type ViewSlot = {
  id: number;
  label: string;
  kind: ViewKind;
  team: Team;
  /** 已载入的 demo 路径；空串表示该路尚未绑定 */
  source: string;
};

const PLAYER_COUNT = 10;

export const OVERVIEW_ID = 0;

const initialViews: ViewSlot[] = [
  { id: OVERVIEW_ID, label: '2D 总览', kind: 'overview', team: 'none', source: '' },
  ...Array.from({ length: PLAYER_COUNT }, (_, index): ViewSlot => ({
    id: index + 1,
    label: `P${index + 1}`,
    kind: 'player',
    team: index < PLAYER_COUNT / 2 ? 't' : 'ct',
    source: '',
  })),
];

type AppState = {
  views: ViewSlot[];
  activeView: number;
  bindSource: (id: number, source: string) => void;
  setActiveView: (id: number) => void;
  rotate: (step: number) => void;
  loadedCount: () => number;
};

export const useAppStore = create<AppState>((set, get) => ({
  views: initialViews,
  activeView: OVERVIEW_ID,
  bindSource: (id, source) =>
    set((state) => ({
      views: state.views.map((view) => (view.id === id ? { ...view, source } : view)),
    })),
  setActiveView: (id) => set({ activeView: id }),
  rotate: (step) => {
    const views = get().views.filter((view) => view.kind === 'player');
    if (views.length === 0) return;
    const current = get().activeView;
    const index = views.findIndex((view) => view.id === current);
    const next = index < 0 ? views[0] : views[(index + step + views.length) % views.length];
    if (next) set({ activeView: next.id });
  },
  loadedCount: () =>
    get().views.filter((view) => view.kind === 'player' && view.source !== '').length,
}));
