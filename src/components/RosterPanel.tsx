import { TEAM_COLOR_HEX } from '../replay/geometry.ts';
import { rosterAt } from '../replay/selectors.ts';
import type { Replay } from '../replay/types.ts';

type Props = {
  replay: Replay;
  tick: number;
  focusSlot: number;
  onFocus: (slot: number) => void;
};

/** 右栏十人状态：HP / 存活 / 阵营，随播放头更新 */
export function RosterPanel({ replay, tick, focusSlot, onFocus }: Props) {
  const rows = rosterAt(replay, tick);

  return (
    <ul className="roster">
      {rows.map(({ meta, frame }) => {
        const hp = frame?.hp ?? 0;
        const alive = frame?.alive ?? false;
        return (
          <li
            key={meta.slot}
            className={`roster-row${meta.slot === focusSlot ? ' is-focus' : ''}${alive ? '' : ' is-dead'}`}
            onClick={() => onFocus(meta.slot)}
          >
            <span className="roster-team" style={{ background: TEAM_COLOR_HEX[meta.team] }} aria-hidden="true" />
            <span className="roster-slot">P{meta.slot}</span>
            <span className="roster-name">{meta.name}</span>
            <span className="roster-hp">
              <span className="roster-hp-bar">
                <span className="roster-hp-fill" style={{ width: `${Math.max(0, Math.min(100, hp))}%` }} />
              </span>
              <span className="roster-hp-text">{alive ? Math.round(hp) : 'KIA'}</span>
            </span>
          </li>
        );
      })}
    </ul>
  );
}
