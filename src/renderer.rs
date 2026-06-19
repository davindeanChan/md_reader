use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

/// 阅读区的最大内容宽度（px）。超宽屏下限制行宽，避免一行过长导致阅读疲劳。
const MAX_CONTENT_WIDTH: f32 = 860.0;

/// 在垂直滚动区中渲染 Markdown 内容。
///
/// - `zoom`：整体缩放倍率（1.0 = 默认），通过放大所有文字样式字号实现。
/// - 内容宽度受 [`MAX_CONTENT_WIDTH`] 限制并居中。
/// - 缩放通过 `scope_builder` 隔离为子 UI 样式，不污染父级样式，避免样式泄露导致
///   相邻面板（如 TOC 侧边栏）的分隔线/边框渲染异常。
pub fn render_markdown(ui: &mut egui::Ui, cache: &mut CommonMarkCache, text: &str, zoom: f32) {
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

        ui.scope_builder(egui::UiBuilder::new().style(zoomed_style), |ui| {
            render_content(ui, cache, text, scale);
        });
    } else {
        render_content(ui, cache, text, scale);
    }
}

/// 实际的滚动 + 居中渲染逻辑
fn render_content(ui: &mut egui::Ui, cache: &mut CommonMarkCache, text: &str, scale: f32) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(8.0);

            // 限制最大宽度并水平居中
            let available = ui.available_width();
            let content_w = available.min(MAX_CONTENT_WIDTH * scale);
            let indent = ((available - content_w) / 2.0).max(0.0);

            ui.horizontal(|ui| {
                if indent > 0.0 {
                    ui.add_space(indent);
                }
                ui.vertical(|ui| {
                    ui.set_max_width(content_w);
                    CommonMarkViewer::new().show(ui, cache, text);
                });
            });
            ui.add_space(8.0);
        });
}
