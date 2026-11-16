import { Application, Graphics } from 'pixi.js';
import { useEffect, useRef } from 'react';

/**
 * 雷达盘面占位：只证明 PixiJS 画布能挂载、能画网格。
 * 真实盘面要等地图校准（calibration）数据落地后再接。
 */
export function RadarCanvas() {
  const hostRef = useRef<HTMLDivElement | null>(null);

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

      // StrictMode 下 effect 可能在 init 完成前就被清理，必须在此处补一次销毁
      if (disposed) {
        app.destroy(true);
        return;
      }

      host.appendChild(app.canvas);
      ready = true;
      drawGrid(app, host.clientWidth, host.clientHeight);
    };

    void boot();

    return () => {
      disposed = true;
      // init 未完成的场景交给 boot() 里的 disposed 分支处理，这里只销毁已就绪的实例
      if (ready) app.destroy(true);
    };
  }, []);

  return <div className="radar" ref={hostRef} aria-label="盘面预览" />;
}

function drawGrid(app: Application, width: number, height: number) {
  const grid = new Graphics();
  const step = 32;
  const color = 0x2a3a4d;

  for (let x = 0; x <= width; x += step) {
    grid.moveTo(x, 0).lineTo(x, height);
  }
  for (let y = 0; y <= height; y += step) {
    grid.moveTo(0, y).lineTo(width, y);
  }
  grid.stroke({ width: 1, color, alpha: 0.6 });

  const frame = new Graphics();
  frame.rect(0, 0, width, height).stroke({ width: 1, color: 0x4a6180, alpha: 0.9 });

  app.stage.addChild(grid, frame);
}
