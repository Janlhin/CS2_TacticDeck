//! 本机 demo 库：扫描目录里的 `.dem` 与已导出的 `replay.json`，供左栏入口挑选。

use serde::Serialize;
use std::path::{Path, PathBuf};

/// 库里的一条候选：要么是还没导出成 replay.json 的 `.dem`，要么是已导出的回放
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LibraryEntry {
    pub path: String,
    pub name: String,
    /// `demo`（待导出）或 `replay`（可直接载入）
    pub kind: String,
    pub map: Option<String>,
    pub rounds: Option<u32>,
    pub ticks: Option<u32>,
    pub bytes: u64,
}

/// 默认库目录：环境变量优先，其次「文档/CS2_TacticDeck/demos」，最后是当前工作目录
pub fn default_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(dir) = std::env::var("TACTICDECK_DEMO_DIR")
        && !dir.trim().is_empty()
    {
        dirs.push(PathBuf::from(dir));
    }
    if let Ok(home) = std::env::var("USERPROFILE") {
        dirs.push(
            PathBuf::from(home)
                .join("Documents")
                .join("CS2_TacticDeck")
                .join("demos"),
        );
    }
    if let Ok(cwd) = std::env::current_dir() {
        dirs.push(cwd);
    }
    // dev 构建时 Tauri 进程的工作目录常常是 `src-tauri`，再补上仓库根
    dirs.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    if let Some(root) = Path::new(env!("CARGO_MANIFEST_DIR")).parent() {
        dirs.push(root.to_path_buf());
    }
    dirs
}

/// 扫描一个目录（不递归）：`.dem` 记作待导出条目，能解析成回放的 `.json` 记作可载入条目
pub fn scan_dir(dir: &Path) -> Vec<LibraryEntry> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for item in entries.flatten() {
        let path = item.path();
        if !path.is_file() {
            continue;
        }
        if let Some(entry) = summarize(&path) {
            out.push(entry);
        }
    }
    out.sort_by_key(|entry| entry.name.to_lowercase());
    out
}

/// 把一个文件描述成库条目；不认识的扩展名或解析失败的回放返回 `None`
pub fn summarize(path: &Path) -> Option<LibraryEntry> {
    let name = path.file_name()?.to_string_lossy().to_string();
    let ext = path
        .extension()
        .map(|value| value.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let bytes = std::fs::metadata(path).ok()?.len();
    let base = LibraryEntry {
        path: path.to_string_lossy().to_string(),
        name: name.clone(),
        kind: "demo".to_string(),
        map: None,
        rounds: None,
        ticks: None,
        bytes,
    };
    match ext.as_str() {
        "dem" => Some(base),
        "json" => {
            let text = std::fs::read_to_string(path).ok()?;
            let value: serde_json::Value = serde_json::from_str(&text).ok()?;
            summarize_replay(&value, base)
        }
        _ => None,
    }
}

/// 从解析好的 JSON 里取地图 / 回合数 / 总 tick；不像回放的直接判负
fn summarize_replay(value: &serde_json::Value, base: LibraryEntry) -> Option<LibraryEntry> {
    let frames = value.get("frames")?.as_array()?;
    let rounds = value.get("rounds")?.as_array()?;
    let map = value.get("map")?.as_str()?.to_string();
    let ticks = frames
        .last()
        .and_then(|frame| frame.get("tick"))
        .and_then(|tick| tick.as_u64())
        .map(|tick| tick as u32);
    Some(LibraryEntry {
        kind: "replay".to_string(),
        map: Some(map),
        rounds: Some(rounds.len() as u32),
        ticks,
        ..base
    })
}

/// 列表：给定目录就只看那一个，否则按默认目录逐个扫、按路径去重
pub fn list(dir: Option<&Path>) -> Vec<LibraryEntry> {
    let dirs = match dir {
        Some(dir) => vec![dir.to_path_buf()],
        None => default_dirs(),
    };
    let mut out: Vec<LibraryEntry> = Vec::new();
    for dir in dirs {
        for entry in scan_dir(&dir) {
            if out.iter().any(|existing| existing.path == entry.path) {
                continue;
            }
            out.push(entry);
        }
    }
    out
}

/// 读一份回放；只做最低限度校验，字段合法性交给前端 `parseReplayPayload`
pub fn read_replay(path: &Path) -> Result<serde_json::Value, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("读不了 {}：{error}", path.display()))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|error| format!("不是合法 JSON：{error}"))?;
    if value
        .get("frames")
        .and_then(|value| value.as_array())
        .is_none()
    {
        return Err("这份 JSON 不像 replay.json（缺 frames）".to_string());
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tacticdeck-lib-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample_replay() -> serde_json::Value {
        serde_json::json!({
            "map": "de_mirage",
            "tickRate": 64,
            "players": [],
            "rounds": [{"index": 0, "startTick": 65, "endTick": 8971, "winner": "t"}],
            "frames": [{"tick": 72, "players": []}, {"tick": 6575, "players": []}],
            "events": []
        })
    }

    #[test]
    fn library_scan_finds_demo_and_replay() {
        let dir = temp_dir("scan");
        fs::write(dir.join("test_demo.dem"), b"fake").unwrap();
        fs::write(
            dir.join("replay.json"),
            serde_json::to_string(&sample_replay()).unwrap(),
        )
        .unwrap();
        fs::write(dir.join("notes.txt"), b"ignore").unwrap();

        let entries = scan_dir(&dir);
        assert_eq!(entries.len(), 2, "只认 .dem 与 replay.json：{entries:?}");
        let replay = entries
            .iter()
            .find(|entry| entry.name == "replay.json")
            .expect("应识别出 replay.json");
        assert_eq!(replay.kind, "replay");
        assert_eq!(replay.map.as_deref(), Some("de_mirage"));
        assert_eq!(replay.rounds, Some(1));
        assert_eq!(replay.ticks, Some(6575));
        let demo = entries
            .iter()
            .find(|entry| entry.name == "test_demo.dem")
            .expect("应识别出 demo");
        assert_eq!(demo.kind, "demo");
        assert_eq!(demo.map, None);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn library_ignores_json_that_is_not_a_replay() {
        let dir = temp_dir("junk");
        fs::write(dir.join("package.json"), br#"{"name":"x"}"#).unwrap();
        assert!(scan_dir(&dir).is_empty(), "普通 JSON 不应进库");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_replay_rejects_non_replay_json() {
        let dir = temp_dir("read");
        let plain = dir.join("plain.json");
        fs::write(&plain, br#"{"a":1}"#).unwrap();
        assert!(read_replay(&plain).is_err(), "缺 frames 应报错");
        assert!(
            read_replay(&dir.join("missing.json")).is_err(),
            "缺文件应报错"
        );

        let good = dir.join("replay.json");
        fs::write(&good, serde_json::to_string(&sample_replay()).unwrap()).unwrap();
        let value = read_replay(&good).unwrap();
        assert_eq!(value["map"], "de_mirage");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn default_dirs_are_deduplicated_by_list() {
        let dir = temp_dir("dedup");
        fs::write(
            dir.join("replay.json"),
            serde_json::to_string(&sample_replay()).unwrap(),
        )
        .unwrap();
        let entries = list(Some(&dir));
        assert_eq!(entries.len(), 1);
        let listed = list(Some(&dir));
        assert_eq!(listed[0].path, entries[0].path);
        let _ = fs::remove_dir_all(&dir);
    }
}
