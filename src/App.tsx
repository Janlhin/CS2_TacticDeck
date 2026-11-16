import { useEffect, useState } from 'react';
import { RadarCanvas } from './components/RadarCanvas';
import { useAppStore } from './store/appStore';

type AppInfo = {
  name: string;
  version: string;
  pov_slots: number;
};

const BUILD_INFO: AppInfo = { name: 'CS2 TacticDeck', version: '0.1.0', pov_slots: 10 };

function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export default function App() {
  const views = useAppStore((state) => state.views);
  const activeView = useAppStore((state) => state.activeView);
  const setActiveView = useAppStore((state) => state.setActiveView);
  const rotate = useAppStore((state) => state.rotate);
  const loadedCount = useAppStore((state) => state.loadedCount());
  const [info, setInfo] = useState<AppInfo>(BUILD_INFO);
  const [bridge, setBridge] = useState<'pending' | 'ok' | 'browser'>('pending');

  useEffect(() => {
    if (!isTauriRuntime()) {
      setBridge('browser');
      return;
    }
    void (async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      const payload = await invoke<AppInfo>('get_app_info');
      setInfo(payload);
      setBridge('ok');
    })();
  }, []);

  const overview = views.find((view) => view.kind === 'overview');
  const players = views.filter((view) => view.kind === 'player');
  const current = views.find((view) => view.id === activeView) ?? overview ?? views[0];

  return (
    <div className="shell">
      <header className="topbar">
        <div className="brand">
          <span className="brand-mark" aria-hidden="true" />
          <span className="brand-name">{info.name}</span>
          <span className="brand-version">v{info.version}</span>
        </div>
        <div className="topbar-status">
          <span className={`dot dot-${bridge}`} aria-hidden="true" />
          <span>
            {bridge === 'ok' ? 'Rust 桥已连通' : bridge === 'browser' ? '浏览器预览（无 Rust 桥）' : '正在握手'}
          </span>
          <span className="sep">|</span>
          <span>
            已绑定 {loadedCount}/{info.pov_slots} 人
          </span>
        </div>
      </header>

      <main className="workspace">
        <aside className="viewrail" aria-label="视角列表">
          {overview && (
            <button
              type="button"
              className={`rail-item rail-fixed${overview.id === activeView ? ' is-active' : ''}`}
              onClick={() => setActiveView(overview.id)}
            >
              <span className="rail-badge rail-badge-overview" aria-hidden="true">
                ◆
              </span>
              <span className="rail-label">{overview.label}</span>
              <span className="rail-meta">固定</span>
            </button>
          )}

          <div
            className="rail-list"
            onWheel={(event) => {
              if (event.shiftKey) {
                event.preventDefault();
                rotate(event.deltaY > 0 ? 1 : -1);
              }
            }}
          >
            {players.map((view) => (
              <button
                key={view.id}
                type="button"
                className={`rail-item${view.id === activeView ? ' is-active' : ''}`}
                onClick={() => setActiveView(view.id)}
              >
                <span className={`rail-badge rail-badge-${view.team}`} aria-hidden="true" />
                <span className="rail-label">{view.label}</span>
                <span className="rail-meta">{view.source ? '占用' : '空'}</span>
              </button>
            ))}
          </div>

          <p className="rail-hint">滚轮下滑轮换十人 · Shift+滚轮切换主视角</p>
        </aside>

        <section className="viewport" aria-label="主视图">
          <header className="viewport-head">
            <span>{current?.kind === 'overview' ? '2D 总览' : `POV ${current?.label ?? ''}`}</span>
            <span className="viewport-meta">tick → 秒（待接解析）</span>
          </header>
          <div className="viewport-body">
            {current?.kind === 'overview' ? (
              <div className="viewport-overview">
                <RadarCanvas />
                <p className="hint">2D 总览：十人点位将由 demo 解析填充，当前为网格占位。</p>
              </div>
            ) : (
              <div className="viewport-placeholder">
                <span className="viewport-placeholder-title">{current?.label} 视角视频未接入</span>
                <span className="viewport-placeholder-sub">
                  录制完成后按 tick 对齐播放；未录制时从这里切换十人镜头。
                </span>
              </div>
            )}
          </div>
        </section>

        <aside className="sidebar">
          <section className="panel">
            <h2>复盘参数</h2>
            <ul className="kv">
              <li>
                <span>当前视图</span>
                <strong>{current?.kind === 'overview' ? '2D 总览' : `POV ${current?.label}`}</strong>
              </li>
              <li>
                <span>左栏槽位</span>
                <strong>1 固定 + {players.length} 人</strong>
              </li>
              <li>
                <span>播放刻度</span>
                <strong>tick → 秒（待接解析）</strong>
              </li>
              <li>
                <span>同步方式</span>
                <strong>主视图驱动 + 其余跟随</strong>
              </li>
            </ul>
          </section>
          <section className="panel">
            <h2>录制状态</h2>
            <ul className="kv">
              <li>
                <span>采集模型</span>
                <strong>单机多遍（逐遍锁选手）</strong>
              </li>
              <li>
                <span>已解析回合</span>
                <strong>0</strong>
              </li>
            </ul>
            <p className="hint">录制完成会自动切到 2D 回放（T-16）。</p>
          </section>
        </aside>
      </main>

      <footer className="statusbar">
        <span>骨架阶段：布局与状态层可用，解析与播放尚未接入</span>
        <span className="sep">|</span>
        <span>验收口径 npm run verify</span>
        <span className="sep">|</span>
        <span className="license">AGPL-3.0-or-later · 源码 github.com/Janlhin/CS2_TacticDeck</span>
      </footer>
    </div>
  );
}
