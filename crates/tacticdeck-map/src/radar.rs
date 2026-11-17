//! 雷达底图：从 VPK 里的 `*.vtex_c` 取出像素，解成 PNG。
//!
//! VTEX 是 Source 2 的编译纹理：外面套一层资源块（RED2 / DATA），
//! DATA 块里才是真正纹理头（宽高、格式、mip 数），像素紧跟在 DATA 块之后。
//! 雷达图是 1024×1024 的 BC7，且被 LZ4 压过，所以要解两次。

use anyhow::{Context as _, Result, bail};
use std::path::Path;

use crate::map_profile::discover_cs2;
use crate::vpk::Vpk;

/// 底图在 VPK 里的目录
const RADAR_DIR: &str = "panorama/images/overheadmaps";
/// 资源文件头版本（`RED2` 那套）
const RESOURCE_HEADER_VERSION: u16 = 12;
/// 只认这两类块：DATA 是内容块，其余块（RED2 等）跳过
const BLOCK_DATA: u32 = 0x4154_4144; // "DATA"

pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct VtexHeader {
    pub width: u16,
    pub height: u16,
    pub format: u8,
    /// 像素数据在文件里的起点
    pub data_offset: usize,
}

/// 底图条目名（不带扩展名）：`de_mirage` → `de_mirage_radar_psd`
pub fn radar_texture_name(map: &str) -> String {
    format!("{map}_radar_psd")
}

/// 拆资源块表，返回 DATA 块的「起点 + 长度」
pub fn find_data_block(bytes: &[u8]) -> Result<(usize, usize)> {
    anyhow::ensure!(bytes.len() >= 16, "资源文件太短");
    let header_version = u16::from_le_bytes(bytes[4..6].try_into()?);
    anyhow::ensure!(
        header_version == RESOURCE_HEADER_VERSION,
        "资源头版本不对：{header_version}"
    );
    let block_offset = u32::from_le_bytes(bytes[8..12].try_into()?) as usize;
    let block_count = u32::from_le_bytes(bytes[12..16].try_into()?) as usize;
    // 块表起点：16 字节头 + 两个 u32（块偏移 / 块数）之后的偏移
    let table = 16 + block_offset.saturating_sub(8);
    for index in 0..block_count {
        let entry = table + index * 12;
        let kind = u32::from_le_bytes(
            bytes
                .get(entry..entry + 4)
                .context("块表截断")?
                .try_into()?,
        );
        let relative = u32::from_le_bytes(
            bytes
                .get(entry + 4..entry + 8)
                .context("块表截断")?
                .try_into()?,
        ) as usize;
        let size = u32::from_le_bytes(
            bytes
                .get(entry + 8..entry + 12)
                .context("块表截断")?
                .try_into()?,
        ) as usize;
        if kind == BLOCK_DATA {
            // 偏移是相对「读掉 type 之后的位置」算的
            return Ok((entry + 4 + relative, size));
        }
    }
    bail!("资源里没有 DATA 块")
}

/// 读 VTEX 头：宽高、格式、像素起点
pub fn parse_vtex_header(bytes: &[u8]) -> Result<VtexHeader> {
    let (data_start, data_size) = find_data_block(bytes)?;
    anyhow::ensure!(data_size >= 36, "DATA 块太小，不像 VTEX");
    let version = u16::from_le_bytes(
        bytes
            .get(data_start..data_start + 2)
            .context("VTEX 头截断")?
            .try_into()?,
    );
    anyhow::ensure!(version == 1, "VTEX 版本不支持：{version}");
    let width = u16::from_le_bytes(
        bytes
            .get(data_start + 20..data_start + 22)
            .context("VTEX 头截断")?
            .try_into()?,
    );
    let height = u16::from_le_bytes(
        bytes
            .get(data_start + 22..data_start + 24)
            .context("VTEX 头截断")?
            .try_into()?,
    );
    let format = *bytes.get(data_start + 26).context("VTEX 头截断")?;
    let mips = *bytes.get(data_start + 27).context("VTEX 头截断")?;
    anyhow::ensure!(mips == 1, "只支持单 mip 纹理，读到 {mips} 级");
    Ok(VtexHeader {
        width,
        height,
        format,
        data_offset: data_start + data_size,
    })
}

