//! 标注模型：一条标注长什么样、编辑动作怎么记、手册怎么落盘。
//!
//! 设计取向是「命令即历史」：所有编辑都走 [`Command`]，撤销栈就是命令数组倒着放，
//! 这样撤销/重做、导入导出、协作合并都只需要处理同一套结构。

mod manual;

pub use manual::{Annotation, Command, Manual, UndoStack};
