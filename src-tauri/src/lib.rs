use serde::Serialize;

mod library;

#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub pov_slots: u8,
}

/// 十名选手各一路 POV；另有固定 2D 总览一个槽位，左栏合计 11 项。
pub const POV_SLOTS: u8 = 10;

pub fn app_info() -> AppInfo {
    AppInfo {
        name: "CS2 TacticDeck".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        pov_slots: POV_SLOTS,
    }
}

#[tauri::command]
fn get_app_info() -> AppInfo {
    app_info()
}

/// 本机 demo 库列表；给 `dir` 就只扫那一个目录
#[tauri::command]
fn list_replays(dir: Option<String>) -> Vec<library::LibraryEntry> {
    library::list(dir.as_deref().map(std::path::Path::new))
}

/// 读一份已导出的 replay.json
#[tauri::command]
fn load_replay(path: String) -> Result<serde_json::Value, String> {
    library::read_replay(std::path::Path::new(&path))
}

/// 地图雷达底图：从本机 CS2 的 VPK 现取一张 PNG，回 data URL（带缓存）
#[tauri::command]
fn radar_image(map: String) -> Result<String, String> {
    use base64::Engine as _;
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(cache) = cache.lock()
        && let Some(url) = cache.get(&map)
    {
        return Ok(url.clone());
    }

    let png = tacticdeck_map::radar::radar_png(&map, None)
        .map_err(|error| format!("取 {map} 雷达底图失败：{error}"))?;
    let url = format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png)
    );
    if let Ok(mut cache) = cache.lock() {
        cache.insert(map, url.clone());
    }
    Ok(url)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            list_replays,
            load_replay,
            radar_image
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 启动失败");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_reports_ten_player_slots() {
        let info = app_info();
        assert_eq!(info.pov_slots, POV_SLOTS);
        assert_eq!(info.pov_slots, 10);
    }

    #[test]
    fn app_info_version_matches_cargo_manifest() {
        let info = app_info();
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert!(!info.version.is_empty());
    }
}
