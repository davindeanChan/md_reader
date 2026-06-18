# MD 阅读器 - 文件关联注册脚本（当前用户，无需管理员权限）
# 运行方式：右键此文件 → "使用 PowerShell 运行"

$exePath = (Resolve-Path "$PSScriptRoot\target\release\md-reader.exe").Path
if (-not (Test-Path $exePath)) {
    Write-Host "错误: 未找到 $exePath，请先执行 cargo build --release" -ForegroundColor Red
    exit 1
}

Write-Host "将 MD 阅读器注册为 .md 文件处理程序..."
Write-Host "exe 路径: $exePath"

# ProgID 名称
$progId = "MDReader.File"

# 1. 注册应用程序能力（让"打开方式"对话框能发现此程序）
$appPath = "HKCU:\Software\Classes\Applications\md-reader.exe"
New-Item -Path $appPath -Force | Out-Null
Set-ItemProperty -Path $appPath -Name "FriendlyAppName" -Value "MD 阅读器"

$shellPath = "$appPath\shell\open\command"
New-Item -Path $shellPath -Force | Out-Null
Set-ItemProperty -Path $shellPath -Name "(default)" -Value "`"$exePath`" `"%1`""

# 2. 注册 ProgID（文件类型定义）
$progIdPath = "HKCU:\Software\Classes\$progId"
New-Item -Path $progIdPath -Force | Out-Null
Set-ItemProperty -Path $progIdPath -Name "(default)" -Value "Markdown 文档"

# 设置默认图标（使用 exe 内嵌的图标资源）
$defaultIconPath = "$progIdPath\DefaultIcon"
New-Item -Path $defaultIconPath -Force | Out-Null
Set-ItemProperty -Path $defaultIconPath -Name "(default)" -Value "`"$exePath`",0"

$progIdShellPath = "$progIdPath\shell\open\command"
New-Item -Path $progIdShellPath -Force | Out-Null
Set-ItemProperty -Path $progIdShellPath -Name "(default)" -Value "`"$exePath`" `"%1`""

# 3. 将 .md 扩展名关联到 ProgID
$mdPath = "HKCU:\Software\Classes\.md"
New-Item -Path $mdPath -Force | Out-Null
# 保留原有默认值（如果有的话），添加 OpenWithProgids
$openWithPath = "$mdPath\OpenWithProgids"
New-Item -Path $openWithPath -Force | Out-Null
New-ItemProperty -Path $openWithPath -Name $progId -Value ([byte[]]@()) -PropertyType None -Force | Out-Null

# 同样注册 .markdown 扩展名
$markdownPath = "HKCU:\Software\Classes\.markdown"
New-Item -Path $markdownPath -Force | Out-Null
$openWithMd = "$markdownPath\OpenWithProgids"
New-Item -Path $openWithMd -Force | Out-Null
New-ItemProperty -Path $openWithMd -Name $progId -Value ([byte[]]@()) -PropertyType None -Force | Out-Null

# 4. 设为默认打开程序（写入 HKCU 的 UserChoice 需要特殊处理，此处直接设置 Classes 默认值）
Set-ItemProperty -Path $mdPath -Name "(default)" -Value $progId
Set-ItemProperty -Path $markdownPath -Name "(default)" -Value $progId

Write-Host ""
Write-Host "注册完成！" -ForegroundColor Green
Write-Host ""
Write-Host "已注册以下扩展名："
Write-Host "  .md       → MD 阅读器"
Write-Host "  .markdown → MD 阅读器"
Write-Host ""
Write-Host "提示：如果右键打开方式仍未显示，请注销并重新登录，或运行以下命令刷新资源管理器："
Write-Host '  Stop-Process -Name explorer -Force; Start-Process explorer'
Write-Host ""
Write-Host "按任意键退出..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
