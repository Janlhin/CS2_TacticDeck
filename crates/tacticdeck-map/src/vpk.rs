//! 最小可用的 Valve VPK 目录解析：只做「按路径取出一段字节」这件事。
//!
//! VPK 结构（v1 / v2 都适用）：
//! - 文件头：签名 0x55AA1234、版本号、目录树长度；v2 再多 4 个 u32 段长度
//! - 目录树：扩展名 / 目录 / 文件名逐层以空串分隔，每个条目 18 字节
//! - 条目数据：archive index 为 0x7FFF 时在本体文件里，否则在 `pak01_NNN.vpk`

use anyhow::{Context as _, Result};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

const SIGNATURE: u32 = 0x55aa_1234;
const DIR_ARCHIVE: u16 = 0x7fff;

#[derive(Clone, Debug)]
pub struct VpkEntry {
    pub extension: String,
    pub directory: String,
    pub name: String,
    pub archive_index: u16,
    pub offset: u32,
    pub length: u32,
    pub preload: Vec<u8>,
}

impl VpkEntry {
    pub fn full_name(&self) -> String {
        if self.directory.is_empty() {
            format!("{}.{}", self.name, self.extension)
        } else {
            format!("{}/{}.{}", self.directory, self.name, self.extension)
        }
    }
}

pub struct Vpk {
    dir_path: PathBuf,
    dir_bytes: Vec<u8>,
    entries: Vec<VpkEntry>,
}

fn read_string(bytes: &[u8], cursor: &mut usize, what: &str) -> Result<String> {
    let rest = bytes
        .get(*cursor..)
        .with_context(|| format!("VPK 目录树在 {what} 处截断"))?;
    let end = rest
        .iter()
        .position(|byte| *byte == 0)
        .with_context(|| format!("VPK 目录树的 {what} 没有结束符"))?;
    *cursor += end + 1;
    Ok(String::from_utf8(rest[..end].to_vec())
        .unwrap_or_else(|_| String::from_utf8_lossy(&rest[..end]).into_owned()))
}

impl Vpk {
    pub fn open(dir_path: &Path) -> Result<Self> {
        let mut file = File::open(dir_path)
            .with_context(|| format!("打开 VPK 索引 {} 失败", dir_path.display()))?;
        let mut dir_bytes = Vec::new();
        file.read_to_end(&mut dir_bytes)?;

        anyhow::ensure!(dir_bytes.len() >= 12, "VPK 文件太小，不是合法索引");
        let signature = u32::from_le_bytes(dir_bytes[0..4].try_into()?);
        anyhow::ensure!(signature == SIGNATURE, "VPK 签名不符：{signature:#x}");
        let version = u32::from_le_bytes(dir_bytes[4..8].try_into()?);
        anyhow::ensure!((1..=2).contains(&version), "不支持的 VPK 版本：{version}");

        // 目录树起点：v1 在 12 字节后，v2 在 28 字节后
        let mut cursor = if version == 1 { 12 } else { 28 };
        let mut entries = Vec::new();

        loop {
            let extension = read_string(&dir_bytes, &mut cursor, "扩展名")?;
            if extension.is_empty() {
                break;
            }
            loop {
                let directory = read_string(&dir_bytes, &mut cursor, "目录")?;
                if directory.is_empty() {
                    break;
                }
                loop {
                    let name = read_string(&dir_bytes, &mut cursor, "文件名")?;
                    if name.is_empty() {
                        break;
                    }
                    let header = dir_bytes
                        .get(cursor..cursor + 18)
                        .context("VPK 目录条目截断")?;
                    let archive_index = u16::from_le_bytes(header[6..8].try_into()?);
                    let offset = u32::from_le_bytes(header[8..12].try_into()?);
                    let length = u32::from_le_bytes(header[12..16].try_into()?);
                    let preload_len = u16::from_le_bytes(header[4..6].try_into()?) as usize;
                    cursor += 18;

                    let preload = dir_bytes
                        .get(cursor..cursor + preload_len)
                        .context("VPK 预载数据截断")?
                        .to_vec();
                    cursor += preload_len;

                    entries.push(VpkEntry {
                        extension: extension.clone(),
                        directory: directory.clone(),
                        name,
                        archive_index,
                        offset,
                        length,
                        preload,
                    });
                }
            }
        }

        Ok(Self {
            dir_path: dir_path.to_path_buf(),
            dir_bytes,
            entries,
        })
    }

