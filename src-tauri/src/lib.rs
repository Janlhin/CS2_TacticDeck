use serde::Serialize;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_app_info])
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
