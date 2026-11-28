import { formatSeconds, roundIndexAt, tickToSeconds, totalTicks } from '../replay/selectors.ts';
import type { Replay } from '../replay/types.ts';

type Props = {
  replay: Replay;
  tick: number;
  playing: boolean;
  speed: number;
  onSeek: (tick: number) => void;
  onToggle: () => void;
  onSpeed: (speed: number) => void;
};

const SPEEDS = [0.5, 1, 2, 4];

/** 回合条 + 播放头：以 tick 为单位，跳转即改播放头 */
export function Timeline({ replay, tick, playing, speed, onSeek, onToggle, onSpeed }: Props) {
  const end = totalTicks(replay);
  const currentRound = roundIndexAt(replay, tick);
  const rounds = replay.rounds;

  return (
    <div className="timeline">
      <div className="timeline-rounds" role="tablist" aria-label="回合">
        {rounds.map((round) => (
          <button
            key={round.index}
            type="button"
            role="tab"
            aria-selected={round.index === currentRound}
            className={`round-chip${round.index === currentRound ? ' is-active' : ''}`}
            onClick={() => onSeek(round.startTick)}
            title={`回合 ${round.index + 1} · ${round.winner ? (round.winner === 't' ? 'T 胜' : 'CT 胜') : '进行中'}`}
          >
            R{round.index + 1}
          </button>
        ))}
      </div>

      <div className="timeline-scrub">
        <button type="button" className="play-btn" onClick={onToggle} aria-label={playing ? '暂停' : '播放'}>
          {playing ? '❚❚' : '▶'}
        </button>
        <input
          className="scrub"
          type="range"
          min={0}
          max={end}
          step={1}
          value={Math.round(tick)}
          onChange={(event) => onSeek(Number(event.target.value))}
          aria-label="播放头"
        />
        <span className="timeline-time">
          tick {Math.round(tick)} · {formatSeconds(tickToSeconds(replay, tick))}
        </span>
        <div className="speed-group" role="group" aria-label="播放倍率">
          {SPEEDS.map((value) => (
            <button
              key={value}
              type="button"
              className={`speed-btn${value === speed ? ' is-active' : ''}`}
              onClick={() => onSpeed(value)}
            >
              {value}×
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
