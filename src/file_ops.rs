use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// 打开原生文件选择对话框，返回选中的 .md 文件路径
pub fn open_file_dialog() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("选择 Markdown 文件")
        .add_filter("Markdown 文件", &["md", "markdown", "mdown", "mkd"])
        .add_filter("所有文件", &["*"])
        .pick_file()
}

/// 读取 Markdown 文件内容。
///
/// 编码探测顺序：
/// 1. 去除 UTF-8 BOM 后按 UTF-8 严格解码；
/// 2. 失败则按 GBK 解码（中文 Windows 常见编码）；
/// 3. 仍失败则按 UTF-8 lossy 兜底，保证用户至少能看到内容而不是"点了没反应"。
pub fn read_markdown_file(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("读取文件失败: {}", e))?;

    // 去掉 UTF-8 BOM
    let bytes = strip_utf8_bom(&bytes);

    // 1) UTF-8 严格
    if let Ok(s) = std::str::from_utf8(bytes) {
        return Ok(s.to_string());
    }
    // 2) GBK / GB18030
    {
        let (cow, _, had_errors) = encoding_rs::GBK.decode(bytes);
        if !had_errors {
            return Ok(cow.into_owned());
        }
    }
    // 3) 兜底：UTF-8 lossy
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

/// 去掉 UTF-8 BOM（EF BB BF），返回可能被裁剪后的字节切片
fn strip_utf8_bom(bytes: &[u8]) -> &[u8] {
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        &bytes[3..]
    } else {
        bytes
    }
}

/// 判断文件是否为 Markdown 文件
pub fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_lowercase().as_str(), "md" | "markdown" | "mdown" | "mkd"))
        .unwrap_or(false)
}

/// 获取文件最后修改时间（用于自动刷新检测）。文件不可达时返回 None。
pub fn file_mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}
