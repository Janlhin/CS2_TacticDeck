//! 标注本体、命令与手册（含撤销栈）

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 一条标注：挂在某个回合的某段 tick 上，可选绑定某名选手
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotation {
    /// 手册内唯一 id，由 [`Manual`] 递增分配
    pub id: u64,
    /// 所属回合（从 1 开始，与回放里的回合条对齐）
    pub round: u32,
    /// 起始 tick（含）
    pub start_tick: u32,
    /// 结束 tick（含），必须 ≥ `start_tick`
    pub end_tick: u32,
    /// 标注针对的槽位（1..=10），不填就是「全队」
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<u8>,
    /// 正文；空字符串也算合法（先画框，回头再写）
    pub text: String,
    /// 颜色，`#rrggbb`
    pub color: String,
}

impl Annotation {
    /// 是否覆盖某个 tick
    #[must_use]
    pub fn covers(&self, tick: u32) -> bool {
        (self.start_tick..=self.end_tick).contains(&tick)
    }
}

/// 一次编辑动作
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Command {
    /// 新增一条标注
    Add { annotation: Annotation },
    /// 改正文 / 颜色 / 区间（整体替换，id 不变）
    Update {
        before: Annotation,
        after: Annotation,
    },
    /// 删除一条标注
    Remove { annotation: Annotation },
}

/// 一本手册：某个 demo 上的全部标注
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manual {
    /// 对应的 demo 文件名或回放 id，空表示还没绑定
    pub source: String,
    pub annotations: Vec<Annotation>,
    next_id: u64,
}

impl Manual {
    /// 新建一本绑定到某份回放的手册
    #[must_use]
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            annotations: Vec::new(),
            next_id: 1,
        }
    }

    /// 取下一个可用 id（由 [`UndoStack::apply`] 内部调用）
    pub fn take_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// 当前覆盖某个 tick 的标注（按开始 tick 排序）
    #[must_use]
    pub fn at_tick(&self, tick: u32) -> Vec<&Annotation> {
        let mut hits: Vec<&Annotation> =
            self.annotations.iter().filter(|a| a.covers(tick)).collect();
        hits.sort_by_key(|a| (a.start_tick, a.id));
        hits
    }

    /// 落盘成 JSON（UTF-8、两空格缩进，方便人直接改）
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("建目录 {} 失败", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(self).context("手册序列化失败")?;
        std::fs::write(path, json).with_context(|| format!("写 {} 失败", path.display()))
    }

    /// 从 JSON 读回；不校验 tick 区间是否在回放范围内，那是上层结合 `replay.json` 的事
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text =
            std::fs::read_to_string(path).with_context(|| format!("读 {} 失败", path.display()))?;
        Self::from_json(&text)
    }

    /// 从 JSON 字符串解析
    pub fn from_json(text: &str) -> Result<Self> {
        let manual: Self = serde_json::from_str(text).context("手册 JSON 不合法")?;
        // next_id 落盘后可能落后于已有 id（手改过 JSON），这里往上抬到安全值
        let max_id = manual.annotations.iter().map(|a| a.id).max().unwrap_or(0);
        let mut manual = manual;
        manual.next_id = manual.next_id.max(max_id + 1);
        Ok(manual)
    }
}

/// 撤销栈：`done` 是可重放的历史，`undone` 是被撤销、还能重做的命令
#[derive(Debug, Default)]
pub struct UndoStack {
    done: Vec<Command>,
    undone: Vec<Command>,
}

impl UndoStack {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 已生效的命令条数
    #[must_use]
    pub fn len(&self) -> usize {
        self.done.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.done.is_empty()
    }

    /// 还能撤销几次
    #[must_use]
    pub fn undo_depth(&self) -> usize {
        self.done.len()
    }

    /// 还能重做几次
    #[must_use]
    pub fn redo_depth(&self) -> usize {
        self.undone.len()
    }

    /// 执行一条命令并记进历史；任何新命令都会清空重做栈
    pub fn apply(&mut self, manual: &mut Manual, command: Command) {
        // 新增命令里的 id 可能是占位的 0：先让手册分配真 id，再把「带真 id 的命令」记进历史，
        // 否则撤销时会按 id 对不上、删不掉刚才那条
        let command = match command {
            Command::Add { mut annotation } => {
                if annotation.id == 0 {
                    annotation.id = manual.take_id();
                } else {
                    manual.next_id = manual.next_id.max(annotation.id + 1);
                }
                Command::Add { annotation }
            }
            other => other,
        };
        apply_raw(manual, &command);
        self.done.push(command);
        self.undone.clear();
    }

    /// 撤销最近一条命令，返回是否真的撤销了
    pub fn undo(&mut self, manual: &mut Manual) -> bool {
        let Some(command) = self.done.pop() else {
            return false;
        };
        revert(manual, &command);
        self.undone.push(command);
        true
    }

    /// 重做最近撤销的命令
    pub fn redo(&mut self, manual: &mut Manual) -> bool {
        let Some(command) = self.undone.pop() else {
            return false;
        };
        apply_raw(manual, &command);
        self.done.push(command);
        true
    }

