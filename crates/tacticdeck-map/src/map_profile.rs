//! 地图雷达标定：从本机 CS2 的 `pak01_dir.vpk` 里取 `resource/overviews/*.txt`，
//! 解析出 `pos_x / pos_y / scale / rotate`，供世界坐标换算出雷达图层坐标。

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::vpk::Vpk;

/// 一张地图的雷达标定参数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MapProfile {
    /// `de_mirage` 这类地图名
    pub map: String,
    /// 贴图名，`overviews/de_mirage`
    pub material: String,
    /// 雷达图左上角对应的世界 X
    pub pos_x: f32,
    /// 雷达图左上角对应的世界 Y
    pub pos_y: f32,
    /// 每个像素代表多少世界单位
    pub scale: f32,
    /// 贴图是否被旋转过（0 / 90 / 180 / 270）
    pub rotate: f32,
    /// 雷达图边长（CS2 overview 统一 1024）
    pub image_size: u32,
}

/// 整张地图表的落盘格式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapProfiles {
    /// 取自哪个 VPK
    pub source: String,
    pub maps: Vec<MapProfile>,
}

impl MapProfile {
    /// 世界坐标 → 雷达图像素（左上原点，未旋转）
    pub fn world_to_pixel(&self, world_x: f32, world_y: f32) -> (f32, f32) {
        let px = (world_x - self.pos_x) / self.scale;
        let py = (self.pos_y - world_y) / self.scale;
        (px, py)
    }

    /// 世界坐标 → 0..1 归一化坐标（左下原点，直接喂前端契约）
    pub fn world_to_norm(&self, world_x: f32, world_y: f32) -> (f32, f32) {
        let (px, py) = self.world_to_pixel(world_x, world_y);
        let size = f32::from(self.image_size as u16);
        (px / size, 1.0 - py / size)
    }
}

/// 极简 KeyValues 解析：只要「键 + 标量值」，忽略注释与嵌套块
fn parse_scalars(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut chars = text.chars().peekable();
    let mut depth: usize = 0;
    let mut pending_key: Option<String> = None;

    while let Some(ch) = chars.next() {
        match ch {
            '/' if matches!(chars.peek(), Some('/')) => {
                // 行注释
                for next in chars.by_ref() {
                    if next == '\n' {
                        break;
                    }
                }
            }
            '"' => {
                let mut value = String::new();
                let mut closed = false;
                for next in chars.by_ref() {
                    if next == '"' {
                        closed = true;
                        break;
                    }
                    value.push(next);
                }
                if !closed {
                    break;
                }
                match pending_key.take() {
                    None => pending_key = Some(value),
                    Some(key) => {
                        if depth <= 1 {
                            out.push((key.to_ascii_lowercase(), value));
                        }
                    }
                }
            }
            '{' => {
                depth += 1;
                pending_key = None;
            }
            '}' => {
                depth = depth.saturating_sub(1);
                pending_key = None;
            }
            _ => {}
        }
    }
    out
}

fn scalar<'a>(pairs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    pairs
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.as_str())
}

/// 把 overview 里的贴图名归一成引擎地图名：
/// `de_dust2_v2` / `de_cache_new` / `de_ancient_night` 都要对回 `de_dust2` / `de_cache` / `de_ancient`
fn canonical_map_name(raw: &str) -> String {
    let name = raw.trim().trim_matches('"').trim().to_ascii_lowercase();
    for suffix in ["_v2", "_new", "_night", "_ladder", "_old"] {
        if let Some(stripped) = name.strip_suffix(suffix) {
            return stripped.to_string();
        }
    }
    name
}

