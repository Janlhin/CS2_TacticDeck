import { Application, Container, Graphics, Sprite, Texture } from 'pixi.js';
import { useEffect, useRef, useState } from 'react';
import { TEAM_COLOR, directionVector, hpColor } from '../replay/geometry.ts';
import { loadRadarImage } from '../replay/radar.ts';
import { frameIndexAt } from '../replay/selectors.ts';
import type { Replay } from '../replay/types.ts';

type Props = {
  replay: Replay;
  tick: number;
  /** 被左栏选中的选手槽位；0 表示 2D 总览，不高亮任何一人 */
  focusSlot: number;
};

const PAD = 14;
const TRAIL_FRAMES = 18;

/**
 * 2D 回放板面：地图底格 + 十人点位 + 朝向 + 血量 + 轨迹。
 * 坐标来自 `Replay` 的归一化值，绘制时翻转 y 轴（数据原点在左下）。
 */
export function ReplayBoard({ replay, tick, focusSlot }: Props) {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const appRef = useRef<Application | null>(null);
  const layerRef = useRef<Container | null>(null);
  const [radar, setRadar] = useState<HTMLImageElement | null>(null);
  // 初始化是异步的，用 ref 保存最新入参，init 完成后补画一帧
  const latestRef = useRef({ replay, tick, focusSlot, radar });
  latestRef.current = { replay, tick, focusSlot, radar };

  // 换地图就换底图；取不到就留在网格上
  useEffect(() => {
    let cancelled = false;
    setRadar(null);
    void loadRadarImage(replay.map).then((url) => {
      if (cancelled || !url) return;
      const image = new Image();
      image.onload = () => {
        if (!cancelled) setRadar(image);
      };
      image.src = url;
    });
    return () => {
      cancelled = true;
    };
  }, [replay.map]);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;

    let disposed = false;
    let ready = false;
    const app = new Application();

    const boot = async () => {
      await app.init({
        backgroundAlpha: 0,
        antialias: true,
        resizeTo: host,
        autoDensity: true,
        resolution: window.devicePixelRatio || 1,
      });

      if (disposed) {
        app.destroy(true);
        return;
      }

      host.appendChild(app.canvas);
      const layer = new Container();
      app.stage.addChild(layer);
      appRef.current = app;
      layerRef.current = layer;
      ready = true;

      // 画布绝对定位由 CSS 控制，这里按宿主真实尺寸显式设定一次渲染尺寸
      const width = Math.max(1, Math.round(host.clientWidth));
      const height = Math.max(1, Math.round(host.clientHeight));
      app.renderer.resize(width, height);

      const { replay: currentReplay, tick: currentTick, focusSlot: currentFocus, radar: currentRadar } =
        latestRef.current;
      paint(app, layer, currentReplay, currentTick, currentFocus, currentRadar);
    };

    void boot();

    return () => {
      disposed = true;
      if (ready) {
        app.destroy(true);
        appRef.current = null;
        layerRef.current = null;
      }
    };
  }, []);

  useEffect(() => {
    const app = appRef.current;
    const layer = layerRef.current;
    if (!app || !layer) return;
    paint(app, layer, replay, tick, focusSlot, radar);
  }, [replay, tick, focusSlot, radar]);

  // 窗口尺寸变化后画布会被拉伸，需要按新尺寸重画
  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;
    const observer = new ResizeObserver(() => {
      const app = appRef.current;
      const layer = layerRef.current;
      if (!app || !layer) return;
      const width = Math.max(1, Math.round(host.clientWidth));
      const height = Math.max(1, Math.round(host.clientHeight));
      app.renderer.resize(width, height);
      const latest = latestRef.current;
      paint(app, layer, latest.replay, latest.tick, latest.focusSlot, latest.radar);
    });
    observer.observe(host);
    return () => observer.disconnect();
  }, []);

  return <div className="replay-board" ref={hostRef} aria-label="2D 回放板面" data-tick={Math.round(tick)} />;
}