    /// 清空历史（切 demo 时用）
    pub fn clear(&mut self) {
        self.done.clear();
        self.undone.clear();
    }
}

fn apply_raw(manual: &mut Manual, command: &Command) {
    match command {
        Command::Add { annotation } => {
            manual.annotations.push(annotation.clone());
        }
        Command::Update { after, .. } => {
            if let Some(slot) = manual.annotations.iter_mut().find(|a| a.id == after.id) {
                *slot = after.clone();
            }
        }
        Command::Remove { annotation } => {
            manual.annotations.retain(|a| a.id != annotation.id);
        }
    }
}

fn revert(manual: &mut Manual, command: &Command) {
    match command {
        Command::Add { annotation } => {
            manual.annotations.retain(|a| a.id != annotation.id);
        }
        Command::Update { before, .. } => {
            if let Some(slot) = manual.annotations.iter_mut().find(|a| a.id == before.id) {
                *slot = before.clone();
            }
        }
        Command::Remove { annotation } => {
            manual.annotations.push(annotation.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Annotation, Command, Manual, UndoStack};

    fn ann(id: u64, text: &str) -> Annotation {
        Annotation {
            id,
            round: 1,
            start_tick: 100,
            end_tick: 200,
            slot: Some(3),
            text: text.to_string(),
            color: "#ff8844".to_string(),
        }
    }

    #[test]
    fn add_update_remove_with_undo_redo() {
        let mut manual = Manual::new("demo.dem");
        let mut stack = UndoStack::new();

        stack.apply(
            &mut manual,
            Command::Add {
                annotation: ann(0, "架枪"),
            },
        );
        assert_eq!(manual.annotations.len(), 1);
        assert_eq!(manual.annotations[0].id, 1);

        let before = manual.annotations[0].clone();
        let mut updated = before.clone();
        updated.text = "架枪 + 补烟".to_string();
        stack.apply(
            &mut manual,
            Command::Update {
                before,
                after: updated.clone(),
            },
        );
        assert_eq!(manual.annotations[0].text, "架枪 + 补烟");

        assert!(stack.undo(&mut manual));
        assert_eq!(manual.annotations[0].text, "架枪");
        assert!(stack.redo(&mut manual));
        assert_eq!(manual.annotations[0].text, "架枪 + 补烟");

        let removed = manual.annotations[0].clone();
        stack.apply(
            &mut manual,
            Command::Remove {
                annotation: removed,
            },
        );
        assert!(manual.annotations.is_empty());

        assert!(stack.undo(&mut manual));
        assert_eq!(manual.annotations.len(), 1);
        assert_eq!(stack.undo_depth(), 2);
        assert_eq!(stack.redo_depth(), 1);
    }

    #[test]
    fn new_command_clears_redo_stack() {
        let mut manual = Manual::new("demo.dem");
        let mut stack = UndoStack::new();
        stack.apply(
            &mut manual,
            Command::Add {
                annotation: ann(0, "a"),
            },
        );
        assert!(stack.undo(&mut manual));
        assert_eq!(stack.redo_depth(), 1);

        stack.apply(
            &mut manual,
            Command::Add {
                annotation: ann(0, "b"),
            },
        );
        assert_eq!(stack.redo_depth(), 0, "新命令必须清掉重做栈");
        assert_eq!(manual.annotations.len(), 1);
    }

    #[test]
    fn queries_by_tick() {
        let mut manual = Manual::new("demo.dem");
        let mut stack = UndoStack::new();
        stack.apply(
            &mut manual,
            Command::Add {
                annotation: ann(0, "early"),
            },
        );
        let mut late = ann(0, "late");
        late.start_tick = 300;
        late.end_tick = 400;
        stack.apply(&mut manual, Command::Add { annotation: late });

        assert_eq!(manual.at_tick(150).len(), 1);
        assert_eq!(manual.at_tick(350).len(), 1);
        assert!(manual.at_tick(250).is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = std::env::temp_dir().join("tacticdeck-annotate-test");
        let manual = {
            let mut manual = Manual::new("demo.dem");
            manual.annotations.push(ann(1, "回防"));
            manual
        };
        let path = dir.join("demo.json");
        manual.save(&path).unwrap();
        let loaded = Manual::load(&path).unwrap();
        assert_eq!(
            loaded.annotations, manual.annotations,
            "落盘再读回，标注本体要一致"
        );
        assert!(loaded.next_id >= 2, "读回后的 next_id 要跳过已有 id");
    }

    #[test]
    fn from_json_raises_next_id_above_existing_ids() {
        let text = r##"{
          "source": "demo.dem",
          "annotations": [
            {"id": 7, "round": 2, "startTick": 10, "endTick": 20, "text": "手改", "color": "#ffffff"}
          ],
          "nextId": 1
        }"##;
        let manual = Manual::from_json(text).unwrap();
        assert_eq!(manual.annotations[0].id, 7);
        assert!(manual.next_id >= 8);
    }
}
