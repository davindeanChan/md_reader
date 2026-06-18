# MD 阅读器

一个用 Rust + [egui](https://github.com/emilk/egui) 编写的轻量 Markdown 阅读器，专为 Windows 桌面阅读体验优化。体积小、启动快、零运行时依赖。

## ✨ 功能特性

- **完整 Markdown 渲染**：标题、粗体/斜体/删除线、列表、表格、代码块（带语法高亮）、链接、图片、任务列表
- **编码自动识别**：UTF-8 / UTF-8 BOM / GBK 自动探测，中文 Windows 下乱码或读取失败不再是问题
- **TOC 大纲侧边栏**：自动提取 H1–H6 标题生成可折叠大纲，F9 切换
- **查找**：Ctrl+F 弹出查找条，实时统计匹配数量
- **缩放**：Ctrl + `=`/`-`/`0` 调整阅读字号（50%–300%）
- **主题**：Ctrl+L 一键切换亮/暗主题
- **文件自动刷新**：外部编辑器保存后自动重载（800ms 轮询 mtime）
- **拖拽打开**：直接把 `.md` 文件拖进窗口即可
- **最近文件**：保留最近 10 个文件，一键打开
- **偏好持久化**：主题、缩放、大纲开关、上次文件——关闭再打开全部恢复
- **阅读宽度优化**：超宽屏下限制最大行宽并居中，缓解长行阅读疲劳

## ⌨️ 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+O` | 打开文件 |
| `Ctrl+L` | 切换亮/暗主题 |
| `Ctrl+=` | 放大字号 |
| `Ctrl+-` | 缩小字号 |
| `Ctrl+0` | 重置缩放 |
| `Ctrl+F` | 查找 |
| `F9` | 显示/隐藏大纲 |
| `Esc` | 关闭查找条 |

## 🛠️ 构建

需要 Rust 工具链（`rustup` 安装 stable 即可）。

```bash
cargo build --release
```

产物位于 `target/release/md-reader.exe`。

体积优化已配置（`Cargo.toml` 中 `[profile.release]`）：`opt-level="z"` + `lTO` + `strip` + `panic="abort"`。

## 🔗 关联为 .md 默认打开程序

以 PowerShell 运行仓库内的 `register.ps1`（仅写当前用户注册表，无需管理员权限）：

```powershell
.\register.ps1
```

它会将 `.md` / `.markdown` 关联到本程序，并出现在"打开方式"列表中。若右键菜单未立刻刷新，重启资源管理器即可：

```powershell
Stop-Process -Name explorer -Force; Start-Process explorer
```

## 📂 项目结构

```
src/
├── main.rs      # 入口：窗口、图标、中文字体加载、命令行参数解析
├── app.rs       # 应用主体：UI 布局、快捷键、状态管理
├── renderer.rs  # Markdown 渲染：缩放、阅读宽度限制与居中
├── file_ops.rs  # 文件读取（编码探测）、对话框、mtime 查询
├── toc.rs       # 标题大纲提取
└── config.rs    # 用户偏好与最近文件的持久化
```

## 📝 配置文件位置

`%APPDATA%\md-reader\config.toml`（Windows）。删除该文件即可恢复全部默认设置。

## 📄 License

本项目基于 [MIT License](LICENSE) 开源，可自由使用、修改和分发，请保留原始版权声明。

## 🙏 致谢

- GUI 框架：[egui](https://github.com/emilk/egui)
- Markdown 解析：[pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark)
- 中文编码识别：[encoding_rs](https://github.com/hsivonen/encoding_rs)
