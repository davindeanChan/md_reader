use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

use eframe::egui;
use egui_commonmark::CommonMarkCache;

use crate::config::AppConfig;
use crate::file_ops;
use crate::renderer;
use crate::toc;

pub struct MdReaderApp {
    /// 当前 Markdown 文本内容
    markdown_text: String,
    /// 缓存的源文本字数（避免每帧重算）
    char_count: usize,
    /// 缓存的源文本行数
    line_count: usize,
    /// 缓存的标题大纲（仅当文本变化时重算）
    headings: Vec<toc::Heading>,
    /// 用于检测文本是否变化、决定是否重算 outline
    text_version: u64,

    /// 当前打开的文件路径
    file_path: Option<PathBuf>,
    /// Markdown 渲染缓存
    cache: CommonMarkCache,

    /// 是否使用暗色主题
    dark_mode: bool,
    /// 渲染缩放
    zoom: f32,
    /// 是否显示左侧 TOC
    show_toc: bool,

    /// 上一帧的主题（用于避免每帧 set_visuals）
    last_dark_mode: Option<bool>,

    /// 持久化配置
    config: AppConfig,

    /// 错误提示（显示在状态栏），None 表示无错误
    error_msg: Option<String>,
    /// 错误提示的过期时刻（错误显示几秒后自动清除）
    error_until: Option<Instant>,

    /// 文件自动刷新：上次记录的 mtime
    last_mtime: Option<SystemTime>,
    /// 文件自动刷新：上次检查时间
    last_check: Instant,

    /// 查找功能
    find_open: bool,
    find_query: String,
}

impl MdReaderApp {
    pub fn new(initial_file: Option<PathBuf>) -> Self {
        let config = AppConfig::load();

        let welcome = "# 欢迎使用 MD 阅读器\n\n点击左上角 **打开** 按钮选择一个 `.md` 文件，\n或将 `.md` 文件直接拖拽到本窗口中。\n\n快捷键：**Ctrl+O** 打开 · **Ctrl+L** 切换主题 · **Ctrl+=/-** 缩放 · **Ctrl+F** 查找 · **F9** 大纲\n\n---\n\n## 支持的格式\n\n- 标题（H1 ~ H6）\n- **粗体**、*斜体*、~~删除线~~\n- 有序列表和无序列表\n- 表格\n- 代码块\n- 链接和图片\n- 任务列表\n";

        let mut app = Self {
            char_count: welcome.chars().count(),
            line_count: welcome.lines().count(),
            headings: toc::extract_headings(welcome),
            text_version: 0,
            markdown_text: welcome.to_string(),
            file_path: None,
            cache: CommonMarkCache::default(),
            dark_mode: config.dark_mode,
            zoom: config.zoom,
            show_toc: config.show_toc,
            last_dark_mode: None,
            config,
            error_msg: None,
            error_until: None,
            last_mtime: None,
            last_check: Instant::now(),
            find_open: false,
            find_query: String::new(),
        };

        // 启动时恢复：命令行参数优先，否则尝试恢复上次文件
        if let Some(path) = initial_file {
            app.load_file(path);
        } else if let Some(last) = app.config.last_file.clone() {
            let p = PathBuf::from(&last);
            if p.exists() && file_ops::is_markdown_file(&p) {
                app.load_file(p);
            }
        }
        app
    }

    /// 用新文本替换当前内容，并刷新统计/大纲缓存
    fn set_text(&mut self, text: String) {
        self.char_count = text.chars().count();
        self.line_count = text.lines().count();
        self.headings = toc::extract_headings(&text);
        self.text_version = self.text_version.wrapping_add(1);
        self.markdown_text = text;
    }

    /// 加载指定路径的 Markdown 文件
    fn load_file(&mut self, path: PathBuf) {
        match file_ops::read_markdown_file(&path) {
            Ok(text) => {
                self.set_text(text);
                self.file_path = Some(path.clone());
                self.cache = CommonMarkCache::default();
                self.last_mtime = file_ops::file_mtime(&path);
                self.config.push_recent(&path);
                self.config.save();
                self.clear_error();
            }
            Err(e) => {
                self.show_error(e);
            }
        }
    }