/// 每个 4×4 块要几个字节；`None` 表示不是块压缩格式
fn block_size(format: u8) -> Result<Option<usize>> {
    match format {
        1 => Ok(Some(8)),        // DXT1 / BC1
        2 => Ok(Some(16)),       // DXT5 / BC3
        20 | 22 => Ok(Some(16)), // BC7
        _ => Ok(None),
    }
}

fn bytes_per_pixel(format: u8) -> Result<usize> {
    match format {
        28 => Ok(4), // RGBA8888（本机 CS2 的雷达图用这个）
        other => bail!("不认识的 VTEX 像素格式：{other}"),
    }
}

fn mip0_size(width: u16, height: u16, format: u8) -> Result<usize> {
    let width = width as usize;
    let height = height as usize;
    match block_size(format)? {
        Some(block) => Ok(width.div_ceil(4) * height.div_ceil(4) * block),
        None => Ok(width * height * bytes_per_pixel(format)?),
    }
}

/// VTEX 字节 → RGBA 像素（只解 mip 0）
pub fn decode_vtex(bytes: &[u8]) -> Result<RgbaImage> {
    let header = parse_vtex_header(bytes)?;
    let expected = mip0_size(header.width, header.height, header.format)?;
    let tail = bytes
        .get(header.data_offset..)
        .context("像素数据起点越界")?;

    // 没压缩就是原样一段；压缩过就 LZ4 解到（略大于）预期长度再截断
    let pixels = if tail.len() >= expected {
        tail[..expected].to_vec()
    } else {
        // 引擎的块数据可能比 mip0 理论长度多出几十到几百字节，缓冲给两倍再截断
        let mut decoded = lz4_flex::decompress(tail, expected * 2).context("LZ4 解压雷达图失败")?;
        decoded.truncate(expected);
        decoded
    };

    let width = header.width as usize;
    let height = header.height as usize;
    let pitch = width * 4;

    let Some(block_bytes) = block_size(header.format)? else {
        // 已经是 RGBA8888：直接交出去
        anyhow::ensure!(pixels.len() == width * height * 4, "像素长度与宽高不匹配");
        return Ok(RgbaImage {
            width: header.width as u32,
            height: header.height as u32,
            rgba: pixels,
        });
    };

    let mut rgba = vec![0u8; width * height * 4];
    let mut block = vec![0u8; block_bytes];
    for block_y in 0..height.div_ceil(4) {
        for block_x in 0..width.div_ceil(4) {
            let index = block_y * width.div_ceil(4) + block_x;
            let source = pixels
                .get(index * block_bytes..(index + 1) * block_bytes)
                .context("块压缩数据不足")?;
            block.copy_from_slice(source);
            let mut target = vec![0u8; 4 * 4 * 4];
            match header.format {
                1 => bcdec_rs::bc1(&block, &mut target, 4 * 4),
                2 => bcdec_rs::bc3(&block, &mut target, 4 * 4),
                _ => bcdec_rs::bc7(&block, &mut target, 4 * 4),
            }
            // 把 4×4 的小方块贴回大图
            for row in 0..4 {
                let y = block_y * 4 + row;
                if y >= height {
                    break;
                }
                for col in 0..4 {
                    let x = block_x * 4 + col;
                    if x >= width {
                        break;
                    }
                    let from = (row * 4 + col) * 4;
                    let to = y * pitch + x * 4;
                    rgba[to..to + 4].copy_from_slice(&target[from..from + 4]);
                }
            }
        }
    }

    Ok(RgbaImage {
        width: header.width as u32,
        height: header.height as u32,
        rgba,
    })
}

/// RGBA 像素 → PNG 字节
pub fn encode_png(image: &RgbaImage) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, image.width, image.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().context("写 PNG 头失败")?;
        writer
            .write_image_data(&image.rgba)
            .context("写 PNG 像素失败")?;
    }
    Ok(out)
}

