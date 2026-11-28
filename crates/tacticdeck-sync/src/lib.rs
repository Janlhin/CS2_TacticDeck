//! 播放时钟：把「墙钟时间」换算成 demo tick，并吸收掉渲染帧的抖动。
//!
//! 关键约束来自 G2：连续播放 3 分钟后，时钟与墙钟的偏差要小于 100 ms
//! （CS2 的 tick 是 1/64 s ≈ 15.6 ms，100 ms 约等于 6.4 个 tick）。

mod clock;

pub use clock::TickClock;
