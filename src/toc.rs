//! 从 Markdown 文本提取标题大纲，用于左侧 TOC 侧边栏。
//!
//! 实现采用轻量逐行扫描，不引入完整 pulldown-cmark 解析（避免与
//! egui_commonmark 内部解析重复、保持零状态）。仅识别 ATX 风格标题
//! （`# 标题`），忽略代码块内的 `#`，以保证对常见文档准确。

/// 单个大纲条目
#[derive(Debug, Clone)]
pub struct Heading {
    /// 层级：1~6
    pub level: u8,
    /// 标题纯文本（已去除前后 `#` 与空白）
    pub text: String,
}

/// 从 Markdown 文本提取标题列表
pub fn extract_headings(markdown: &str) -> Vec<Heading> {
    let mut out = Vec::new();
    let mut in_fence = false;
    let mut fence_marker: String = String::new();

    for line in markdown.lines() {
        let trimmed = line.trim_start();

        // 代码围栏开关：``` 或 ~~~（允许带长度后缀）
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let marker = &trimmed[..3];
            if !in_fence {
                in_fence = true;
                fence_marker = marker.to_string();
            } else if marker == fence_marker {
                in_fence = false;
                fence_marker.clear();
            }
            continue;
        }
        if in_fence {
            continue;
        }

        // ATX 标题：1~6 个 # 后跟空格或行尾
        let Some(rest) = trimmed.strip_prefix('#') else {
            continue;
        };
        let mut level: u8 = 1;
        let mut s = rest;
        while s.starts_with('#') && level < 6 {
            s = &s[1..];
            level += 1;
        }
        // # 后必须是空格或行尾，否则不算标题（如 #foo）
        if !s.is_empty() && !s.starts_with(' ') {
            continue;
        }
        let text = s.trim().trim_end_matches('#').trim().to_string();
        if !text.is_empty() {
            out.push(Heading { level, text });
        }
    }
    out
}
