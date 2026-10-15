#Requires -Version 5.1
#********************************************************************
#FileName:      management.ps1
#Description:   DockBar 打包/安装/卸载/编译测试脚本
#Usage:         .\management.ps1 [build [版本号] | version <版本号> | install | uninstall |
#               dev | test | type-check | check | clippy | fmt-check | lint |
#               build-ui | build-lib | clean | -h|--help]
#Note:          首次运行被执行策略拦截时：
#               powershell -ExecutionPolicy Bypass -File .\management.ps1 <命令>
#
#               本项目为 Cargo workspace（src + libs/core / libs/utils /
#               libs/dockbar-ipc / libs/dockbar-macros），target 在仓库根目录：
#               - 版本号唯一来源为根 package.json（tauri.conf.json 通过
#                 "version": "../package.json" 引用），同步范围：
#                 package.json + package-lock.json（3 处）+ libs/core/deno.json
#                 + 5 个 Cargo.toml + Cargo.lock（5 个 dockbar* 包）
#               - NSIS 产物位于根 target\release\bundle\nsis
#               - 项目已无密钥化（无更新服务器、无签名密钥）：
#                 tauri build 单段式直接打包 NSIS，无需任何签名私钥
#               - Rust 工具链为 rust-toolchain.toml 锁定的 nightly，
#                 rustup 会自动应用，无需手动切换
#               - 发版流程仍走项目自带 npm run version:start / version:finish
#                 （含 changelog 与 git 提交），本脚本的 version 命令仅做
#                 静默全量同步（等价 versionish.ts ci，不产生提交）
#********************************************************************

param(
    [Parameter(Position = 0)][string]$Command = '',
    [Parameter(Position = 1)][string]$VersionArg = '',
    [switch]$Yes,
    # 兼容 -h / -help / --help 开关写法（PowerShell 会将连字符开头参数绑定为开关）
    [switch]$h,
    [switch]$help
)

# 连字符开关统一归一化为 help 命令
if ($h -or $help) { $Command = 'help' }

$ProjectDir     = $PSScriptRoot
$NsisBundleDir  = Join-Path $ProjectDir 'target\release\bundle\nsis'
$CoreDir        = Join-Path $ProjectDir 'libs\core'
$VersionPattern = '^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$'
$Utf8NoBom      = New-Object System.Text.UTF8Encoding($false)

# 参与版本同步的 crate 清单（根 Cargo.toml 无 version 字段，仅子 crate 有）
$CrateDirs  = @('src', 'libs\core', 'libs\utils', 'libs\dockbar-ipc', 'libs\dockbar-macros')
$CrateNames = @('dockbar', 'dockbar-core', 'dockbar-ipc', 'dockbar-macros', 'dockbar-utils')

# 状态输出（自动对齐到第 60 列，中文按 2 列宽计算）
function Get-DisplayWidth([string]$Text) {
    $width = 0
    foreach ($ch in $Text.ToCharArray()) {
        $code = [int]$ch
        if (($code -ge 0x2E80 -and $code -le 0x9FFF) -or
            ($code -ge 0xF900 -and $code -le 0xFAFF) -or
            ($code -ge 0xFF01 -and $code -le 0xFF60) -or
            ($code -ge 0xFFE0 -and $code -le 0xFFE6)) { $width += 2 } else { $width += 1 }
    }
    $width
}

function Write-Status {
    param([string]$Message, $State)
    $pad = [Math]::Max(1, 60 - (Get-DisplayWidth $Message))
    Write-Host -NoNewline ($Message + (' '* $pad) + '[')
    if ($State -eq 'success' -or $State -eq 0) {
        Write-Host -NoNewline ' OK ' -ForegroundColor Green
    } elseif ($State -eq 'failure' -or $State -eq 1) {
        Write-Host -NoNewline 'FAILED' -ForegroundColor Red
    } else {
        Write-Host -NoNewline 'WARNING' -ForegroundColor Yellow
    }
    Write-Host ']'
}

# 检查上一条原生命令的执行结果
function Assert-LastExitCode {
    param([string]$Message)
    if ($LASTEXITCODE -eq 0) {
        Write-Status "$Message 完成" 0
    } else {
        Write-Status "$Message 失败" 1
        exit 1
    }
}

# 时长格式化：秒 → "X 分 Y 秒" / "Y 秒"
function Format-Duration([double]$Seconds) {
    $total = [int][Math]::Round($Seconds)
    if ($total -ge 60) {
        "{0} 分 {1} 秒" -f ([int][Math]::Floor($total / 60)), ($total % 60)
    } else {
        "$total 秒"
    }
}

