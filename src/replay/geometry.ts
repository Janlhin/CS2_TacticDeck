import type { TeamSide } from './types.ts';

/** 队伍主色：T 橙、CT 蓝（与左栏徽标同一套色） */
export const TEAM_COLOR: Record<TeamSide, number> = {
  t: 0xd29922,
  ct: 0x4c9aff,
};

export const TEAM_COLOR_HEX: Record<TeamSide, string> = {
  t: '#d29922',
  ct: '#4c9aff',
};

export const HP_COLOR = {
  healthy: 0x3fb950,
  hurt: 0xd29922,
  critical: 0xf85149,
} as const;

export function clamp01(value: number): number {
  if (Number.isNaN(value)) return 0;
  return value < 0 ? 0 : value > 1 ? 1 : value;
}

/** 方向角（度，逆时针为正）→ 单位向量，y 轴向下为正，与画布一致 */
export function directionVector(deg: number): { dx: number; dy: number } {
  const rad = (deg * Math.PI) / 180;
  return { dx: Math.cos(rad), dy: -Math.sin(rad) };
}

export function hpColor(hp: number): number {
  if (hp > 50) return HP_COLOR.healthy;
  if (hp > 20) return HP_COLOR.hurt;
  return HP_COLOR.critical;
}
