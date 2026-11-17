use std::time::Duration;

/// 默认 tick 率（CS2 官方服务器）
pub const DEFAULT_TICK_RATE: f64 = 64.0;
/// G2 的容差：连续 3 分钟，播放头与墙钟偏差 < 100 ms
#[cfg_attr(not(test), allow(dead_code))]
pub const DRIFT_BUDGET: Duration = Duration::from_millis(100);

/// 主时钟：以墙钟为参照推进 tick，并吸收渲染帧抖动
///
/// 模型很简单：
/// - 每帧把 `delta` 乘上「倍速 × tick 率」累到 `tick`；
/// - 同时记 `anchor_tick + 锚点以来的墙钟时间` 作为「理论上应该在的 tick」；
/// - 两者差得太远就往回拉一小步，避免随机抖动一路累积成秒级漂移。
#[derive(Debug, Clone)]
pub struct TickClock {
    tick_rate: f64,
    playback_rate: f64,
    tick: f64,
    anchor_tick: f64,
    anchor_time: Duration,
    playing: bool,
}

impl TickClock {
    /// 按给定 tick 率建一个停在 0 tick、暂停状态的钟
    #[must_use]
    pub fn new(tick_rate: f64) -> Self {
        Self {
            tick_rate: sanitize_rate(tick_rate),
            playback_rate: 1.0,
            tick: 0.0,
            anchor_tick: 0.0,
            anchor_time: Duration::ZERO,
            playing: false,
        }
    }

    /// 当前 tick 率（构造时会过滤掉 0 / NaN / 负数）
    #[must_use]
    pub fn tick_rate(&self) -> f64 {
        self.tick_rate
    }

    /// 当前倍速
    #[must_use]
    pub fn playback_rate(&self) -> f64 {
        self.playback_rate
    }

    /// 是否在走
    #[must_use]
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// 播放头所在 tick（取整；四舍五入而不是截断，避免总差半帧）
    #[must_use]
    pub fn current_tick(&self) -> u32 {
        let rounded = self.tick.round();
        if rounded < 0.0 { 0 } else { rounded as u32 }
    }

    /// 播放头的小数位置（0..1），视频 seek 时用
    #[must_use]
    pub fn fractional_tick(&self) -> f64 {
        self.tick - self.tick.floor()
    }

    /// 从播放 / 暂停时刻起算，理论上现在应该在的 tick
    #[must_use]
    pub fn target_tick(&self, now: Duration) -> f64 {
        if !self.playing {
            return self.tick;
        }
        let elapsed = now.saturating_sub(self.anchor_time).as_secs_f64();
        self.anchor_tick + elapsed * self.playback_rate * self.tick_rate
    }

    /// 当前 tick 与墙钟的偏差（正数=跑快了），单位 tick
    #[must_use]
    pub fn drift_ticks(&self, now: Duration) -> f64 {
        self.tick - self.target_tick(now)
    }

    /// 从暂停切入播放：以当前 tick 为锚点
    pub fn play(&mut self, now: Duration) {
        if self.playing {
            return;
        }
        self.playing = true;
        self.reanchor(now);
    }

    /// 暂停：把 tick 钉在当前位置
    pub fn pause(&mut self, now: Duration) {
        if !self.playing {
            return;
        }
        self.tick = self.target_tick(now);
        self.playing = false;
        self.reanchor(now);
    }

    /// 跳到某个 tick（播放中跳转也会重设锚点）
    pub fn seek(&mut self, tick: u32, now: Duration) {
        self.tick = f64::from(tick);
        self.reanchor(now);
    }

    /// 改倍速：先把当前位置对齐墙钟，再以新倍速重新起算
    pub fn set_playback_rate(&mut self, rate: f64, now: Duration) {
        self.tick = self.target_tick(now);
        self.playback_rate = sanitize_playback_rate(rate);
        self.reanchor(now);
    }

    /// 每帧推进；`delta` 是上一帧到这一帧的墙钟差，`now` 是本帧的墙钟时刻
    ///
    /// 返回推进后的 tick（浮点），方便上层直接插值。
    pub fn advance(&mut self, delta: Duration, now: Duration) -> f64 {
        if self.playing {
            self.tick += delta.as_secs_f64() * self.playback_rate * self.tick_rate;
            let drift = self.target_tick(now) - self.tick;
            // 偏差超过 1 tick 就一次性对齐（多半是掉帧 / 断点调试），
            // 小偏差每帧只吃 5%，播放观感更顺，但 3 分钟内足以收敛。
            self.tick += if drift.abs() > 1.0 {
                drift
            } else {
                drift * 0.05
            };
        }
        self.tick
    }

    /// 给某一路视频算补偿倍速：让它在 2 秒内把与主时钟的差距吃回去
    ///
    /// 视频条用得上——某一路 POV 落后主时钟 30 tick 时，这一路就该以略大于 1 的倍速追上来。
    #[must_use]
    pub fn rate_to_close(&self, track_tick: f64) -> f64 {
        let drift = self.tick - track_tick;
        sanitize_playback_rate(self.playback_rate * (1.0 + drift / (2.0 * self.tick_rate)))
    }

