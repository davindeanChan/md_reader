//! 用户偏好与最近文件列表的持久化。
//!
//! 配置文件位于 `<用户配置目录>/md-reader/config.toml`，
//! Windows 下通常是 `%APPDATA%/md-reader/config.toml`。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 持久化的应用偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 是否暗色主题
    pub dark_mode: bool,
    /// 渲染缩放（1.0 = 默认）
    pub zoom: f32,
    /// 是否显示左侧 TOC 大纲
    pub show_toc: bool,
    /// 最近打开的文件（最多 10 条，最近的在前）
    #[serde(default)]
    pub recent_files: Vec<String>,
    /// 上次打开的文件（启动时自动恢复）
    #[serde(default)]
    pub last_file: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            dark_mode: true,
            zoom: 1.0,
            show_toc: true,
            recent_files: Vec::new(),
            last_file: None,
        }
    }
}

impl AppConfig {
    /// 配置目录（不存在则尝试创建）
    fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("md-reader"))
    }

    /// 配置文件完整路径
    fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("config.toml"))
    }

    /// 从磁盘加载配置；失败或不存在时返回默认值
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };
        match std::fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// 写入磁盘。失败仅打印到 stderr（配置丢失不应影响阅读）。
    pub fn save(&self) {
        let (Some(dir), Some(path)) = (Self::config_dir(), Self::config_path()) else {
            return;
        };
        if std::fs::create_dir_all(&dir).is_err() {
            return;
        }
        match toml::to_string_pretty(self) {
            Ok(text) => {
                if let Err(e) = std::fs::write(&path, text) {
                    eprintln!("保存配置失败: {}", e);
                }
            }
            Err(e) => eprintln!("序列化配置失败: {}", e),
        }
    }

    /// 记录一个最近打开的文件，置顶并去重，最多保留 10 条
    pub fn push_recent(&mut self, path: &Path) {
        let s = path.display().to_string();
        self.recent_files.retain(|p| p != &s);
        self.recent_files.insert(0, s);
        self.recent_files.truncate(10);
        self.last_file = Some(path.display().to_string());
    }
}