    /// 找出某个目录下指定扩展名的全部条目（不区分大小写）
    pub fn find(&self, directory: &str, extension: &str) -> Vec<&VpkEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                entry.directory.eq_ignore_ascii_case(directory)
                    && entry.extension.eq_ignore_ascii_case(extension)
            })
            .collect()
    }

    /// 读取一个条目的完整字节（预载段 + 归档段）
    pub fn read(&self, entry: &VpkEntry) -> Result<Vec<u8>> {
        let mut out = entry.preload.clone();
        if entry.length > 0 {
            let mut body = Vec::with_capacity(entry.length as usize);
            if entry.archive_index == DIR_ARCHIVE {
                let start = entry.offset as usize;
                let end = start + entry.length as usize;
                let slice = self
                    .dir_bytes
                    .get(start..end)
                    .context("VPK 条目偏移越界（本体文件）")?;
                body.extend_from_slice(slice);
            } else {
                let archive = self.archive_path(entry.archive_index);
                let mut file = File::open(&archive)
                    .with_context(|| format!("打开归档 {} 失败", archive.display()))?;
                let mut offset_file = std::io::BufReader::new(&mut file);
                std::io::Seek::seek(
                    &mut offset_file,
                    std::io::SeekFrom::Start(u64::from(entry.offset)),
                )?;
                let mut chunk = vec![0u8; entry.length as usize];
                offset_file.read_exact(&mut chunk)?;
                body = chunk;
            }
            out.extend_from_slice(&body);
        }
        Ok(out)
    }

    fn archive_path(&self, index: u16) -> PathBuf {
        let base = self
            .dir_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("pak01_dir.vpk");
        let prefix = base
            .strip_suffix("_dir.vpk")
            .filter(|prefix| !prefix.is_empty())
            .unwrap_or("pak01");
        self.dir_path
            .with_file_name(format!("{prefix}_{index:03}.vpk"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// 拼一个只有一条目的最小 VPK v1，验证目录树解析与本体读取
    fn write_sample(path: &Path, payload: &[u8]) {
        let mut tree = Vec::new();
        tree.extend_from_slice(b"txt\0");
        tree.extend_from_slice(b"resource/overviews\0");
        tree.extend_from_slice(b"de_test\0");
        tree.extend_from_slice(&0u32.to_le_bytes());
        tree.extend_from_slice(&0u16.to_le_bytes());
        tree.extend_from_slice(&DIR_ARCHIVE.to_le_bytes());
        tree.extend_from_slice(&0u32.to_le_bytes());
        tree.extend_from_slice(&0u32.to_le_bytes());
        tree.extend_from_slice(&0xFFFFu16.to_le_bytes());
        tree.push(0);
        tree.push(0);
        tree.push(0);

        let data_start = (12 + tree.len()) as u32;
        let entry_at = tree.len() - 3 - 18;
        tree[entry_at + 8..entry_at + 12].copy_from_slice(&data_start.to_le_bytes());
        tree[entry_at + 12..entry_at + 16].copy_from_slice(&(payload.len() as u32).to_le_bytes());

        let mut file = File::create(path).unwrap();
        file.write_all(&SIGNATURE.to_le_bytes()).unwrap();
        file.write_all(&1u32.to_le_bytes()).unwrap();
        file.write_all(&(tree.len() as u32).to_le_bytes()).unwrap();
        file.write_all(&tree).unwrap();
        file.write_all(payload).unwrap();
    }

    #[test]
    fn reads_entry_from_dir_archive() {
        let dir = std::env::temp_dir().join(format!("tacticdeck-vpk-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let vpk_path = dir.join("pak01_dir.vpk");
        let payload = b"\"de_test\"\n{\n\t\"pos_x\"\t\"0\"\n}\n";
        write_sample(&vpk_path, payload);

        let vpk = Vpk::open(&vpk_path).unwrap();
        let found = vpk.find("resource/overviews", "txt");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].full_name(), "resource/overviews/de_test.txt");
        let data = vpk.read(found[0]).unwrap();
        assert_eq!(data, payload);

        std::fs::remove_dir_all(&dir).ok();
    }
}
