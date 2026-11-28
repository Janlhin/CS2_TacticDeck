import { formatSeconds, roundEvents } from '../replay/selectors.ts';
import type { Replay } from '../replay/types.ts';

type Props = {
  replay: Replay;
  tick: number;
  onSeek: (tick: number) => void;
};

/** 本回合事件列表：按时间正序，未到的事件置灰；点击任意一条把播放头跳到该 tick */
export function EventFeed({ replay, tick, onSeek }: Props) {
  const rows = roundEvents(replay, tick);

  if (rows.length === 0) {
    return <p className="hint">本回合没有击杀事件。</p>;
  }

  return (
    <ul className="event-feed">
      {rows.map((row) => (
        <li key={`${row.event.tick}-${row.event.victim}`}>
          <button
            type="button"
            className={`event-row${row.passed ? '' : ' is-future'}`}
            onClick={() => onSeek(row.event.tick)}
            aria-label={`跳到 tick ${row.event.tick}`}
          >
            <span className="event-time">{formatSeconds(row.elapsed)}</span>
            <span className="event-text">
              P{row.event.killer} → P{row.event.victim}
              {row.event.headshot ? ' · 爆头' : ''}
            </span>
            <span className="event-tick">{row.event.tick}</span>
          </button>
        </li>
      ))}
    </ul>
  );
}
