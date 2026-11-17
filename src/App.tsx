import { useEffect, useState } from 'react';
import { EventFeed } from './components/EventFeed.tsx';
import { HeatPanel } from './components/HeatPanel.tsx';
import { ReplayBoard } from './components/ReplayBoard.tsx';
import { RosterPanel } from './components/RosterPanel.tsx';
import { Timeline } from './components/Timeline.tsx';
import { buildSampleReplay } from './replay/fixture.ts';
import {
  entryLabel,
  firstReplay,
  isTauriRuntime,
  isUsableEntry,
  listLibrary,
  readReplay,
  type LibraryEntry,
} from './replay/library.ts';
import { aliveCount, formatSeconds, killsUpTo, roundAt, roundEvents, tickToSeconds } from './replay/selectors.ts';
import { useAppStore } from './store/appStore.ts';
import { OVERVIEW_ID } from './store/rail.ts';

type AppInfo = {
  name: string;
  version: string;
  pov_slots: number;
};

const BUILD_INFO: AppInfo = { name: 'CS2 TacticDeck', version: '0.1.0', pov_slots: 10 };

function errorText(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}

export default function App() {
  const views = useAppStore((state) => state.views);
  const activeView = useAppStore((state) => state.activeView);
  const setActiveView = useAppStore((state) => state.setActiveView);
  const rotate = useAppStore((state) => state.rotate);
  const loadedCount = useAppStore((state) => state.loadedCount());
  const replay = useAppStore((state) => state.replay);
  const tick = useAppStore((state) => state.tick);
  const playing = useAppStore((state) => state.playing);
  const speed = useAppStore((state) => state.speed);
  const loadReplay = useAppStore((state) => state.loadReplay);
  const seekTick = useAppStore((state) => state.seekTick);
  const advance = useAppStore((state) => state.advance);
  const togglePlay = useAppStore((state) => state.togglePlay);
  const setSpeed = useAppStore((state) => state.setSpeed);

  const [info, setInfo] = useState<AppInfo>(BUILD_INFO);
  const [bridge, setBridge] = useState<'pending' | 'ok' | 'browser'>('pending');
  const [library, setLibrary] = useState<LibraryEntry[]>([]);
  const [libraryNote, setLibraryNote] = useState<string | null>(null);
  const [loadedPath, setLoadedPath] = useState<string | null>(null);

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

  // 启动装载：有本机回放就先吃真实数据，否则退回合成样本把链路点亮
  useEffect(() => {
    if (useAppStore.getState().replay) return;
    const fallback = () => {
      const sample = buildSampleReplay();
      loadReplay(sample, `${sample.map}#sample`);
    };
    if (!isTauriRuntime()) {
      fallback();
      return;
    }
    void (async () => {
      try {
        const entries = await listLibrary();
        setLibrary(entries);
        const first = firstReplay(entries);
        if (!first) {
          setLibraryNote('没扫到回放：把 replay.json 放进 文档/CS2_TacticDeck/demos 再重新扫描。');
          fallback();
          return;
        }
        loadReplay(await readReplay(first.path), first.path);
        setLoadedPath(first.path);
      } catch (error) {
        setLibraryNote(`读库失败：${errorText(error)}`);
        fallback();
      }
    })();
  }, [loadReplay]);

  const refreshLibrary = async () => {
    if (!isTauriRuntime()) {
      setLibraryNote('浏览器预览下没有 Rust 桥，读不到本机文件。');
      return;
    }
    try {
      const entries = await listLibrary();
      setLibrary(entries);
      setLibraryNote(entries.length === 0 ? '没扫到回放或 demo。' : null);
    } catch (error) {
      setLibraryNote(`读库失败：${errorText(error)}`);
    }
  };

  const openEntry = async (entry: LibraryEntry) => {
    try {
      const next = await readReplay(entry.path);
      loadReplay(next, entry.path);
      setLoadedPath(entry.path);
      setActiveView(OVERVIEW_ID);
      setLibraryNote(null);
    } catch (error) {
      setLibraryNote(`载入失败：${errorText(error)}`);
    }
  };

  // 播放循环：requestAnimationFrame 驱动 store 里的 tick
  useEffect(() => {
    if (!playing) return;
    let raf = 0;
    let last = performance.now();
    const loop = (now: number) => {
      advance(now - last);
      last = now;
      raf = requestAnimationFrame(loop);
    };
    raf = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(raf);
  }, [playing, advance]);

  const overview = views.find((view) => view.kind === 'overview');
  const players = views.filter((view) => view.kind === 'player');
  const current = views.find((view) => view.id === activeView) ?? overview ?? views[0];
  const focusSlot = current?.kind === 'player' ? current.id : 0;

  const round = replay ? roundAt(replay, tick) : null;
  const kills = replay ? killsUpTo(replay, tick) : [];
  const latestKill = kills[0];
  const tAlive = replay ? aliveCount(replay, tick, 't') : 0;
  const ctAlive = replay ? aliveCount(replay, tick, 'ct') : 0;

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
            <span className="viewport-meta">
              {replay ? `tick ${Math.round(tick)} · ${formatSeconds(tickToSeconds(replay, tick))}` : 'tick → 秒'}
            </span>
          </header>
          <div className="viewport-body">
            {!replay ? (
              <div className="viewport-placeholder">
                <span className="viewport-placeholder-title">尚未载入回放</span>
                <span className="viewport-placeholder-sub">载入 demo 后这里显示十人 2D 回放。</span>
              </div>
            ) : current?.kind === 'overview' ? (
              <ReplayBoard replay={replay} tick={tick} focusSlot={0} />
            ) : (
              <div className="viewport-split">
                <div className="viewport-placeholder">
                  <span className="viewport-placeholder-title">{current?.label} POV 视频未接入</span>
                  <span className="viewport-placeholder-sub">
                    录制完成后按 tick 对齐播放；当前只在右栏与板面看到该选手点位。
                  </span>
                </div>
                <ReplayBoard replay={replay} tick={tick} focusSlot={focusSlot} />
              </div>
            )}
          </div>
          {replay && (
            <Timeline
              replay={replay}
              tick={tick}
              playing={playing}
              speed={speed}
              onSeek={seekTick}
              onToggle={togglePlay}
              onSpeed={setSpeed}
            />
          )}
        </section>

        <aside className="sidebar">
          <section className="panel">
            <h2>demo 库</h2>
            {library.length === 0 ? (
              <p className="hint">库里还没有条目。</p>
            ) : (
              <ul className="library-list">
                {library.map((entry) => (
                  <li key={entry.path}>
                    <button
                      type="button"
                      className={`library-item${loadedPath === entry.path ? ' is-loaded' : ''}`}
                      disabled={!isUsableEntry(entry)}
                      onClick={() => void openEntry(entry)}
                      title={entry.path}
                    >
                      <span className="library-name">{entry.name}</span>
                      <span className="library-meta">{entryLabel(entry)}</span>
                      {!isUsableEntry(entry) ? (
                        <span className="library-meta">待导出：probe --demo … --out replay.json</span>
                      ) : null}
                    </button>
                  </li>
                ))}
              </ul>
            )}
            <button type="button" className="link-button" onClick={() => void refreshLibrary()}>
              重新扫描
            </button>
            {libraryNote ? <p className="hint">{libraryNote}</p> : null}
          </section>

          <section className="panel">
            <h2>复盘参数</h2>
            <ul className="kv">
              <li>
                <span>地图</span>
                <strong>{replay?.map ?? '—'}</strong>
              </li>
              <li>
                <span>当前回合</span>
                <strong>{round ? `R${round.index + 1} / ${replay?.rounds.length}` : '—'}</strong>
              </li>
              <li>
                <span>存活</span>
                <strong>
                  T {tAlive} · CT {ctAlive}
                </strong>
              </li>
              <li>
                <span>播放刻度</span>
                <strong>{replay ? `${replay.tickRate} tick/s` : '—'}</strong>
              </li>
            </ul>
          </section>

          <section className="panel">
            <h2>事件</h2>
            {latestKill ? (
              <p className="kill-line">
                最近一击 P{latestKill.killer} → P{latestKill.victim}
                {latestKill.headshot ? ' · 爆头' : ''}
                <span className="kill-tick">tick {latestKill.tick}</span>
              </p>
            ) : (
              <p className="hint">当前播放头之前没有击杀。</p>
            )}
            {replay ? <EventFeed replay={replay} tick={tick} onSeek={seekTick} /> : null}
            {replay ? (
              <p className="hint">
                本回合共 {roundEvents(replay, tick).length} 次击杀 · 已发生 {kills.length} · 点击任一行跳到该 tick
              </p>
            ) : null}
          </section>

          <section className="panel panel-grow">
            <h2>十人状态</h2>
            {replay ? (
              <RosterPanel replay={replay} tick={tick} focusSlot={focusSlot} onFocus={setActiveView} />
            ) : (
              <p className="hint">未载入回放。</p>
            )}
          </section>

          <section className="panel">
            <h2>热图</h2>
            {replay ? <HeatPanel replay={replay} tick={tick} /> : <p className="hint">未载入回放。</p>}
          </section>
        </aside>
      </main>

      <footer className="statusbar">
        <span>
          {replay
            ? `${replay.map} · ${replay.rounds.length} 回合 · ${replay.players.length} 人${
                replay.meta ? ` · ${replay.meta.normalize}` : ' · 合成样本'
              }`
            : '未载入回放'}
        </span>
        <span className="sep">|</span>
        <span>验收口径 deno task verify</span>
        <span className="sep">|</span>
        <span className="license">AGPL-3.0-or-later · 源码 github.com/Janlhin/CS2_TacticDeck</span>
      </footer>
    </div>
  );
}
