# StringCraft（字符串工坊）

跨平台（Windows 10/11、macOS）轻量级字符串转换工具。技术栈：Tauri 2 + Rust，前端 Svelte + TypeScript + Vite。

详细需求见 [docs/需求方案.md](docs/需求方案.md)，开发日志见 [docs/devlog/](docs/devlog/)，Windows 回归清单见 [docs/测试清单.md](docs/测试清单.md)。

## 主要功能

- 悬浮条：全局快捷键呼入/呼出、自由拖动与位置记忆、按宽度自动换行、边缘拖动调整宽度
- 23 个内置转换：大小写、驼峰、分隔符替换、JSON 格式化、人民币大写转换等
- 自定义按钮：加后缀、加前缀、文本替换、去重复行
- 设置页：快捷键录制、按钮管理（增删/拖拽排序/显示隐藏）、外观滑块实时反馈、20 种马卡龙/莫兰迪配色
- 配置导出/导入、中英文切换、检查更新与可选自动更新
- 托盘常驻：呼出/隐藏悬浮条、打开设置、退出

## 快速开始

前置要求：

- Node.js 20+（含 npm）
- Rust stable + Cargo
- Windows：Visual Studio Build Tools（含 MSVC 与 Windows SDK）；macOS：Xcode Command Line Tools

```bash
npm install
npm run tauri dev
```

## 常用命令

| 命令 | 说明 |
| --- | --- |
| `npm run dev` | 仅启动前端 Vite 开发服务器 |
| `npm run check` | 前端类型检查（svelte-check） |
| `npm run build` | 前端生产构建 |
| `npm run tauri dev` | 启动 Tauri 开发调试 |
| `npm run tauri build -- --debug` | Windows debug 构建 |
| `npm run tauri build` | 生产构建（NSIS 安装包） |

## 检查更新

更新检查基于 GitHub Releases（`Jiang-Xiaozhong/StringCraft`）。应用会在启动后延迟检查一次，之后每 24 小时检查一次；也可在设置页手动“检查更新”。

## 隐私

- 除“检查更新”外无任何网络请求；更新检查仅访问 GitHub Releases，不上传数据、无遥测。
- 被转换的文本只在内存中处理，不写入日志、配置或磁盘。

## 反馈与打赏

有任何问题或建议请反馈至：jxzlh1208@163.com。如果对你有帮助，也可以在设置页打赏支持。

## 目录结构

```text
StringCraft
├── src/                 # Svelte 前端（悬浮条 + 设置窗口 + 语言包 + 收款码）
├── src-tauri/           # Rust 后端与 Tauri 配置
│   ├── src/config.rs    # 默认配置与 23 个内置按钮
│   ├── src/transform.rs # 转换纯函数与单元测试
│   ├── src/update.rs    # 检查更新与自动更新
│   ├── src/tray.rs      # 托盘图标与菜单
│   └── src/commands.rs  # Tauri IPC 命令
├── assets/              # 原始设计素材（占位图标）
└── docs/                # 需求方案、开发日志与测试清单
```

## 开发阶段

当前处于 M7（Windows 打磨与发布准备）阶段，详见需求文档第 7 节。
