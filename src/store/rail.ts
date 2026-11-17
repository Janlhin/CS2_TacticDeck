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

export const PLAYER_COUNT = 10;
export const OVERVIEW_ID = 0;
/** 左栏槽位总数：1 个固定 2D 总览 + 10 人 */
export const SLOT_COUNT = PLAYER_COUNT + 1;

export function buildViews(): ViewSlot[] {
  return [
    { id: OVERVIEW_ID, label: '2D 总览', kind: 'overview', team: 'none', source: '' },
    ...Array.from({ length: PLAYER_COUNT }, (_, index): ViewSlot => ({
      id: index + 1,
      label: `P${index + 1}`,
      kind: 'player',
      team: index < PLAYER_COUNT / 2 ? 't' : 'ct',
      source: '',
    })),
  ];
}

/** 只在 player 槽之间轮换；返回新的 activeView */
export function rotateView(views: ViewSlot[], activeView: number, step: number): number {
  const players = views.filter((view) => view.kind === 'player');
  const first = players[0];
  if (!first) return activeView;
  const index = players.findIndex((view) => view.id === activeView);
  const nextIndex = index < 0 ? 0 : (index + step + players.length) % players.length;
  const next = players[nextIndex] ?? first;
  return next.id;
}

export function loadedCount(views: ViewSlot[]): number {
  return views.filter((view) => view.kind === 'player' && view.source !== '').length;
}

/** 载入回放后把十人槽位一并绑定到同一份来源 */
export function bindAllSources(views: ViewSlot[], source: string): ViewSlot[] {
  return views.map((view) => (view.kind === 'player' ? { ...view, source } : view));
}