/// 解析一份 overview 文本（VPK 里就是 CS:GO 那一套 KeyValues）
pub fn parse_overview(text: &str, fallback_map: &str) -> Result<MapProfile> {
    let pairs = parse_scalars(text);
    // 首行可能是 `// HLTV overview description file for de_dust2_v2.bsp` 这种注释，不能当地图名
    let first_line = text.lines().find(|line| {
        let trimmed = line.trim();
        !trimmed.is_empty() && !trimmed.starts_with("//")
    });
    let raw_name = first_line
        .map(|line| line.trim().trim_matches('"'))
        .filter(|name| !name.is_empty())
        .unwrap_or(fallback_map);
    let map = canonical_map_name(raw_name);

    let number = |key: &str| -> Result<f32> {
        scalar(&pairs, key)
            .with_context(|| format!("overview 缺少字段 {key}"))?
            .trim()
            .parse::<f32>()
            .with_context(|| format!("overview 字段 {key} 不是数字"))
    };

    Ok(MapProfile {
        material: scalar(&pairs, "material").unwrap_or_default().to_string(),
        map,
        pos_x: number("pos_x")?,
        pos_y: number("pos_y")?,
        scale: number("scale")?,
        rotate: scalar(&pairs, "rotate")
            .and_then(|value| value.trim().parse::<f32>().ok())
            .unwrap_or(0.0),
        image_size: 1024,
    })
}

/// 从 VPK 抽出全部 overview
pub fn extract_from_vpk(vpk_path: &Path) -> Result<MapProfiles> {
    let vpk = Vpk::open(vpk_path)?;
    let mut maps = Vec::new();
    for entry in vpk.find("resource/overviews", "txt") {
        let bytes = vpk
            .read(entry)
            .with_context(|| format!("读取 {} 失败", entry.full_name()))?;
        let text = String::from_utf8_lossy(&bytes);
        match parse_overview(&text, &entry.name) {
            Ok(profile) => maps.push(profile),
            Err(error) => eprintln!("跳过 {}：{error}", entry.full_name()),
        }
    }
    anyhow::ensure!(!maps.is_empty(), "VPK 里没有找到 resource/overviews/*.txt");
    maps.sort_by(|a, b| a.map.cmp(&b.map));
    // 同一张地图可能有 _v2 / _night 等多份 overview，保留第一份即可
    maps.dedup_by(|a, b| a.map == b.map);
    Ok(MapProfiles {
        source: vpk_path.display().to_string(),
        maps,
    })
}

/// 定位本机 CS2 的 `game/csgo` 目录：参数优先，其次环境变量，最后读 Steam 配置
pub fn discover_cs2(explicit: Option<&Path>) -> Result<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(path) = explicit {
        candidates.push(path.to_path_buf());
    }
    if let Ok(dir) = std::env::var("CS2_DIR")
        && !dir.trim().is_empty()
    {
        candidates.push(PathBuf::from(dir));
    }
    candidates.extend(steam_candidates());

    for candidate in &candidates {
        if let Some(found) = normalize_cs2_dir(candidate) {
            return Ok(found);
        }
    }
    anyhow::bail!(
        "找不到 CS2 的 game/csgo 目录；用 --cs2 <路径> 或环境变量 CS2_DIR 指定。试过 {} 个候选",
        candidates.len()
    )
}

fn normalize_cs2_dir(path: &Path) -> Option<PathBuf> {
    // 接受三种写法：game/csgo、Counter-Strike Global Offensive、其 game 目录
    let direct = path.join("pak01_dir.vpk");
    if direct.is_file() {
        return Some(path.to_path_buf());
    }
    let nested = path.join("game").join("csgo").join("pak01_dir.vpk");
    if nested.is_file() {
        return Some(path.join("game").join("csgo"));
    }
    let inner = path.join("csgo").join("pak01_dir.vpk");
    if inner.is_file() {
        return Some(path.join("csgo"));
    }
    None
}

#[cfg(windows)]
fn steam_candidates() -> Vec<PathBuf> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    let mut out = Vec::new();
    let steam_path = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("Software\\Valve\\Steam")
        .ok()
        .and_then(|key| key.get_value::<String, _>("SteamPath").ok());
    let Some(steam_path) = steam_path else {
        return out;
    };
    let steam_root = PathBuf::from(steam_path);
    out.push(cs2_common_dir(&steam_root));

    // libraryfolders.vdf 里可能还有别的盘
    let folders = steam_root.join("steamapps").join("libraryfolders.vdf");
    if let Ok(text) = std::fs::read_to_string(&folders) {
        for line in text.lines() {
            let line = line.trim();
            if !line.starts_with("\"path\"") {
                continue;
            }
            let value = line.rsplit('"').nth(1).unwrap_or("").replace("\\\\", "\\");
            if value.is_empty() {
                continue;
            }
            out.push(cs2_common_dir(&PathBuf::from(value)));
        }
    }
    out
}