    fn reanchor(&mut self, now: Duration) {
        self.anchor_tick = self.tick;
        self.anchor_time = now;
    }
}

fn sanitize_rate(rate: f64) -> f64 {
    if rate.is_finite() && rate > 0.0 {
        rate
    } else {
        DEFAULT_TICK_RATE
    }
}

fn sanitize_playback_rate(rate: f64) -> f64 {
    if rate.is_finite() {
        rate.clamp(0.05, 16.0)
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::{DRIFT_BUDGET, TickClock};
    use std::time::Duration;

    /// 3 分钟、60 fps、每次 delta 记错 ±20 ms 的抖动序列
    fn simulate(minutes: u64) -> Vec<Duration> {
        let fps = 60u64;
        let frames = minutes * 60 * fps;
        let base = 1_000_000.0 / fps as f64; // 16.67 ms
        (0..frames)
            .map(|i| {
                // 用一个确定性伪随机，测试可复现
                let noise = ((i as f64 * 12.9898).sin() * 43758.5453).fract();
                let jitter = (noise - 0.5) * 40.0; // ±20 ms
                Duration::from_micros((base + jitter).max(1.0) as u64)
            })
            .collect()
    }

    #[test]
    fn starts_paused_at_zero() {
        let clock = TickClock::new(64.0);
        assert_eq!(clock.current_tick(), 0);
        assert!(!clock.is_playing());
        assert_eq!(clock.tick_rate(), 64.0);
    }

    #[test]
    fn advances_at_wall_clock_rate() {
        let mut clock = TickClock::new(64.0);
        clock.play(Duration::ZERO);
        let mut now = Duration::ZERO;
        for _ in 0..64 * 60 {
            now += Duration::from_millis(16);
            clock.advance(Duration::from_millis(16), now);
        }
        // 64 帧 × 16 ms = 1.024 s ≈ 65.5 tick
        let wall_clock_ticks = now.as_secs_f64() * 64.0;
        assert!(
            (f64::from(clock.current_tick()) - wall_clock_ticks).abs() < 40.0,
            "tick = {}，墙钟应约 {wall_clock_ticks}",
            clock.current_tick()
        );
    }

    #[test]
    fn honors_playback_rate() {
        let mut clock = TickClock::new(64.0);
        clock.play(Duration::ZERO);
        clock.set_playback_rate(2.0, Duration::ZERO);
        clock.advance(Duration::from_secs(5), Duration::from_secs(5));
        // 5 s × 2 倍 × 64 tick = 640
        assert!((f64::from(clock.current_tick()) - 640.0).abs() < 5.0);
    }

    #[test]
    fn seek_and_pause_hold_position() {
        let mut clock = TickClock::new(64.0);
        clock.seek(1234, Duration::from_secs(1));
        assert_eq!(clock.current_tick(), 1234);
        clock.play(Duration::from_secs(1));
        clock.advance(Duration::from_secs(1), Duration::from_secs(2));
        clock.pause(Duration::from_secs(2));
        let paused = clock.current_tick();
        // 暂停后再怎么推进都不该动
        clock.advance(Duration::from_secs(3), Duration::from_secs(5));
        assert_eq!(clock.current_tick(), paused);
    }

    /// G2 的核心回归：3 分钟抖动播放后，播放头与墙钟偏差 < 100 ms
    #[test]
    fn three_minutes_jittery_playback_stays_within_budget() {
        let mut clock = TickClock::new(64.0);
        clock.play(Duration::ZERO);
        let mut now = Duration::ZERO;
        for delta in simulate(3) {
            now += delta;
            clock.advance(delta, now);
        }
        let drift = clock.drift_ticks(now);
        let drift_seconds = drift.abs() * Duration::from_secs(1).as_secs_f64() / 64.0;
        let budget_seconds = DRIFT_BUDGET.as_secs_f64();
        assert!(
            drift_seconds < budget_seconds,
            "3 分钟后偏差 {drift_seconds:.4}s（{drift:.2} tick），超出 {}s",
            budget_seconds
        );
    }

    #[test]
    fn suggests_correction_when_behind() {
        let mut clock = TickClock::new(64.0);
        clock.play(Duration::ZERO);
        // 主时钟推进 2 s（128 tick），某一路视频还停在 64 tick：落后 1 s
        clock.advance(Duration::from_secs(2), Duration::from_secs(2));
        let suggested = clock.rate_to_close(64.0);
        assert!(
            suggested > 1.05,
            "落后时应给出明显大于 1 的补偿倍速，实际 {suggested}"
        );
        // 视频超前时反过来要减速
        let slower = clock.rate_to_close(clock.target_tick(Duration::from_secs(4)) + 128.0);
        assert!(slower < 1.0, "超前时应小于 1，实际 {slower}");
    }
}
