fn main() {
    // 仅 Windows 下嵌入 ICO 图标到 exe 资源
    #[cfg(target_os = "windows")]
    {
        let icon_path = std::path::Path::new("assets/icon.ico");
        if icon_path.exists() {
            let mut res = winresource::WindowsResource::new();
            res.set_icon("assets/icon.ico");
            if let Err(e) = res.compile() {
                eprintln!("警告: 嵌入图标失败: {}", e);
            }
        } else {
            eprintln!("警告: 未找到 assets/icon.ico，跳过图标嵌入");
        }
    }
}
