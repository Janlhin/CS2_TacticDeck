/**
 * 本机 demo 库：从 Rust 侧取候选列表、读回放，并在浏览器预览时优雅降级。
 * 校验逻辑保持纯函数，方便 `bun run test:web` 直接测。
 */

import type { Replay } from './types.ts';

export type LibraryEntry = {
  path: string;
  name: string;
  kind: 'demo' | 'replay';
  map: string | null;
  rounds: number | null;
  ticks: number | null;
  bytes: number;
};

export function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: call } = await import('@tauri-apps/api/core');
  return await call<T>(command, args);
}

/** 拉取本机 demo 库；浏览器预览下返回空列表而不报错 */
export async function listLibrary(dir?: string): Promise<LibraryEntry[]> {
  if (!isTauriRuntime()) return [];
  const entries = await invoke<LibraryEntry[]>('list_replays', dir ? { dir } : {});
  return entries.filter(isUsableEntry);
}

/** 只保留能直接载入的回放；`.dem` 需要先导出成 replay.json */
export function isUsableEntry(entry: LibraryEntry): boolean {
  return entry.kind === 'replay' && entry.path !== '' && entry.name !== '';
}

/** 取第一条可载入的回放，供启动时自动落位 */
export function firstReplay(entries: LibraryEntry[]): LibraryEntry | null {
  return entries.find(isUsableEntry) ?? null;
}

export function entryLabel(entry: LibraryEntry): string {
  const parts = [entry.name];
  if (entry.map) parts.push(entry.map);
  if (entry.rounds !== null) parts.push(`R${entry.rounds}`);
  if (entry.bytes > 0) parts.push(formatBytes(entry.bytes));
  return parts.join(' · ');
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const kb = bytes / 1024;
  if (kb < 1024) return `${kb.toFixed(0)} KB`;
  return `${(kb / 1024).toFixed(1)} MB`;
}

/** 读一份 replay.json 并校验字段 */
export async function readReplay(path: string): Promise<Replay> {
  if (!isTauriRuntime()) throw new Error('浏览器预览下没有 Rust 桥，读不到本机文件');
  const payload = await invoke<unknown>('load_replay', { path });
  return parseReplayPayload(payload);
}

/**
 * 把外部 JSON 校验成 `Replay`。
 * 只挡住会让回放出错的硬伤：缺帧、缺回合、tickRate 不合法、选手槽位重复。
 */
export function parseReplayPayload(value: unknown): Replay {
  if (typeof value !== 'object' || value === null) {
    throw new Error('回放不是对象');
  }
  const raw = value as Record<string, unknown>;
  const map = raw.map;
  if (typeof map !== 'string' || map === '') throw new Error('回放缺少地图名');
  const tickRate = raw.tickRate;
  if (typeof tickRate !== 'number' || !Number.isFinite(tickRate) || tickRate <= 0) {
    throw new Error('回放的 tickRate 不合法');
  }
  const frames = raw.frames;
  if (!Array.isArray(frames) || frames.length === 0) throw new Error('回放没有帧');
  const rounds = raw.rounds;
  if (!Array.isArray(rounds) || rounds.length === 0) throw new Error('回放没有回合');
  const players = raw.players;
  if (!Array.isArray(players) || players.length === 0) throw new Error('回放没有选手');

  const slots = new Set<number>();
  for (const player of players) {
    const slot = (player as { slot?: unknown }).slot;
    if (typeof slot !== 'number' || !Number.isInteger(slot) || slot < 1) {
      throw new Error('选手槽位必须是正整数');
    }
    if (slots.has(slot)) throw new Error(`选手槽位重复：P${slot}`);
    slots.add(slot);
  }

  return value as Replay;
}
