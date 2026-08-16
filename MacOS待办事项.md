# macOS 待办事项

> 状态：M8 macOS 适配进行中。本机为 Windows，以下事项需在 macOS 环境或 GitHub Actions 上完成/验证。

## 已完成（代码已写好，待 macOS 验证）

- [ ] macOS 构建 workflow（`.github/workflows/macos-build.yml`）：推送到 `dev`/`main` 后自动在 macOS runner 上编译、检查、测试并产出 debug 产物
- [ ] 辅助功能权限检测（`AXIsProcessTrusted`）与设置页“macOS 权限”区
- [ ] 一键打开系统“辅助功能”设置
- [ ] 自动替换前未授权权限时的明确提示
- [ ] macOS 默认快捷键 `Cmd+Option+Space`，注册时 `Ctrl`→`Cmd`、`Alt`→`Option`
- [ ] 托盘菜单栏行为（左键弹出菜单）
- [ ] DMG 打包配置（`minimumSystemVersion 10.13`）

## 待开发 / 待验证

- [ ] macOS 真机/CI 编译验证：`AXIsProcessTrusted` 链接、DMG 打包是否通过
- [ ] 首次启动/首次使用时自动引导开启辅助功能权限的交互体验
- [ ] 快捷键录制、冲突提示在 macOS 上验证
- [ ] 自动替换链路在 macOS 常用应用验证（备忘录、浏览器、IDE、Office）
- [ ] 悬浮条“不抢焦点”（非激活面板）行为验证
- [ ] DMG 安装、卸载与启动验证
- [ ] 自动更新 macOS 侧：检测到新版本后下载 DMG 并引导安装（目前只实现 Windows）
- [ ] 开机自启（LaunchAgent）在 macOS 上验证
- [ ] 设置窗口/悬浮条窗口在 macOS 上的显示、隐藏、关闭行为验证
- [ ] 中英文切换、赞助区在 macOS 上验证
- [ ] 双架构（Intel + Apple Silicon）：安装 `x86_64-apple-darwin` 与 `aarch64-apple-darwin` 目标并合并 Universal 包
- [ ] 签名与公证：配置 Apple Developer 账号后执行 `codesign`、`notarytool`、`stapler`
- [ ] macOS 回归清单（参考 `docs/测试清单.md`，在 macOS 上执行）

## 测试方式建议

1. GitHub Actions macOS runner：推代码后云端自动编译验证（已在 workflow 中配置）。
2. 云 Mac：MacStadium、MacinCloud、AWS EC2 Mac 按小时租用。
3. 借一台 Mac 做完整交互测试。
