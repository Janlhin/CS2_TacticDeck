//! 输出命名规则（纯函数）：同一输入恒定输出，不含时间戳、随机数或环境相关项。
//!
//! - 文件名含地图名、回合号、选手 slot；
//! - 路径非法字符（`< > : " / \ | ? *` 与控制字符）替换为下划线；
//! - slot 与回合号都补零到两位：`slot 01` … `slot 10`。不补零的话，字典序里
//!   `slot1` 会排在 `slot10` 前面，和数值序不一致；补零后两种顺序一致，排序无歧义。

/// 非法字符统一替换成这个字符
const ILLEGAL_REPLACEMENT: char = '_';

/// 把一段组件名清洗成安全的文件名字符串；清洗结果对同一输入恒定
fn sanitize_component(raw: &str) -> String {
    let mut out: String = raw
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => ILLEGAL_REPLACEMENT,
            c if c.is_control() => ILLEGAL_REPLACEMENT,
            c => c,
        })
        .collect();
    // 连续下划线压成一个，避免斜杠 / 冒号串起来后留下多段下划线
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    let trimmed = out.trim_matches(ILLEGAL_REPLACEMENT).trim().to_string();
    if trimmed.is_empty() {
        "unknown".to_string()
    } else {
        trimmed
    }
}

/// 视频文件名：`<map>_round<NN>_slot<NN>.mp4`
#[must_use]
pub fn video_file_name(map: &str, round: u32, slot: u8) -> String {
    let map = sanitize_component(map);
    format!("{map}_round{round:02}_slot{slot:02}.mp4")
}

/// 回合级 tick 元数据文件名：`<map>_round<NN>_meta.json`
///
/// 元数据是一回合一份、内含全部轨，所以文件名不带 slot。
#[must_use]
pub fn metadata_file_name(map: &str, round: u32) -> String {
    let map = sanitize_component(map);
    format!("{map}_round{round:02}_meta.json")
}

#[cfg(test)]
mod tests {
    use super::{metadata_file_name, video_file_name};

    #[test]
    fn same_input_produces_same_output() {
        let a = video_file_name("de_mirage", 4, 3);
        let b = video_file_name("de_mirage", 4, 3);
        assert_eq!(a, b);
        let c = metadata_file_name("de_mirage", 4);
        let d = metadata_file_name("de_mirage", 4);
        assert_eq!(c, d);
    }

    #[test]
    fn slot_zero_padding_keeps_numeric_and_lexical_order() {
        let slot1 = video_file_name("de_mirage", 4, 1);
        let slot10 = video_file_name("de_mirage", 4, 10);
        assert!(slot1 < slot10, "补零后 slot1 应排在 slot10 之前");
        assert!(slot1.contains("slot01"));
        assert!(slot10.contains("slot10"));
    }

    #[test]
    fn illegal_path_chars_are_replaced() {
        let name = video_file_name("de/mirage:ct?|x", 2, 8);
        for illegal in ['<', '>', ':', '"', '/', '\\', '|', '?', '*'] {
            assert!(!name.contains(illegal), "{illegal} 不该出现在文件名里");
        }
        assert!(name.ends_with(".mp4"));
    }

    #[test]
    fn empty_map_falls_back_to_unknown() {
        assert_eq!(metadata_file_name("   ", 0), "unknown_round00_meta.json");
    }

    #[test]
    fn file_names_carry_map_round_and_slot() {
        let name = video_file_name("de_nuke", 7, 6);
        assert!(name.starts_with("de_nuke_round07_slot06"));
        assert_eq!(
            metadata_file_name("de_nuke", 7),
            "de_nuke_round07_meta.json"
        );
    }
}
