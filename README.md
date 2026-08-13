# StringCraft（字符串工坊）

跨平台（Windows 10/11、macOS）轻量级字符串转换工具。技术栈：Tauri 2 + Rust，前端 Svelte + TypeScript + Vite。

详细需求见 [docs/需求方案.md](docs/需求方案.md)，开发日志见 [docs/devlog/](docs/devlog/)。

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

## 目录结构

```text
StringCraft
├── src/                 # Svelte 前端（悬浮条 + 设置窗口）
├── src-tauri/           # Rust 后端与 Tauri 配置
│   ├── src/config.rs    # 默认配置与 20 个内置按钮
│   ├── src/hotkey.rs    # 全局快捷键注册
│   ├── src/tray.rs      # 托盘图标与菜单
│   └── src/commands.rs  # Tauri IPC 命令
├── assets/              # 原始设计素材（占位图标）
└── docs/                # 需求方案与开发日志
```

## 开发阶段

当前处于 M1（项目骨架）阶段，详见需求文档第 7 节。
