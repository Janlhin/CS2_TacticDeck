import { test } from 'bun:test';


import { cachedRadar, clearRadarCache, loadRadarImage, rememberRadar } from '../src/replay/radar.ts';

test('雷达缓存：取过的地址会被记住', () => {
  clearRadarCache();
  rememberRadar('de_mirage', 'data:image/png;base64,AAA');
  if (cachedRadar('de_mirage') !== 'data:image/png;base64,AAA') throw new Error('缓存没记住地址');
  if (cachedRadar('de_dust2') !== null) throw new Error('没取过的地图应是 null');
});

test('雷达缓存：可以记住“没有底图”', () => {
  clearRadarCache();
  rememberRadar('de_nuke', null);
  if (cachedRadar('de_nuke') !== null) throw new Error('空结果应是 null');
});

test('浏览器预览没有 Rust 桥，取底图返回 null', async () => {
  clearRadarCache();
  const url = await loadRadarImage('de_mirage');
  if (url !== null) throw new Error('浏览器里不该拿到底图');
});
