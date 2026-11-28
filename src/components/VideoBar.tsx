import { useEffect, useRef, useState } from 'react';
import {
  mountedVideoSlots,
  selectTrack,
  sliceTracksByRange,
  visibleCoverRange,
  type TrackIndex,
  type TrackManifest,
} from '../replay/tracks.ts';

/**
 * 封面单项宽度（px）：与 `styles.scss` 中 `.video-cover` 的 `flex-basis` 写死同一数值，
 * 区间按「单项尺寸」折算可见下标；改宽度必须两处同步。
 */
const COVER_ITEM_WIDTH = 120;
/** 视口外左右多缓冲的封面数，避免快速横向滚动时边缘白屏 */
const COVER_OVERSCAN = 1;
/** 挂载前的视口宽度兜底常量；挂载后由容器实测 `clientWidth` 覆盖，避免首屏区间算空 */
const COVER_VIEWPORT_FALLBACK = 360;

type Props = {
  /** 已注入的清单；null 表示尚未接线，此时只渲染空态文案 */
  manifest: TrackManifest | null;
  /** 清单注入时构建的 slot → 轨 索引；切主不重建 */
  index: TrackIndex;
  /** 当前主视图槽位（player 槽 id 即 slot）；overview 为 0 */
  activeSlot: number;
  /** 点击封面切主；唯一入口仍是 store 的 setActiveView */
  onSelect: (slot: number) => void;
};

/**
 * T-07 视频条：上方主视频区只挂当前主轨一路 `<video>`，下方横向封面条按可见区间懒渲染。
 * 本轮不加载真实媒体：封面 / 视频路径都来自清单数据，组件不硬编码任何媒体 URL。
 */
export function VideoBar({ manifest, index, activeSlot, onSelect }: Props) {
  // hooks 必须先于提前 return 调用：空态分支也走同一组件实例
  const listRef = useRef<HTMLDivElement | null>(null);
  const [scrollOffset, setScrollOffset] = useState(0);
  const [viewportWidth, setViewportWidth] = useState(COVER_VIEWPORT_FALLBACK);

  // 挂载后与容器尺寸变化时同步实测值；横向条的「位置偏移」是 scrollLeft，「视口尺寸」是 clientWidth
  useEffect(() => {
    const container = listRef.current;
    if (!container) return;
    const sync = () => {
      setScrollOffset(container.scrollLeft);
      setViewportWidth(container.clientWidth);
    };
    sync();
    const observer = new ResizeObserver(sync);
    observer.observe(container);
    return () => observer.disconnect();
  }, []);

  if (!manifest) {
    return (
      <div className="video-bar video-bar-empty">
        <span className="viewport-placeholder-title">录像轨清单未注入</span>
        <span className="viewport-placeholder-sub">
          接入录制产出后，这里按回合列出各路 POV 封面并点击切主；当前只到可视化与逻辑层。
        </span>
      </div>
    );
  }

  // 只允许主视图那一路挂载视频元素，最多 1 个；其余路数只留封面，不挂 `<video>`
  const mounted = mountedVideoSlots(index, activeSlot);
  const activeTrack = mounted[0] !== undefined ? selectTrack(index, mounted[0]) : undefined;

  // 按可见区间切出本帧应挂载 DOM 的封面；区间外的路数不渲染，不是渲染后再隐藏
  const range = visibleCoverRange(
    scrollOffset,
    COVER_ITEM_WIDTH,
    viewportWidth,
    manifest.tracks.length,
    COVER_OVERSCAN,
  );
  const visibleTracks = sliceTracksByRange(manifest.tracks, range);

  return (
    <div className="video-bar">
      <div className="video-bar-stage">
        {activeTrack ? (
          <video
            className="video-bar-main"
            playsInline
            preload="none"
            poster={activeTrack.cover}
            src={activeTrack.video}
            // 本轮无真实录制，不自动播放；对齐播放由后续同步层接管
            muted
          />
        ) : (
          <div className="viewport-placeholder">
            <span className="viewport-placeholder-title">当前主视角暂无录像轨</span>
            <span className="viewport-placeholder-sub">该槽位不在本回合清单内；下方封面列出已注入的录像路数。</span>
          </div>
        )}
      </div>

      <div
        ref={listRef}
        className="video-bar-covers"
        aria-label="录像轨封面列表"
        onScroll={(event) => setScrollOffset(event.currentTarget.scrollLeft)}
      >
        {visibleTracks.map((track) => (
          <button
            key={track.slot}
            type="button"
            className={`video-cover${track.slot === activeSlot ? ' is-active' : ''}`}
            onClick={() => onSelect(track.slot)}
            title={`P${track.slot}`}
          >
            <img src={track.cover} alt={`P${track.slot} 封面`} loading="lazy" decoding="async" />
            <span className="video-cover-label">P{track.slot}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