    fn show_error(&mut self, msg: String) {
        self.error_msg = Some(msg);
        self.error_until = Some(Instant::now() + Duration::from_secs(5));
        eprintln!("{}", self.error_msg.as_ref().unwrap());
    }

    fn clear_error(&mut self) {
        self.error_msg = None;
        self.error_until = None;
    }

    /// 处理拖拽事件
    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        for file in dropped {
            if let Some(path) = &file.path {
                if file_ops::is_markdown_file(path) {
                    self.load_file(path.clone());
                    return;
                }
            }
        }
    }

    /// 检查当前文件是否在外部被修改，若修改则自动重载
    fn check_file_changed(&mut self) {
        let Some(path) = self.file_path.clone() else {
            return;
        };
        if self.last_check.elapsed() < Duration::from_millis(800) {
            return;
        }
        self.last_check = Instant::now();
        let Some(now_mtime) = file_ops::file_mtime(&path) else {
            return;
        };
        if self.last_mtime != Some(now_mtime) {
            // 重载
            if let Ok(text) = file_ops::read_markdown_file(&path) {
                self.set_text(text);
                self.cache = CommonMarkCache::default();
                self.last_mtime = Some(now_mtime);
            }
        }
    }

    /// 处理全局快捷键
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            // Ctrl+O 打开
            if i.key_pressed(egui::Key::O) && (i.modifiers.ctrl || i.modifiers.command) {
                if let Some(path) = file_ops::open_file_dialog() {
                    self.load_file(path);
                }
            }
            // Ctrl+L 切换主题
            if i.key_pressed(egui::Key::L) && (i.modifiers.ctrl || i.modifiers.command) {
                self.dark_mode = !self.dark_mode;
                self.config.dark_mode = self.dark_mode;
                self.config.save();
            }
            // Ctrl+F 查找
            if i.key_pressed(egui::Key::F) && (i.modifiers.ctrl || i.modifiers.command) {
                self.find_open = true;
            }
            // Ctrl+= / NumPad+ 放大
            if i.key_pressed(egui::Key::Equals) && (i.modifiers.ctrl || i.modifiers.command) {
                self.zoom = (self.zoom + 0.1).min(3.0);
                self.config.zoom = self.zoom;
                self.config.save();
            }
            // Ctrl+- 缩小
            if i.key_pressed(egui::Key::Minus) && (i.modifiers.ctrl || i.modifiers.command) {
                self.zoom = (self.zoom - 0.1).max(0.5);
                self.config.zoom = self.zoom;
                self.config.save();
            }
            // Ctrl+0 重置缩放
            if i.key_pressed(egui::Key::Num0) && (i.modifiers.ctrl || i.modifiers.command) {
                self.zoom = 1.0;
                self.config.zoom = self.zoom;
                self.config.save();
            }
            // F9 切换大纲
            if i.key_pressed(egui::Key::F9) {
                self.show_toc = !self.show_toc;
                self.config.show_toc = self.show_toc;
                self.config.save();
            }
            // Esc 关闭查找
            if i.key_pressed(egui::Key::Escape) && self.find_open {
                self.find_open = false;
            }
        });
    }

    /// 更新窗口标题
    fn update_title(&self, ctx: &egui::Context) {
        let title = match &self.file_path {
            Some(p) => {
                let name = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("未知文件");
                format!("{} - MD 阅读器", name)
            }
            None => "MD 阅读器".to_string(),
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }
}