/// 从本机 CS2 的 VPK 里取出某张地图的雷达图并做成 PNG
pub fn radar_png(map: &str, cs2_dir: Option<&Path>) -> Result<Vec<u8>> {
    let cs2 = discover_cs2(cs2_dir)?;
    let vpk_path = cs2.join("pak01_dir.vpk");
    let vpk = Vpk::open(&vpk_path)?;
    let name = radar_texture_name(map);
    let entry = vpk
        .find(RADAR_DIR, "vtex_c")
        .into_iter()
        .find(|entry| entry.name.eq_ignore_ascii_case(&name))
        .with_context(|| format!("VPK 里没有 {RADAR_DIR}/{name}.vtex_c"))?;
    let bytes = vpk.read(entry)?;
    let image = decode_vtex(&bytes)?;
    encode_png(&image)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 拼一个只有 DATA 块的最小资源：资源头 16B + 块表 12B + VTEX 头 36B + 像素
    fn sample_resource(width: u16, height: u16, format: u8, payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&2048u32.to_le_bytes()); // 资源总长（这里不校验）
        out.extend_from_slice(&RESOURCE_HEADER_VERSION.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes()); // 资源类型版本
        out.extend_from_slice(&8u32.to_le_bytes()); // 块表偏移
        out.extend_from_slice(&1u32.to_le_bytes()); // 只有一个块

        // 块表条目：类型 + 相对偏移 + 长度。相对偏移从「读掉 type 之后」的位置算
        let entry_at = out.len();
        out.extend_from_slice(&BLOCK_DATA.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes()); // 先占位，后面回填
        out.extend_from_slice(&(40u32).to_le_bytes());

        let data_at = out.len();
        // VTEX 头：版本 1、flags 0、4 个浮点反射率、宽高深度、格式、单 mip
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&[0u8; 16]);
        out.extend_from_slice(&width.to_le_bytes());
        out.extend_from_slice(&height.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.push(format);
        out.push(1);
        out.extend_from_slice(&0u32.to_le_bytes()); // picmip
        out.extend_from_slice(&8u32.to_le_bytes()); // 额外数据偏移
        out.extend_from_slice(&0u32.to_le_bytes()); // 额外数据条数
        // 像素跟在 DATA 块后面：像素起点 = DATA 起点 + DATA 长度
        out.extend_from_slice(payload);

        let relative = data_at - (entry_at + 4);
        out[entry_at + 4..entry_at + 8].copy_from_slice(&(relative as u32).to_le_bytes());
        out
    }

    #[test]
    fn radar_asset_name_follows_valve_convention() {
        assert_eq!(radar_texture_name("de_mirage"), "de_mirage_radar_psd");
        assert_eq!(radar_texture_name("de_dust2"), "de_dust2_radar_psd");
    }

    #[test]
    fn parses_data_block_and_vtex_header() {
        let bytes = sample_resource(4, 4, 28, &[0x1fu8; 16]);
        let (start, size) = find_data_block(&bytes).unwrap();
        assert_eq!(size, 40, "DATA 块至少装下 40 字节 VTEX 头");
        let header = parse_vtex_header(&bytes).unwrap();
        assert_eq!(header.width, 4);
        assert_eq!(header.height, 4);
        assert_eq!(header.format, 28);
        assert_eq!(header.data_offset, start + size, "像素紧跟 DATA 块");
    }

    #[test]
    fn decodes_uncompressed_rgba_texture() {
        let payload = [
            10u8, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 100, 110, 120, 255,
        ];
        let bytes = sample_resource(2, 2, 28, &payload);
        let image = decode_vtex(&bytes).unwrap();
        assert_eq!(image.width, 2);
        assert_eq!(image.height, 2);
        assert_eq!(image.rgba, payload, "RGBA8888 应原样解出");
    }

    #[test]
    fn rejects_non_resource_bytes() {
        let junk = b"not a resource";
        assert!(parse_vtex_header(junk).is_err());
    }

    #[test]
    fn writes_a_readable_png() {
        let image = RgbaImage {
            width: 2,
            height: 2,
            rgba: vec![255; 2 * 2 * 4],
        };
        let png = encode_png(&image).unwrap();
        assert_eq!(&png[1..4], b"PNG", "应产出一个真的 PNG");
    }
}
