// 在 release 模式下使用 windows 子系统，避免双击打开文件时弹出黑色控制台窗口
// （像 notepad.exe 那样直接打开，不带终端）。debug 模式保留 console 以便查看 eprintln! 调试输出。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod file_ops;
mod renderer;
mod toc;

use eframe::NativeOptions;
use std::path::PathBuf;

/// 嵌入图标 PNG 数据到二进制文件中
const ICON_PNG: &[u8] = include_bytes!("../assets/icon_256.png");

fn main() -> eframe::Result {
    // 解析命令行参数：支持 `md-reader.exe <file.md>` 方式打开文件
    let initial_file: Option<PathBuf> = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .filter(|p| p.exists() && file_ops::is_markdown_file(p));

    // 解码图标 PNG 为 RGBA 像素数据
    let icon = load_icon();

    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title("MD 阅读器")
        .with_inner_size([900.0, 700.0])
        .with_min_inner_size([400.0, 300.0])
        .with_drag_and_drop(true);

    if let Some(icon_data) = icon {
        viewport = viewport.with_icon(icon_data);
    }

    let options = NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "MD 阅读器",
        options,
        Box::new(|cc| {
            setup_chinese_font(&cc.egui_ctx);
            Ok(Box::new(app::MdReaderApp::new(initial_file)))
        }),
    )
}

/// 加载中文字体（微软雅黑），使 egui 能正确显示中文
fn setup_chinese_font(ctx: &eframe::egui::Context) {
    let mut fonts = eframe::egui::FontDefinitions::default();

    // 尝试加载系统中文字体
    let font_paths = [
        "C:\\Windows\\Fonts\\msyh.ttc",   // 微软雅黑
        "C:\\Windows\\Fonts\\msyhbd.ttc", // 微软雅黑粗体
        "C:\\Windows\\Fonts\\simhei.ttf", // 黑体
        "C:\\Windows\\Fonts\\simsun.ttc", // 宋体
    ];

    let mut loaded = false;
    for path in &font_paths {
        if let Ok(font_data) = std::fs::read(path) {
            let name = "chinese";
            fonts.font_data.insert(
                name.to_owned(),
                eframe::egui::FontData::from_owned(font_data).into(),
            );
            fonts
                .families
                .entry(eframe::egui::FontFamily::Proportional)
                .or_default()
                .insert(0, name.to_owned());
            fonts
                .families
                .entry(eframe::egui::FontFamily::Monospace)
                .or_default()
                .push(name.to_owned());
            loaded = true;
            eprintln!("已加载中文字体: {}", path);
            break;
        }
    }

    if !loaded {
        eprintln!("警告: 未找到系统中文字体，中文可能显示为方块");
    }

    ctx.set_fonts(fonts);
}

/// 从嵌入的 PNG 数据解码图标
fn load_icon() -> Option<eframe::egui::IconData> {
    use image::GenericImageView;
    let img = image::load_from_memory(ICON_PNG).ok()?;
    let (w, h) = img.dimensions();
    let rgba = img.to_rgba8().into_raw();
    Some(eframe::egui::IconData {
        rgba,
        width: w,
        height: h,
    })
}
