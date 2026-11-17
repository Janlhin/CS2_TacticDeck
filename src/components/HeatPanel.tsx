import { heatGrid, killSpots } from '../replay/selectors.ts';
import type { Replay } from '../replay/types.ts';

type Props = {
  replay: Replay;
  tick: number;
};

/** 热图网格：20 列 12 行，行号自上而下，与板面翻转 y 轴后的视觉一致 */
const COLS = 20;
const ROWS = 12;

/** 击杀落点热图：只统计当前回合、播放头之前的击杀 */
export function HeatPanel({ replay, tick }: Props) {
  const spots = killSpots(replay, tick, 'round');
  const cells = heatGrid(spots, COLS, ROWS);
  const hottest = cells.reduce((best, cell) => (cell.count > best ? cell.count : best), 0);

  return (
    <div className="heat-panel">
      <div
        className="heat-grid"
        aria-label="击杀落点热图"
        style={{ gridTemplateColumns: `repeat(${COLS}, 1fr)`, gridTemplateRows: `repeat(${ROWS}, 1fr)` }}
      >
        {cells.map((cell) => (
          <div
            key={`${cell.col}-${cell.row}`}
            className="heat-cell"
            style={
              cell.count > 0
                ? { backgroundColor: `rgba(210, 153, 34, ${(0.15 + cell.level * 0.75).toFixed(3)})` }
                : undefined
            }
            title={cell.count > 0 ? `${cell.count} 次` : ''}
          />
        ))}
      </div>
      <p className="hint">
        本回合落点 {spots.length} 个 · 最热格 {hottest} 次
      </p>
    </div>
  );
}