impl eframe::App for MdReaderApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // 拖拽 / 快捷键 / 自动刷新
        self.handle_dropped_files(&ctx);
        self.handle_shortcuts(&ctx);
        self.check_file_changed();

        // 主题：仅在变化时切换，避免每帧重绘
        if self.last_dark_mode != Some(self.dark_mode) {
            ctx.set_visuals(if self.dark_mode {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            self.last_dark_mode = Some(self.dark_mode);
        }

        // 错误提示自动过期
        if let Some(until) = self.error_until {
            if Instant::now() > until {
                self.clear_error();
            }
        }

        // 窗口标题
        self.update_title(&ctx);

        // 顶部工具栏
        egui::Panel::top("toolbar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("📂 打开").clicked() {
                    if let Some(path) = file_ops::open_file_dialog() {
                        self.load_file(path);
                    }
                }

                // 最近文件下拉
                ui.menu_button("🕘 最近", |ui| {
                    if self.config.recent_files.is_empty() {
                        ui.label("(无)");
                    } else {
                        for f in self.config.recent_files.clone() {
                            if ui.button(&f).clicked() {
                                let p = PathBuf::from(&f);
                                if p.exists() {
                                    self.load_file(p);
                                } else {
                                    self.show_error(format!("文件不存在: {}", f));
                                }
                                ui.close();
                            }
                        }
                    }
                });

                ui.separator();

                // 显示当前文件名
                let file_name = self
                    .file_path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("未打开文件");
                ui.label(file_name);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(if self.dark_mode { "☀️ 亮色" } else { "🌙 暗色" })
                        .clicked()
                    {
                        self.dark_mode = !self.dark_mode;
                        self.config.dark_mode = self.dark_mode;
                        self.config.save();
                    }
                    if ui.button("🔎 查找").clicked() {
                        self.find_open = true;
                    }
                    if ui.button("☰ 大纲").clicked() {
                        self.show_toc = !self.show_toc;
                        self.config.show_toc = self.show_toc;
                        self.config.save();
                    }
                    ui.separator();
                    ui.label(format!("缩放 {:.0}%", self.zoom * 100.0));
                    if ui.button("➕").clicked() {
                        self.zoom = (self.zoom + 0.1).min(3.0);
                        self.config.zoom = self.zoom;
                        self.config.save();
                    }
                    if ui.button("➖").clicked() {
                        self.zoom = (self.zoom - 0.1).max(0.5);
                        self.config.zoom = self.zoom;
                        self.config.save();
                    }
                });
            });
        });

        // 查找条（条件显示）
        if self.find_open {
            egui::Panel::top("find_bar")
                .exact_size(34.0)
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("🔍");
                        let resp = ui.text_edit_singleline(&mut self.find_query);
                        resp.request_focus();
                        let hits = if self.find_query.is_empty() {
                            0
                        } else {
                            self.markdown_text
                                .to_lowercase()
                                .matches(&self.find_query.to_lowercase())
                                .count()
                        };
                        ui.label(format!("{} 处", hits));
                        if ui.button("✕").clicked() {
                            self.find_open = false;
                        }
                    });
                });
        }

        // 底部状态栏
        egui::Panel::bottom("statusbar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if let Some(err) = &self.error_msg {
                    ui.colored_label(egui::Color32::from_rgb(230, 120, 80), format!("⚠ {}", err));
                } else {
                    let path_str = self
                        .file_path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "无".to_string());
                    ui.label(format!("📄 {}", path_str));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{} 行 / {} 字", self.line_count, self.char_count));
                });
            });
        });

        // 左侧 TOC 大纲
        if self.show_toc {
            egui::Panel::left("toc")
                .resizable(true)
                .default_size(220.0)
                .size_range(120.0..=400.0)
                .show_inside(ui, |ui| {
                    ui.heading("大纲");
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if self.headings.is_empty() {
                                ui.label("(无标题)");
                            } else {
                                for h in &self.headings {
                                    let indent = (h.level.saturating_sub(1) as f32) * 14.0;
                                    ui.horizontal(|ui| {
                                        ui.add_space(indent);
                                        let label = format!("{} {}", marker(h.level), h.text);
                                        // 使用明确宽度 + 自动换行的 Selectable 按钮，
                                        // 避免长标题以其未换行宽度撑开 TOC panel frame，
                                        // 导致 central panel 起始位置错位而出现黑块。
                                        let max_w = ui.available_width().max(0.0);
                                        let btn =
                                            egui::Button::selectable(false, label.as_str()).wrap();
                                        if ui.add_sized([max_w, 0.0], btn).clicked() {
                                            ui.ctx().request_repaint();
                                        }
                                    });
                                }
                            }
                        });
                });
        }

        // 中央 Markdown 渲染区
        egui::CentralPanel::default().show_inside(ui, |ui| {
            renderer::render_markdown(ui, &mut self.cache, &self.markdown_text, self.zoom);
        });
    }
}

/// 根据标题层级返回一个可视化前缀符号
fn marker(level: u8) -> &'static str {
    match level {
        1 => "▍",
        2 => "▪",
        3 => "·",
        _ => "·",
    }
}
