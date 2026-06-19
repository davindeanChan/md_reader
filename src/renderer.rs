use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::toc;

/// 阅读区的最大内容宽度（px）。超宽屏下限制行宽，避免一行过长导致阅读疲劳。
const MAX_CONTENT_WIDTH: f32 = 860.0;

/// 在垂直滚动区中渲染 Markdown 内容，并支持「点击大纲标题 → 滚动到对应段落」。
///
/// - `scroll_target`：`Some(i)` 表示把第 `i` 个标题（在 [`toc::extract_headings`]
///   返回列表中的下标）所在段落滚动到视口顶部。
/// - 实现按标题（`byte_start`）把正文切成多段，逐段调用
///   [`CommonMarkViewer::show`] 渲染；每段文本前注入全文的链接引用定义
///   （[`toc::extract_link_definitions`]），避免参考式链接因分段而失效。
/// - 文档无标题时退化为整篇渲染（与未启用该功能前一致）。
///
/// 返回值：是否消费了跳转目标（即 `scroll_target` 命中了一个真实段落）。
pub fn render_markdown_with_toc(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    text: &str,
    zoom: f32,
    scroll_target: Option<usize>,
) -> bool {
    let scale = zoom.clamp(0.5, 3.0);

    if (scale - 1.0).abs() > 1e-3 {
        // 创建一份放大的 text_styles，通过 scope_builder 隔离到子 UI，
        // 确保父 UI 的 style 不被修改，避免影响同层其他面板的样式。
        let mut zoomed_sizes = ui.style().text_styles.clone();
        for (_id, style) in zoomed_sizes.iter_mut() {
            style.size *= scale;
        }

        let mut zoomed_style: egui::Style = (**ui.style()).clone();
        zoomed_style.text_styles = zoomed_sizes;

        let mut consumed = false;
        ui.scope_builder(egui::UiBuilder::new().style(zoomed_style), |ui| {
            consumed = render_content(ui, cache, text, scale, scroll_target);
        });
        consumed
    } else {
        render_content(ui, cache, text, scale, scroll_target)
    }
}

/// 实际的滚动 + 居中渲染逻辑。返回是否消费了 `scroll_target`。
fn render_content(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    text: &str,
    scale: f32,
    scroll_target: Option<usize>,
) -> bool {
    let headings = toc::extract_headings(text);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(8.0);

            // 限制最大宽度并水平居中
            let available = ui.available_width();
            let content_w = available.min(MAX_CONTENT_WIDTH * scale);
            let indent = ((available - content_w) / 2.0).max(0.0);

            let consumed = ui
                .horizontal(|ui| {
                    if indent > 0.0 {
                        ui.add_space(indent);
                    }
                    ui.vertical(|ui| {
                        ui.set_max_width(content_w);

                        if headings.is_empty() {
                            // 无标题：整篇渲染（与启用该功能前一致）
                            CommonMarkViewer::new().show(ui, cache, text);
                            false
                        } else {
                            render_segments(
                                ui,
                                cache,
                                text,
                                &headings,
                                scroll_target,
                            )
                        }
                    })
                    .inner
                })
                .inner;

            ui.add_space(8.0);
            consumed
        })
        .inner
}

/// 按标题分段渲染正文，并对 `scroll_target` 对应的段在渲染前触发滚动。
///
/// 返回是否消费了跳转目标。
fn render_segments(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    text: &str,
    headings: &[toc::Heading],
    scroll_target: Option<usize>,
) -> bool {
    // 注入全文链接引用定义，保证每段都能解析参考式链接
    let defs = toc::extract_link_definitions(text);

    // 按标题 byte_start 切段：
    //   seg 0 = 首个标题之前的「前言」
    //   seg (i+1) = 以第 i 个标题开头、到第 i+1 个标题之前的正文
    // 点击目标 i 对应 seg (i+1)。
    let mut starts: Vec<usize> = headings.iter().map(|h| h.byte_start).collect();
    starts.insert(0, 0); // 前言段从 0 开始
    let total = text.len();

    let mut consumed = false;
    for (seg_idx, &start) in starts.iter().enumerate() {
        let end = starts.get(seg_idx + 1).copied().unwrap_or(total);
        let segment = &text[start..end];

        // seg_idx == 0 是前言（无对应标题），不响应跳转；
        // seg_idx == k 对应 headings[k-1]。
        if scroll_target == Some(seg_idx.saturating_sub(1)) && seg_idx > 0 {
            // 把该段起始滚动到视口顶部，冒泡到本 ScrollArea。
            // 必须在该段渲染之前调用，作用于下一个 widget 位置。
            ui.scroll_to_cursor(Some(egui::Align::TOP));
            consumed = true;
        }

        // 每段文本 = 链接定义 + 段原文
        let seg_text = format!("{defs}\n{segment}");
        CommonMarkViewer::new().show(ui, cache, &seg_text);
    }
    consumed
}