function paint(
  app: Application,
  layer: Container,
  replay: Replay,
  tick: number,
  focusSlot: number,
  radar: HTMLImageElement | null,
) {
  layer.removeChildren().forEach((child) => child.destroy());

  const width = app.screen.width;
  const height = app.screen.height;
  if (width < 32 || height < 32) return;

  // 雷达图是正方形：按短边取一个居中的方块，人点与底图共用同一套换算
  const box = Math.min(width, height) - 2 * PAD;
  const left = (width - box) * 0.5;
  const top = (height - box) * 0.5;

  const toPx = (x: number, y: number) => ({
    px: left + x * box,
    py: top + (1 - y) * box,
  });

  const base = new Graphics();
  base.roundRect(0, 0, width, height, 6).fill({ color: 0x0c1117, alpha: 1 });
  layer.addChild(base);

  // 有真实底图就铺底图，没有就退回网格 + 包点占位框
  if (radar) {
    const sprite = new Sprite(Texture.from(radar));
    sprite.x = left;
    sprite.y = top;
    sprite.width = box;
    sprite.height = box;
    layer.addChild(sprite);
  } else {
    const grid = new Graphics();
    const step = Math.max(22, Math.round(box / 14));
    for (let px = left; px <= left + box + 0.5; px += step) {
      grid.moveTo(px, top).lineTo(px, top + box);
    }
    for (let py = top; py <= top + box + 0.5; py += step) {
      grid.moveTo(left, py).lineTo(left + box, py);
    }
    grid.stroke({ width: 1, color: 0x24303f, alpha: 0.55 });
    layer.addChild(grid);

    const sites = new Graphics();
    for (const site of [
      { x: 0.66, y: 0.26, color: 0xd29922 },
      { x: 0.18, y: 0.18, color: 0x4c9aff },
    ]) {
      const a = toPx(site.x, site.y);
      const b = toPx(site.x + 0.16, site.y + 0.14);
      sites.rect(a.px, a.py, b.px - a.px, b.py - a.py).fill({ color: site.color, alpha: 0.07 });
      sites
        .rect(a.px, a.py, b.px - a.px, b.py - a.py)
        .stroke({ width: 1, color: site.color, alpha: 0.35 });
    }
    layer.addChild(sites);
  }

  const border = new Graphics();
  border.roundRect(left, top, box, box, 4).stroke({ width: 1, color: 0x24303f, alpha: 1 });
  layer.addChild(border);

  const index = frameIndexAt(replay, tick);
  if (index < 0) return;
  const frame = replay.frames[index];
  if (!frame) return;
  const teams = new Map(replay.players.map((player) => [player.slot, player.team]));

  // 轨迹：往回取 TRAIL_FRAMES 帧，逐段淡化
  const from = Math.max(0, index - TRAIL_FRAMES);
  for (const player of frame.players) {
    const trail = new Graphics();
    let started = false;
    for (let i = from; i <= index; i += 1) {
      const sampleFrame = replay.frames[i];
      if (!sampleFrame) continue;
      const sample = sampleFrame.players.find((item) => item.slot === player.slot);
      if (!sample || !sample.alive) {
        started = false;
        continue;
      }
      const p = toPx(sample.x, sample.y);
      if (!started) {
        trail.moveTo(p.px, p.py);
        started = true;
      } else {
        trail.lineTo(p.px, p.py);
      }
    }
    trail.stroke({
      width: 1,
      color: TEAM_COLOR[teams.get(player.slot) ?? 't'],
      alpha: 0.4,
    });
    layer.addChild(trail);
  }

  for (const player of frame.players) {
    const team = teams.get(player.slot) ?? 't';
    const p = toPx(player.x, player.y);
    const dot = new Graphics();

    dot.circle(p.px, p.py, player.alive ? 6 : 3.5).fill({
      color: player.alive ? TEAM_COLOR[team] : 0x55606c,
      alpha: player.alive ? 1 : 0.6,
    });

    if (player.alive) {
      const v = directionVector(player.dir);
      const tipX = p.px + v.dx * 13;
      const tipY = p.py + v.dy * 13;
      const perpX = -v.dy * 3.4;
      const perpY = v.dx * 3.4;
      dot
        .poly([p.px, p.py, tipX + perpX, tipY + perpY, tipX - perpX, tipY - perpY])
        .fill({ color: TEAM_COLOR[team], alpha: 0.85 });

      dot
        .rect(p.px - 8, p.py - 13, 16, 3)
        .fill({ color: 0x101820, alpha: 0.9 });
      dot
        .rect(p.px - 8, p.py - 13, 16 * (Math.max(0, Math.min(100, player.hp)) / 100), 3)
        .fill({ color: hpColor(player.hp), alpha: 1 });

      if (player.slot === focusSlot) {
        dot.circle(p.px, p.py, 10).stroke({ width: 1.5, color: 0xffffff, alpha: 0.9 });
      }
    }

    layer.addChild(dot);
  }
}
