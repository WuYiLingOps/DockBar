# Changelog

## [1.0.0] - 2026-10-14

首个正式版本。

### Highlights

- Dock 任务栏：mac 式图标悬停放大动画（弹簧物理驱动），悬停显示应用名称提示，均可在设置中调整或关闭
- 桌面 Widget 系统、主题系统与插件体系（widgets / themes / plugins）
- Windows 深度集成：任务栏替换、多显示器、自动隐藏、开机自启
- 内置 WebView2 Fixed Version Runtime 安装包，无需在线安装运行时
- 完整 i18n 支持（含简体/繁体中文）

### Changed

- 移除启动 splash 页与管理员权限告警弹窗
- 内置 widget 加载时标记 bundled，不再误弹权限请求

### Build

- 无签名（keyless）发布链路：安装包完整性基于 SHA256 校验和，不依赖代码签名服务
- CI/CD 精简：仅保留 ci.yml（格式化 / lint / 类型检查 / 测试）与 release.yml（tag 触发的双架构 NSIS 构建）
