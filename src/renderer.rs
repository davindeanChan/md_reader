use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

/// 阅读区的最大内容宽度（px）。超宽屏下限制行宽，避免一行过长导致阅读疲劳。
const MAX_CONTENT_WIDTH: f32 = 860.0;

/// 在垂直滚动区中渲染 Markdown 内容。
///
/// - `zoom`：整体缩放倍率（1.0 = 默认），通过放大所有文字样式字号实现。
/// - 内容宽度受 [`MAX_CONTENT_WIDTH`] 限制并居中。
pub fn render_markdown(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    text: &str,
    zoom: f32,
) {
    let scale = zoom.clamp(0.5, 3.0);

    // 应用缩放：把所有 text_style 的字号整体放大
    if (scale - 1.0).abs() > 1e-3 {
        let mut sizes = ui.style().text_styles.clone();
        for (_id, style) in sizes.iter_mut() {
            style.size *= scale;
        }
        ui.style_mut().text_styles = sizes;
    }

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