#[cfg(not(windows))]
fn steam_candidates() -> Vec<PathBuf> {
    Vec::new()
}

fn cs2_common_dir(library_root: &Path) -> PathBuf {
    library_root
        .join("steamapps")
        .join("common")
        .join("Counter-Strike Global Offensive")
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIRAGE: &str = r#""de_mirage"
{
	"material"	"overviews/de_mirage"	// texture file
	"pos_x"		"-3230"	// X coordinate,
	"pos_y"		"1713"	// Y coordinate,
	"scale"		"5.00" 	// and used scale
	"rotate"	"0"	// map was rotated
	"zoom"		"0"	// optimal zoom
}
"#;

    #[test]
    fn parses_mirage_overview() {
        let profile = parse_overview(MIRAGE, "fallback").unwrap();
        assert_eq!(profile.map, "de_mirage");
        assert_eq!(profile.material, "overviews/de_mirage");
        assert_eq!(profile.pos_x, -3230.0);
        assert_eq!(profile.pos_y, 1713.0);
        assert_eq!(profile.scale, 5.0);
        assert_eq!(profile.rotate, 0.0);
        assert_eq!(profile.image_size, 1024);
    }

    #[test]
    fn skips_comment_first_line_for_map_name() {
        let text = r#"
// HLTV overview description file for de_dust2_v2.bsp
"de_dust2_v2"
{
	"material"	"overviews/de_dust2_v2"
	"pos_x"		"-2476"
	"pos_y"		"3239"
	"scale"		"4.4"
	"rotate"	"1"
}
"#;
        let profile = parse_overview(text, "de_dust2_v2").unwrap();
        assert_eq!(profile.map, "de_dust2", "注释不能被当地图名，_v2 也要归一");
    }

    #[test]
    fn maps_world_to_radar() {
        let profile = parse_overview(MIRAGE, "de_mirage").unwrap();
        // 雷达左上角那一点，像素应当接近 (0, 0)
        let (px, py) = profile.world_to_pixel(profile.pos_x, profile.pos_y);
        assert!(px.abs() < 0.001 && py.abs() < 0.001);
        // 归一化后落在左下原点：x≈0、y≈1
        let (x, y) = profile.world_to_norm(profile.pos_x, profile.pos_y);
        assert!(x.abs() < 0.001 && (y - 1.0).abs() < 0.001);
    }

    /// 校准用例：真实 demo（tick 72 出生帧）的世界坐标必须落到雷达图上的出生点位置
    ///
    /// 参考值来自 `data/radar_info/de_mirage.txt`：TSpawn (0.87, 0.36)、CTSpawn (0.28, 0.70)，
    /// 其中 y 以图片**底部**为原点，换算成像素（以左上角为原点、1024 边长）是
    /// T ≈ (891, 655)、CT ≈ (287, 307)。容差 40 像素，够容纳出生点散布。
    #[test]
    fn coords_spawn_points_land_on_radar() {
        let profile = parse_overview(MIRAGE, "de_mirage").unwrap();
        // 世界坐标由 cell_to_world 还原后实测（tick 72）：T 侧槽位 1、CT 侧槽位 6
        let (px, py) = profile.world_to_pixel(1378.0, -1487.0);
        assert!((px - 891.0).abs() < 40.0, "T 出生点 x 像素 {px}");
        assert!((py - 655.0).abs() < 40.0, "T 出生点 y 像素 {py}");
        let (px, py) = profile.world_to_pixel(-1970.5, 294.8);
        assert!((px - 287.0).abs() < 40.0, "CT 出生点 x 像素 {px}");
        assert!((py - 307.0).abs() < 40.0, "CT 出生点 y 像素 {py}");
    }
}