# 补充工具链 PATH / 环境变量：新开终端本可用，但部分会话（如从旧进程派生）
# 继承的是设置环境变量之前的旧 PATH，这里从注册表补齐缺失的目录
function Initialize-BuildEnvironment {
    $known = @($env:Path -split ';' | ForEach-Object { $_.Trim().TrimEnd('\') } | Where-Object { $_ })
    foreach ($scope in 'Machine', 'User') {
        $value = [Environment]::GetEnvironmentVariable('Path', $scope)
        if (-not $value) { continue }
        foreach ($dir in ($value -split ';')) {
            $dir = $dir.Trim().TrimEnd('\')
            if (-not $dir -or $known -contains $dir) { continue }
            if (Test-Path -LiteralPath $dir) {
                $env:Path = "$env:Path;$dir"
                $known += $dir
            }
        }
    }
    foreach ($name in 'CARGO_HOME', 'RUSTUP_HOME') {
        if (-not (Get-Item "Env:$name" -ErrorAction SilentlyContinue)) {
            $v = [Environment]::GetEnvironmentVariable($name, 'User')
            if (-not $v) { $v = [Environment]::GetEnvironmentVariable($name, 'Machine') }
            if ($v) { Set-Item -Path "Env:$name" -Value $v }
        }
    }

    foreach ($cmd in 'node', 'npm', 'cargo', 'rustc') {
        if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) {
            Write-Status "缺少依赖命令: $cmd" 1
            if ($cmd -in 'cargo', 'rustc') {
                Write-Host "      安装 Rust: https://rustup.rs （或 winget install Rustlang.Rustup）"
            } else {
                Write-Host "      安装 Node.js: https://nodejs.org （或 nvm-windows）"
            }
            exit 1
        }
    }

    # deno：质量检查（deno lint / deno fmt / npm run type-check 中的 deno check）
    # 与 libs/core 构建依赖；本机缺失时仅告警，不阻断与 deno 无关的命令
    if (-not (Get-Command deno -ErrorAction SilentlyContinue)) {
        Write-Status "未检测到 deno，lint/fmt/type-check/build-lib 将不可用" 2
        Write-Host "      安装: https://deno.com （或 winget install DenoLand.Deno）"
    }

    # WebView2 Runtime：Tauri 窗口渲染依赖
    $wv2Keys = @(
        'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}',
        'HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}',
        'HKCU:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
    )
    if (-not ($wv2Keys | Where-Object { (Get-ItemProperty $_ -ErrorAction SilentlyContinue).pv })) {
        Write-Status "未检测到 WebView2 Runtime，应用无法启动" 2
        Write-Host "      安装: https://developer.microsoft.com/microsoft-edge/webview2/"
    }

    # MSVC C++ 生成工具：Rust MSVC 工具链链接依赖
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (Test-Path -LiteralPath $vswhere) {
        $vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
        if (-not $vsPath) {
            Write-Status "未检测到 MSVC C++ 生成工具，Rust 构建会失败" 2
            Write-Host "      安装: Visual Studio Installer 勾选 '使用 C++ 的桌面开发'"
        }
    } else {
        Write-Status "未检测到 vswhere，无法确认 MSVC 工具链" 2
    }

    Set-CargoMirrorEnv
    $nodeV  = & node -v
    $cargoV = (& cargo -V) -replace '^cargo\s+', ''
    Write-Status "构建环境检查通过 (node $nodeV / cargo $cargoV)" 0
}

# 注入 Cargo 镜像参数（环境变量方式）
# Windows 下无法照搬 Linux 的 cargo 包装器方案：tauri CLI 通过 CreateProcess
# 直接查找 cargo.exe，不会经 cmd 解析 .cmd 包装脚本。改用 Cargo 的环境变量配置：
# 配置键的 . 和 - 都转成 _（如 source.crates-io.replace-with → CARGO_SOURCE_CRATES_IO_REPLACE_WITH），
# 优先级高于 CARGO_HOME 配置文件，且随进程传给 npm → tauri CLI → cargo 的整条调用链。
# 切换镜像时，注释当前启用的三行，并取消注释目标镜像的三行。
function Set-CargoMirrorEnv {
    # rsproxy 字节源（当前启用，与本机 CARGO_HOME config.toml 一致）
    $MirrorName  = 'rsproxy 字节源'
    $ReplaceWith = 'rsproxy'
    $RegistryUrl = 'sparse+https://rsproxy.cn/index/'

    # Nexus 内网源（Linux 构建机使用）
    # $MirrorName  = 'Nexus 内网源'
    # $ReplaceWith = 'nexus'
    # $RegistryUrl = 'sparse+http://nexus.huang.org/repository/cargo-proxy/'

    # 阿里云源
    # $MirrorName  = '阿里云源'
    # $ReplaceWith = 'aliyun'
    # $RegistryUrl = 'sparse+https://mirrors.aliyun.com/crates.io-index/'

    # 清华源
    # $MirrorName  = '清华源'
    # $ReplaceWith = 'tuna'
    # $RegistryUrl = 'sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/'

    # 中科大源
    # $MirrorName  = '中科大源'
    # $ReplaceWith = 'ustc'
    # $RegistryUrl = 'sparse+https://mirrors.ustc.edu.cn/crates.io-index/'

    if (-not $ReplaceWith -or -not $RegistryUrl) {
        Write-Status "未启用任何 Cargo 镜像源，请取消注释 Set-CargoMirrorEnv 中的目标镜像" 1
        exit 1
    }

    $env:CARGO_SOURCE_CRATES_IO_REPLACE_WITH = $ReplaceWith
    Set-Item -Path ("Env:\CARGO_SOURCE_" + $ReplaceWith.ToUpper() + "_REGISTRY") -Value $RegistryUrl
    Write-Status "已注入 Cargo 镜像参数（$MirrorName）" 0
}

# 定位 Tauri CLI：优先 cargo install tauri-cli 提供的 cargo-tauri / tauri，
# 否则回退到 npx tauri（npx 优先复用本仓库 node_modules 内的 @tauri-apps/cli，
# 不存在时才自动下载缓存）
function Get-TauriCli {
    if (Get-Command cargo-tauri -ErrorAction SilentlyContinue) {
        return @('cargo', 'tauri')
    }
    if (Get-Command tauri -ErrorAction SilentlyContinue) {
        return @('tauri')
    }
    return @('npx', 'tauri')
}

# 统一调用 Tauri CLI：Get-TauriCli 返回 "命令 + 前置参数" 数组，
# 这里展开后追加子命令（注意 PowerShell 单元素数组的 1..0 是降序切片，必须判空）
function Invoke-TauriCli {
    param([string[]]$TauriArgs)
    $cli  = Get-TauriCli
    $head = $cli[0]
    $rest = @()
    if ($cli.Count -gt 1) { $rest = @($cli[1..($cli.Count - 1)]) }
    & $head @rest @TauriArgs
}

# 读取当前应用版本（根 package.json 是唯一来源）
function Get-AppVersion {
    $text = [IO.File]::ReadAllText((Join-Path $ProjectDir 'package.json'))
    $m = [regex]::Match($text, '"version"\s*:\s*"([^"]+)"')
    if (-not $m.Success) {
        Write-Status "无法从 package.json 读取版本号" 1
        exit 1
    }
    $m.Groups[1].Value
}

# 读取 JSON 文本（含 BOM 兼容），供各同步函数使用
function Read-TextFile([string]$Path) {
    [IO.File]::ReadAllText($Path)
}

# 写回 JSON 文本（无 BOM，保留原格式）
function Write-TextFile([string]$Path, [string]$Text) {
    [IO.File]::WriteAllText($Path, $Text, $Utf8NoBom)
}

# 精准替换 package.json / libs/core/deno.json 中的 "version": "<old>"
# （要求恰好出现一次，避免误伤依赖版本）
function Update-JsonVersion([string]$Path, [string]$OldVersion, [string]$NewVersion, [string]$Label) {
    $text    = Read-TextFile $Path
    $pattern = '"version"\s*:\s*"' + [regex]::Escape($OldVersion) + '"'
    $rx      = [regex]::new($pattern)
    $count   = $rx.Matches($text).Count
    if ($count -eq 0) {
        Write-Status "$Label 当前版本不是 $OldVersion" 1
        exit 1
    }
    if ($count -gt 1) {
        Write-Status "$Label 中 version=$OldVersion 出现 $count 次，无法安全替换，请人工处理" 1
        exit 1
    }
    $text = $rx.Replace($text, ('"version": "' + $NewVersion + '"'), 1)
    Write-TextFile $Path $text
    Write-Status "  $Label -> $NewVersion" 0
}

# 精准替换 Cargo.toml 顶部的 version = "<old>"（每文件恰好一处）
function Update-CrateVersion([string]$RelDir, [string]$OldVersion, [string]$NewVersion) {
    $path = Join-Path $ProjectDir (Join-Path $RelDir 'Cargo.toml')
    $text = Read-TextFile $path
    $rx   = [regex]::new('(?m)^version\s*=\s*"' + [regex]::Escape($OldVersion) + '"')
    $count = $rx.Matches($text).Count
    if ($count -eq 0) {
        Write-Status "$RelDir\Cargo.toml 当前版本不是 $OldVersion" 1
        exit 1
    }
    if ($count -gt 1) {
        Write-Status "$RelDir\Cargo.toml 中 version=$OldVersion 出现 $count 次，无法安全替换" 1
        exit 1
    }
    $text = $rx.Replace($text, ('version = "' + $NewVersion + '"'), 1)
    Write-TextFile $path $text
    Write-Status "  $RelDir\Cargo.toml -> $NewVersion" 0
}

# 更新应用版本号（全量同步，格式保留，避免产生无关 diff）：
#   package.json（唯一来源）+ package-lock.json（根条目 + packages[""] +
#   packages["node_modules/@dockbar/lib"] 共 3 处）+ libs/core/deno.json
#   + 5 个成员 crate 的 Cargo.toml + Cargo.lock 中 5 个 dockbar* 包
# 等价于 npm run version:ci（versionish.ts ci），不触碰 changelog、不产生提交
function Update-AppVersion([string]$NewVersion) {
    if ($NewVersion -notmatch $VersionPattern) {
        Write-Status "版本号格式无效: $NewVersion（示例: 2.8.9）" 1
        exit 1
    }
    $currentVersion = Get-AppVersion
    if ($currentVersion -eq $NewVersion) {
        Write-Status "版本号已是 $NewVersion，无需更新" 2
        return
    }

    Write-Status "同步版本号 $currentVersion -> $NewVersion ..." 0

    # package.json（唯一来源）
    Update-JsonVersion (Join-Path $ProjectDir 'package.json') $currentVersion $NewVersion 'package.json'

    # package-lock.json：根条目 + packages[""] 均为 "name": "dockbar" + version，
    # @dockbar/lib 为 file: 链接包，共 3 处，数量必须精确匹配
    $lockPath = Join-Path $ProjectDir 'package-lock.json'
    $text = Read-TextFile $lockPath
    $rxRoot = [regex]::new(('("name": "dockbar",\s*"version"\s*:\s*")' + [regex]::Escape($currentVersion) + '(")'))
    $rootCount = $rxRoot.Matches($text).Count
    $rxLib = [regex]::new(('("name": "@dockbar/lib",\s*"version"\s*:\s*")' + [regex]::Escape($currentVersion) + '(")'))
    $libCount = $rxLib.Matches($text).Count
    if ($rootCount -ne 2 -or $libCount -ne 1) {
        Write-Status "package-lock.json 版本条目异常（dockbar×$rootCount / @dockbar/lib×$libCount），请先运行 npm install 刷新锁文件" 1
        exit 1
    }
    # ${1}/${2} 花括号形式：避免 .NET 把 "$12.9.0" 解析成组 12 引用
    $text = $rxRoot.Replace($text, ('${1}' + $NewVersion + '${2}'))
    $text = $rxLib.Replace($text, ('${1}' + $NewVersion + '${2}'))
    Write-TextFile $lockPath $text
    Write-Status "  package-lock.json (3 处) -> $NewVersion" 0

    # libs/core/deno.json（@dockbar/lib 的 Deno 包版本）
    Update-JsonVersion (Join-Path $CoreDir 'deno.json') $currentVersion $NewVersion 'libs/core/deno.json'

    # 5 个成员 crate 的 Cargo.toml
    foreach ($dir in $CrateDirs) { Update-CrateVersion $dir $currentVersion $NewVersion }

    # Cargo.lock：5 个 dockbar* 包条目
    $lockPath = Join-Path $ProjectDir 'Cargo.lock'
    $text = Read-TextFile $lockPath
    foreach ($pkg in $CrateNames) {
        # (?<eol>\r?\n) 捕获原有换行符，替换时原样保留，避免 diff 噪音
        $rx = [regex]::new(('name = "(?<name>' + [regex]::Escape($pkg) + ')"(?<eol>\r?\n)version = "(?<ver>' + [regex]::Escape($currentVersion) + ')"'))
        if ($rx.Matches($text).Count -ne 1) {
            Write-Status "Cargo.lock 中 $pkg 的版本条目不是 $currentVersion（或出现多次），请先运行 cargo check 刷新锁文件" 1
            exit 1
        }
        $text = $rx.Replace($text, ('name = "${name}"${eol}version = "' + $NewVersion + '"'))
    }
    Write-TextFile $lockPath $text
    Write-Status "  Cargo.lock (5 个包) -> $NewVersion" 0

    Write-Status "应用版本已更新为 $NewVersion" 0
}

# 打包 NSIS 安装包。版本号自动附加年月日时间戳，如 2.8.9+20261013；打包前清理旧产物。
function Invoke-Build([string]$RequestedVersion) {
    if ($RequestedVersion) { Update-AppVersion $RequestedVersion }

    Set-Location $ProjectDir
    Initialize-BuildEnvironment
    Write-Status "项目目录: $ProjectDir" 0

    if (-not (Test-Path -LiteralPath (Join-Path $ProjectDir 'node_modules'))) {
        Write-Status "安装前端依赖 (npm install)..." 0
        & npm install
        Assert-LastExitCode "前端依赖安装"
    }

    if (Test-Path -LiteralPath $NsisBundleDir) {
        Get-ChildItem -LiteralPath $NsisBundleDir -Include *.exe, *.sig, *.zip -Recurse -ErrorAction SilentlyContinue | Remove-Item -Force
        Write-Status "已清理旧 nsis 产物" 0
    }

    $baseVersion  = Get-AppVersion
    $buildVersion = "$baseVersion+$(Get-Date -Format yyyyMMdd)"
    $mergeFile    = Join-Path ([IO.Path]::GetTempPath()) ("tauri-version-" + [Guid]::NewGuid().ToString('N').Substring(0, 8) + ".json")
    Write-Status "构建版本: $buildVersion" 0

    try {
        # --config 以 JSON merge 覆盖版本号（无 BOM，避免 serde 解析失败）
        [IO.File]::WriteAllText($mergeFile, '{"version": "' + $buildVersion + '"}')
        $buildStart = Get-Date
        Write-Status "单段式打包 tauri build --bundles nsis（release 构建耗时较久，项目无密钥）..." 0
        Invoke-TauriCli @('build', '--bundles', 'nsis', '--config', $mergeFile)
        if ($LASTEXITCODE -ne 0) {
            Write-Status "打包 失败" 1
            exit 1
        }
        Write-Status ("构建耗时: " + (Format-Duration ((Get-Date) - $buildStart).TotalSeconds)) 0
    } finally {
        Remove-Item -LiteralPath $mergeFile -Force -ErrorAction SilentlyContinue
    }

    $artifact = Get-ChildItem -LiteralPath $NsisBundleDir -Filter *-setup.exe -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if (-not $artifact) {
        Write-Status "打包完成但未找到 nsis 产物，请检查 $NsisBundleDir" 1
        exit 1
    }
    Write-Status ("产物已生成 ({0:N1} MB)" -f ($artifact.Length / 1MB)) 0
    Write-Host $artifact.FullName -ForegroundColor Green
}

# 从注册表卸载项中定位已安装的 DockBar（NSIS 安装器写入）
function Get-InstalledPackage {
    $roots = @(
        'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall',
        'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall',
        'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall'
    )
    foreach ($root in $roots) {
        if (-not (Test-Path -LiteralPath $root)) { continue }
        foreach ($key in (Get-ChildItem $root -ErrorAction SilentlyContinue)) {
            $props = Get-ItemProperty -Path $key.PSPath -ErrorAction SilentlyContinue
            if ($props -and $props.DisplayName -like 'DockBar*') { return $props }
        }
    }
    return $null
}

# 启动安装/卸载程序；perMachine 安装需要管理员权限，自动触发 UAC
function Start-InstallerProcess([string]$FilePath, [string]$Arguments) {
    try {
        return Start-Process -FilePath $FilePath -ArgumentList $Arguments -Wait -PassThru
    } catch {
        if ($_.Exception.Message -match 'elevation|提升') {
            Write-Status "该操作需要管理员权限，请在 UAC 弹窗中确认" 2
            return Start-Process -FilePath $FilePath -ArgumentList $Arguments -Wait -PassThru -Verb RunAs
        }
        throw
    }
}

# 安装最新的 NSIS 安装包（/S 静默；安装器钩子会自行停止/重装 dockbar 服务）
function Invoke-Install {
    $installer = Get-ChildItem -LiteralPath $NsisBundleDir -Filter *-setup.exe -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if (-not $installer) {
        Write-Status "未找到 NSIS 安装包，请先打包: .\management.ps1 build" 1
        exit 1
    }

    $nsisVersion = ''
    $m = [regex]::Match($installer.Name, '_(\d+\.\d+\.\d+(?:[+-][0-9A-Za-z.-]+)?)_x64-setup\.exe$')
    if ($m.Success) { $nsisVersion = $m.Groups[1].Value }

    $installed = Get-InstalledPackage
    if ($installed) {
        Write-Status "已安装 $($installed.DisplayName) $($installed.DisplayVersion)，安装包版本 $nsisVersion" 0
        if (-not $Yes) {
            if ("$($installed.DisplayVersion)" -eq $nsisVersion) {
                $confirm = Read-Host "版本相同，是否覆盖重装? [y/N]"
            } else {
                $confirm = Read-Host "是否升级/更换到安装包版本? [y/N]"
            }
            if ($confirm -notmatch '^[yY]$') {
                Write-Status "已取消安装" 2
                exit 0
            }
        }
    }

    Write-Status "安装 $($installer.FullName) ..." 0
    $proc = Start-InstallerProcess $installer.FullName '/S'
    if ($proc.ExitCode -ne 0) {
        Write-Status "安装 失败 (退出码 $($proc.ExitCode))" 1
        exit 1
    }
    $after = Get-InstalledPackage
    if ($after) {
        Write-Status "安装 $($after.DisplayName) $($after.DisplayVersion) 完成" 0
    } else {
        Write-Status "安装 完成" 0
    }
    Write-Host "安装完成，可从开始菜单启动 DockBar"
}

# 卸载已安装的 DockBar（/S 静默；卸载器钩子会自行移除 dockbar 服务）
function Invoke-Uninstall {
    $installed = Get-InstalledPackage
    if (-not $installed) {
        Write-Status "DockBar 未安装，无需卸载" 2
        exit 0
    }
    if (-not $installed.UninstallString) {
        Write-Status "未找到卸载程序（注册表 UninstallString 为空）" 1
        exit 1
    }

    Write-Status "卸载 $($installed.DisplayName) $($installed.DisplayVersion) ..." 0
    $uninstPath = $installed.UninstallString.Trim().Trim('"')
    Start-InstallerProcess $uninstPath '/S' | Out-Null

    # NSIS 卸载器会自复制到临时目录执行，-Wait 返回时可能尚未清理完，轮询注册表确认
    $deadline = (Get-Date).AddSeconds(20)
    while ((Get-InstalledPackage) -and (Get-Date) -lt $deadline) {
        Start-Sleep -Milliseconds 500
    }
    if (Get-InstalledPackage) {
        Write-Status "卸载程序已执行，但注册表卸载项仍存在，请稍后手动确认" 2
    } else {
        Write-Status "卸载 完成" 0
    }
}

# 启动开发调试（npm run dev = cargo build + tauri dev，前端经 beforeDevCommand 挂在 :3579）
# tauri dev 把 bundle.resources 拷入 target\debug\static 时只增改、不删除源里已不存在的
# 旧文件；历史版本残留（如已退役的 themes/sounds 资产）会让完整性校验报 Extra 失败，
# 启动前先清除 target\debug\static 中源目录不再包含的残留文件
function Invoke-Dev {
    Set-Location $ProjectDir
    Initialize-BuildEnvironment
    Remove-StaleDevResources
    if (-not (Test-Path -LiteralPath (Join-Path $ProjectDir 'node_modules'))) {
        Write-Status "安装前端依赖 (npm install)..." 0
        & npm install
        Assert-LastExitCode "前端依赖安装"
    }
    & npm run dev
}

# 清理 target\debug\static 中源 src\static 已不存在的残留文件与空目录
function Remove-StaleDevResources {
    $srcRoot = Join-Path $ProjectDir 'src\static'
    if (-not (Test-Path -LiteralPath $srcRoot)) { return }
    $srcFiles = @{}
    Get-ChildItem -LiteralPath $srcRoot -Recurse -File | ForEach-Object {
        $srcFiles[$_.FullName.Substring($srcRoot.Length + 1)] = $true
    }

    $targetRoot = Join-Path $ProjectDir 'target\debug\static'
    if (-not (Test-Path -LiteralPath $targetRoot)) { return }
    $removed = 0
    # 先删孤儿文件，再自底向上删空目录
    Get-ChildItem -LiteralPath $targetRoot -Recurse -File | ForEach-Object {
        $rel = $_.FullName.Substring($targetRoot.Length + 1)
        if (-not $srcFiles.ContainsKey($rel)) {
            Remove-Item -LiteralPath $_.FullName -Force
            $removed++
        }
    }
    Get-ChildItem -LiteralPath $targetRoot -Recurse -Directory |
        Sort-Object { $_.FullName.Length } -Descending | ForEach-Object {
        if (-not (Get-ChildItem -LiteralPath $_.FullName -ErrorAction SilentlyContinue)) {
            Remove-Item -LiteralPath $_.FullName -Force
        }
    }
    if ($removed -gt 0) {
        Write-Status "已清理 $targetRoot 中 $removed 个残留资源文件" 0
    }
}

# 前端 UI 打包（esbuild 多应用构建，产物在 dist/）
function Invoke-BuildUi {
    Set-Location $ProjectDir
    Initialize-BuildEnvironment
    & npm run build:ui
    Assert-LastExitCode "前端 UI 打包"
}

# 共享库构建（libs/core：Rust->TS 绑定 + npm 包，preinstall 依赖它）
function Invoke-BuildLib {
    Set-Location $ProjectDir
    Initialize-BuildEnvironment
    if (-not (Get-Command deno -ErrorAction SilentlyContinue)) {
        Write-Status "缺少 deno，无法构建 libs/core" 1
        exit 1
    }
    & npm run build:lib
    Assert-LastExitCode "共享库构建"
}

# Rust 测试（CI 同款：cargo test --locked，覆盖 workspace 全部成员）
function Invoke-RustTest {
    Invoke-RustCommand 'Rust 测试' @('test', '--locked')
}

# Rust 类型检查（AGENTS.md 口径：优先 cargo check，勿用 release 构建）
function Invoke-RustCheck {
    Invoke-RustCommand 'Rust 类型检查' @('check', '--locked')
}

# Rust 静态检查（CI 同款）
function Invoke-RustClippy {
    Invoke-RustCommand 'Rust 静态检查' @('clippy', '--locked', '--all-targets', '--', '-D', 'warnings')
}

# 格式检查（CI 同款：Rust + Deno）
function Invoke-FmtCheck {
    Set-Location $ProjectDir
    Initialize-BuildEnvironment
    Write-Status "执行 cargo fmt --check ..." 0
    & cargo fmt -- --check
    Assert-LastExitCode "Rust 格式检查"
    if (Get-Command deno -ErrorAction SilentlyContinue) {
        Write-Status "执行 deno fmt --check ..." 0
        & deno fmt --check
        Assert-LastExitCode "Deno 格式检查"
    } else {
        Write-Status "跳过 deno fmt（未安装 deno）" 2
    }
}

# Deno 静态检查（CI 同款）
function Invoke-DenoLint {
    Set-Location $ProjectDir
    Initialize-BuildEnvironment
    if (-not (Get-Command deno -ErrorAction SilentlyContinue)) {
        Write-Status "缺少 deno，无法执行 lint" 1
        exit 1
    }
    & deno lint
    Assert-LastExitCode "Deno 静态检查"
}

# 前端类型检查（tsc + svelte-check + deno check，CI 同款）
function Invoke-TypeCheck {
    Set-Location $ProjectDir
    Initialize-BuildEnvironment
    & npm run type-check
    Assert-LastExitCode "前端类型检查"
}

# Rust 命令统一入口
function Invoke-RustCommand {
    param([string]$Title, [string[]]$CargoArgs)
    Set-Location $ProjectDir
    Initialize-BuildEnvironment
    Write-Status "在 $ProjectDir 执行 cargo $($CargoArgs -join ' ') ..." 0
    & cargo @CargoArgs
    Assert-LastExitCode $Title
}

# 清理生成物（同 scripts/clean.ps1，外加 target 与 nsis 产物）
function Invoke-Clean {
    Set-Location $ProjectDir
    $targets = @(
        (Join-Path $ProjectDir 'node_modules'),
        (Join-Path $ProjectDir 'dist'),
        (Join-Path $ProjectDir 'target'),
        (Join-Path $CoreDir 'gen'),
        (Join-Path $CoreDir 'npm')
    )
    foreach ($t in $targets) {
        if (Test-Path -LiteralPath $t) {
            Write-Status "删除 $t ..." 0
            Remove-Item -Force -Recurse -LiteralPath $t
        }
    }
    Write-Status "清理完成，重新开始请依次运行: npm install && .\management.ps1 build-lib" 0
}

# 显示帮助
function Show-Help {
    Write-Host "用法: .\management.ps1 [build [版本号] | version <版本号> | install | uninstall |"
    Write-Host "               dev | test | type-check | check | clippy | fmt-check | lint |"
    Write-Host "               build-ui | build-lib | clean | -h|--help]"
    Write-Host ""
    Write-Host "命令:"
    Write-Host "  build [版本号]      打包 NSIS 安装包；传版本号时先全量同步项目版本，"
    Write-Host "                      打包版本自动附加日期（如 2.8.9+20261013），完成输出构建耗时"
    Write-Host "  version <版本号>    全量同步版本号（package.json / package-lock.json /"
    Write-Host "                      libs/core/deno.json / 5×Cargo.toml / Cargo.lock），不执行构建；"
    Write-Host "                      发版请用项目自带 npm run version:start / version:finish"
    Write-Host "  install [-Yes]      静默安装最新的 NSIS 安装包（-Yes 跳过覆盖/升级确认，perMachine 需 UAC）"
    Write-Host "  uninstall           静默卸载已安装的 DockBar"
    Write-Host "  dev                 启动开发调试（npm run dev = cargo build + tauri dev）"
    Write-Host "  test                Rust 测试（cargo test --locked，CI 同款）"
    Write-Host "  check               Rust 类型检查（cargo check --locked）"
    Write-Host "  clippy              Rust 静态检查（cargo clippy --all-targets -- -D warnings）"
    Write-Host "  fmt-check           格式检查（cargo fmt --check + deno fmt --check）"
    Write-Host "  lint                Deno 静态检查（deno lint）"
    Write-Host "  type-check          前端类型检查（tsc + svelte-check + deno check）"
    Write-Host "  build-ui            前端 UI 打包（npm run build:ui，产物在 dist/）"
    Write-Host "  build-lib           共享库构建（npm run build:lib，需 deno）"
    Write-Host "  clean               清理 node_modules / dist / target / libs/core 生成物"
    Write-Host "  -h,--help           显示帮助"
    Write-Host ""
    Write-Host "示例:"
    Write-Host "  .\management.ps1 build"
    Write-Host "  .\management.ps1 build 2.9.0"
    Write-Host "  .\management.ps1 version 2.9.0"
    Write-Host "  .\management.ps1 install"
    Write-Host "  .\management.ps1 uninstall"
    Write-Host "  .\management.ps1 check"
}

# 交互菜单
function Show-Menu {
    Write-Host "请选择操作:"
    Write-Host "  1) 打包 NSIS 安装包"
    Write-Host "  2) 安装安装包"
    Write-Host "  3) 卸载"
    Write-Host "  4) 开发调试 (tauri dev)"
    Write-Host "  5) Rust 测试 (cargo test)"
    Write-Host "  6) Rust 类型检查 (cargo check)"
    Write-Host "  7) 前端类型检查 (type-check)"
    Write-Host "  h) 帮助  q) 退出"
    $choice = Read-Host "输入选项"
    switch -regex ($choice) {
        '^1$'    { Invoke-Build }
        '^2$'    { Invoke-Install }
        '^3$'    { Invoke-Uninstall }
        '^4$'    { Invoke-Dev }
        '^5$'    { Invoke-RustTest }
        '^6$'    { Invoke-RustCheck }
        '^7$'    { Invoke-TypeCheck }
        '^[hH]$' { Show-Help }
        '^[qQ]$' { exit 0 }
        '^$'     { exit 0 }
        default  { Write-Status "无效选项: $choice" 1; exit 1 }
    }
}

Write-Host "================================================================"
Write-Host "          DockBar 打包/安装/卸载工具 (Windows)"
Write-Host "================================================================"
Write-Host ""

switch ($Command) {
    'build'      { Invoke-Build $VersionArg }
    'version'    {
        if (-not $VersionArg) {
            Write-Status "请传入版本号，例如: .\management.ps1 version 2.9.0" 1
            exit 1
        }
        Update-AppVersion $VersionArg
    }
    'install'    { Invoke-Install }
    'uninstall'  { Invoke-Uninstall }
    'dev'        { Invoke-Dev }
    'test'       { Invoke-RustTest }
    'check'      { Invoke-RustCheck }
    'clippy'     { Invoke-RustClippy }
    'fmt-check'  { Invoke-FmtCheck }
    'lint'       { Invoke-DenoLint }
    'type-check' { Invoke-TypeCheck }
    'build-ui'   { Invoke-BuildUi }
    'build-lib'  { Invoke-BuildLib }
    'clean'      { Invoke-Clean }
    'help'       { Show-Help }
    '-h'         { Show-Help }
    '--help'     { Show-Help }
    ''           { Show-Menu }
    default      { Write-Status "未知选项: $Command" 1; Show-Help; exit 1 }
}

Write-Host ""
Write-Host "================================================================"
Write-Status "脚本执行完成" 0
Write-Host "================================================================"
