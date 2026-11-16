import { useEffect, useState } from 'react';
import { RadarCanvas } from './components/RadarCanvas';
import { useAppStore } from './store/appStore';

type AppInfo = {
  name: string;
  version: string;
  pov_slots: number;
};

const BUILD_INFO: AppInfo = { name: 'CS2 TacticDeck', version: '0.1.0', pov_slots: 5 };

function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export default function App() {
  const slots = useAppStore((state) => state.slots);
  const activeSlot = useAppStore((state) => state.activeSlot);
  const setActiveSlot = useAppStore((state) => state.setActiveSlot);
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
            已绑定 {loadedCount}/{info.pov_slots} 路
          </span>
        </div>
      </header>

      <main className="workspace">
        <section className="pov-grid" aria-label="五路视角">
          {slots.map((slot) => (
            <button
              key={slot.id}
              type="button"
              className={`pov-cell${slot.id === activeSlot ? ' is-active' : ''}`}
              onClick={() => setActiveSlot(slot.id)}
            >
              <span className="pov-label">{slot.label}</span>
              <span className="pov-source">{slot.source || '未绑定 demo'}</span>
            </button>
          ))}
        </section>

        <aside className="sidebar">
          <section className="panel">
            <h2>盘面</h2>
            <RadarCanvas />
            <p className="hint">网格为占位；地图校准接入后替换为真实 radar。</p>
          </section>
          <section className="panel">
            <h2>复盘参数</h2>
            <ul className="kv">
              <li>
                <span>当前视角</span>
                <strong>POV {activeSlot}</strong>
              </li>
              <li>
                <span>播放刻度</span>
                <strong>tick → 秒（待接解析）</strong>
              </li>
              <li>
                <span>同步方式</span>
                <strong>主 POV 驱动 + 其他路跟随</strong>
              </li>
            </ul>
          </section>
        </aside>
      </main>

      <footer className="statusbar">
        <span>骨架阶段：布局与状态层可用，解析与播放尚未接入</span>
        <span className="sep">|</span>
        <span>验收口径 npm run verify</span>
      </footer>
    </div>
  );
}
