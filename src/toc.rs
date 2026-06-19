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
    /// 该标题在源文本中的起始字节偏移，用于按标题切分正文分段渲染。
    pub byte_start: usize,
}

/// 从 Markdown 文本提取标题列表
pub fn extract_headings(markdown: &str) -> Vec<Heading> {
    let mut out = Vec::new();
    let mut in_fence = false;
    let mut fence_marker: String = String::new();
    // 当前已消费到的字节偏移（即下一行起始位置）
    let mut byte_pos: usize = 0;

    for line in markdown.lines() {
        // 记录本行起始字节偏移后再推进（lines() 不含行尾 \n）
        let line_start = byte_pos;
        byte_pos += line.len() + 1;

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
            out.push(Heading {
                level,
                text,
                byte_start: line_start,
            });
        }
    }
    out
}

/// 收集 Markdown 中的链接引用定义（reference link definitions）。
///
/// 形如 `[label]: destination "可选标题"` 的行。这些定义在
/// [`egui_commonmark`] 内部由 pulldown-cmark 按单段解析，跨段不可见；
/// 当正文按标题分段渲染时，需要把这些定义注入到每一段，避免参考式链接
/// `[text][label]` 因所在段缺少定义而失效。
///
/// 实现为轻量逐行扫描，不引入额外依赖。返回原始定义文本块（每行一条，
/// 末尾带空行），可直接拼接到每段文本前。
pub fn extract_link_definitions(markdown: &str) -> String {
    let mut defs = String::new();
    let mut in_fence = false;
    let mut fence_marker: String = String::new();

    for line in markdown.lines() {
        let trimmed = line.trim_start();

        // 代码围栏开关：与标题扫描保持一致的围栏忽略逻辑
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

        // 形如：   [label]: url "optional title"
        // label 不含未转义的 ]；url 紧跟冒号与空白
        if let Some(rest) = is_link_definition(trimmed) {
            defs.push_str(rest);
            defs.push('\n');
        }
    }
    defs
}

/// 判断一行是否为链接引用定义。若是，返回原始行（含换行由调用方加）。
fn is_link_definition(line: &str) -> Option<&str> {
    // 最多 3 个前导空格（CommonMark 规范）
    let mut s = line;
    let mut leading = 0;
    while leading < 3 && s.starts_with(' ') {
        s = &s[1..];
        leading += 1;
    }
    let s = s.strip_prefix('[')?;
    // 找到闭合 ]（不允许跨行，这里不处理转义的复杂情况，取最常见情形）
    let close = s.find(']')?;
    let label = &s[..close];
    if label.is_empty() {
        return None;
    }
    let after = &s[close + 1..];
    let after = after.strip_prefix(':')?;
    let after = after.trim_start();
    if after.is_empty() {
        return None;
    }
    // destination 必须以非空白字符开头
    Some(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_headings_with_byte_offsets() {
        let md = "intro\n\n# One\n\ntext\n## Two\n";
        let hs = extract_headings(md);
        assert_eq!(hs.len(), 2);
        assert_eq!(hs[0].text, "One");
        assert_eq!(&md[hs[0].byte_start..hs[0].byte_start + 2], "# ");
        assert_eq!(hs[1].text, "Two");
        assert_eq!(&md[hs[1].byte_start..hs[1].byte_start + 3], "## ");
    }

    #[test]
    fn ignores_hash_in_code_fence() {
        let md = "```\n# not a heading\n```\n# real\n";
        let hs = extract_headings(md);
        assert_eq!(hs.len(), 1);
        assert_eq!(hs[0].text, "real");
    }

    #[test]
    fn collects_link_definitions() {
        let md = "[a]: http://x.com\n[b]: http://y.com \"title\"\n\ntext [link][a]\n";
        let defs = extract_link_definitions(md);
        assert!(defs.contains("[a]: http://x.com"));
        assert!(defs.contains("[b]: http://y.com"));
    }

    #[test]
    fn ignores_definition_like_lines_in_fence() {
        let md = "```\n[x]: not a def\n```\n[y]: http://z.com\n";
        let defs = extract_link_definitions(md);
        assert!(defs.contains("[y]:"));
        assert!(!defs.contains("[x]:"));
    }
}
