//! 地图相关的只读能力：VPK 入口、雷达标定、雷达底图解码。
//!
//! 这一层不打日志、不读用户配置，方便 probe（命令行）与 Tauri 后端共用。

pub mod map_profile;
pub mod radar;
pub mod vpk;
