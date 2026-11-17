/**
 * 雷达底图：向 Rust 侧要一张 data URL，并在前端缓存。
 * 底图是从本机 CS2 的 VPK 里现取的，不入库、不随安装包分发。
 */

import { isTauriRuntime } from './library.ts';

const cache = new Map<string, string | null>();

/** 底图数据 URL 的缓存，测试里可以直接塞 */
export function cachedRadar(map: string): string | null {
  return cache.get(map) ?? null;
}

export function rememberRadar(map: string, url: string | null): void {
  cache.set(map, url);
}

export function clearRadarCache(): void {
  cache.clear();
}

async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: call } = await import('@tauri-apps/api/core');
  return await call<T>(command, args);
}

/**
 * 取某张地图的雷达底图；取不到就返回 `null`，界面退回网格。
 * 浏览器预览没有 Rust 桥，同样返回 `null`。
 */
export async function loadRadarImage(map: string): Promise<string | null> {
  const hit = cache.get(map);
  if (hit !== undefined) return hit;
  if (!isTauriRuntime()) {
    return null;
  }
  try {
    const url = await invoke<string>('radar_image', { map });
    rememberRadar(map, url);
    return url;
  } catch {
    rememberRadar(map, null);
    return null;
  }
}
