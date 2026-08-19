# StringCraft

[中文](README.md)

> Select, transform, and auto-paste the result — a featherweight text-processing tool that works in any app.

![GitHub Release](https://img.shields.io/github/v/release/Jiang-Xiaozhong/StringCraft?color=6a9b7f&label=Release)
![Platform](https://img.shields.io/badge/Platform-Windows%2010%2F11-0078d4)
![Installer](https://img.shields.io/badge/Installer-%7E3MB-57b560)
![Memory](https://img.shields.io/badge/Memory-%7E10MB-57b560)
![macOS](https://img.shields.io/badge/macOS-M%20series%20%28soon%29-9aa0a6)

## Why StringCraft?

- **One-step workflow**: select text in any app, press `Ctrl+Alt+Space` to show the floating bar, click a button, and the result replaces your selection automatically — no copy, paste, or window hopping.
- **100% local, zero uploads**: all transforms run in memory on your machine. No network calls, no logs, no telemetry. Safe for code, SQL, logs, configs, and other sensitive content.
- **Featherweight**: ~3MB installer, ~10MB resident memory, fast startup, stays quietly in the system tray.
- **28 ways to transform**: 23 built-in transforms + 5 custom button types covering most everyday text tasks.
- **Fully customizable**: add, remove, reorder, rename, resize buttons; frosted-glass style, theme colors, language — build a toolbar that fits your workflow.

## Built-in Transforms (23)

- **Case**: `AB` UPPERCASE · `ab` lowercase · `Ab` Capitalize Each Word · `aB` lowercase each word's first letter · `Aa` Capitalize Each Sentence
- **Naming style**: `camel` to camelCase · `c_` to snake_case · `c sp` to space-separated (multi-line input stays multi-line)
- **Separators**: space ↔ underscore / newline / hyphen (`s_` `s↵` `s-` `_s` `↵s`) · underscore ↔ hyphen / dot (`_-` `_.` `-_` `._`)
- **Cleanup**: `NoSp` remove all spaces · `NoNl` remove all newlines · `NoSym` strip everything except letters, digits, and whitespace
- **Formatting**: `JSON` format JSON · `NumRMB` numbers to RMB uppercase · `RMBNum` RMB uppercase to numbers

Fully Unicode-aware: Chinese, English, and mixed content are handled consistently.

## Custom Buttons (5 types)

- **Append suffix / Prepend prefix / Prepend & append**: add to every line, preserving line breaks
- **Replace text**: full-text replace (case-sensitive, non-regex)
- **Remove duplicate lines**: keep the first occurrence, preserve order

Custom buttons start hidden; check "Show" in Settings to place them on the floating bar. Names, descriptions, and parameters can be edited anytime.

## Highlights

- 🎯 **Floating bar**: borderless, always-on-top, freely draggable, remembers its position across restarts
- ⌨️ **Global hotkey**: default `Ctrl+Alt+Space` to show/hide, fully configurable
- 🧰 **Button manager**: add, remove, drag to reorder, show/hide; bar width adapts to content
- 🎨 **Appearance**: frosted-glass buttons; 20 Macaron & Morandi preset colors plus custom background; live-adjustable opacity and font size; light / dark / system themes
- 🌍 **Bilingual UI**: switch between Chinese and English, including default button texts (more languages planned)
- 📦 **Config export/import**: restore your setup on a new machine or after a reinstall
- 🔄 **Auto-update**: checks GitHub Releases and can download updates automatically
- 🖥️ **System tray**: show/hide the floating bar or quit at any time
- 💝 **Sponsorship**: scan a QR code in Settings to support development

## Quick Start

1. Download the latest installer from [GitHub Releases](https://github.com/Jiang-Xiaozhong/StringCraft/releases) (Windows 10/11, double-click to install).
2. Press the default hotkey `Ctrl+Alt+Space` to show/hide the floating bar.
3. Select text in any app and click a transform button.
4. The result replaces the selection automatically; open Settings to configure buttons your way.

## System Requirements

- Windows 10 / 11
- macOS (Apple Silicon, M-series) in progress — coming soon

## Privacy

- No network requests except update checks.
- Update checks only access GitHub Releases and upload nothing.
- Transformed text lives only in memory and is never written to logs, config, or disk.

## Future macOS Support

StringCraft currently supports Windows 10/11; a macOS version is being adapted.

Planned macOS support:

- OS: macOS 10.13 or later (Apple Silicon, M-series)
- Global hotkey: `Cmd+Option+Space` to show/hide the floating bar
- Accessibility permission: guided first-run setup (required for auto-replace)
- Menu bar tray: show/hide floating bar, open settings, quit
- Installer: DMG package
- Auto-update: check, download, and install updates
- Chinese/English switching, sponsorship, and all features matching the Windows version

Current status:

- macOS code and cloud build verification are complete;
- Preparing a real macOS device for testing;
- The macOS installer will be published on Releases once tested (timing to be confirmed).

> Note: the macOS version requires real-device interaction testing before release.

## Feedback & Sponsorship

Bugs and suggestions: jxzlh1208@163.com. If StringCraft helps you, consider scanning the QR code in Settings to sponsor ongoing development and maintenance.
