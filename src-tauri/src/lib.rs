use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub pov_slots: u8,
}

/// 五路是产品定义的一部分：主 POV + 最多四路跟随。
pub const POV_SLOTS: u8 = 5;

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
    fn app_info_reports_five_pov_slots() {
        let info = app_info();
        assert_eq!(info.pov_slots, POV_SLOTS);
        assert_eq!(info.pov_slots, 5);
    }

    #[test]
    fn app_info_version_matches_cargo_manifest() {
        let info = app_info();
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert!(!info.version.is_empty());
    }
}
