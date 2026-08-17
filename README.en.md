# StringCraft

[中文](README.md)

A lightweight, fast, fully local string transformation tool. Select any text, transform it with one click — case conversion, camelCase, JSON formatting, RMB uppercase, and more — and paste the result back automatically.

## Highlights

- Lightweight: ~3MB installer, fast startup, stays in the system tray
- Fully local: transformed text is only processed in memory; no uploads, no logs, no tracking
- Global hotkey: select text anywhere and press the hotkey to show the floating bar
- Auto replace: results are pasted back automatically and the original clipboard is restored
- 23 built-in transforms: case, word/sentence casing, camelCase conversions, separator replacements, duplicate-line removal, JSON formatting, RMB uppercase conversion, and more
- Custom buttons: append suffix, prepend prefix, replace text, remove duplicate lines
- Highly customizable: button name/description/size/colors/opacity/language
- Themes: light/dark/system with 20 Macaron and Morandi colors
- Bilingual: Chinese/English UI and default button texts
- Auto update: checks GitHub Releases and can download updates automatically

## Usage

1. Download and install the latest release (Windows 10/11).
2. Press the default hotkey `Ctrl+Alt+Space` to show/hide the floating bar (configurable in Settings).
3. Select text in any app and click a transform button on the floating bar.
4. The result replaces the selection automatically; configure buttons in Settings as you like.

## Download

- Latest release: [GitHub Releases](https://github.com/Jiang-Xiaozhong/StringCraft/releases)
- The installer is a NSIS installer; double-click to install.

## Future macOS Support

StringCraft currently supports Windows 10/11. A macOS version is in progress.

Planned macOS support:

- OS version: macOS 10.13 or later (Apple Silicon, M-series)
- Global hotkey: `Cmd+Option+Space` to show/hide the floating bar
- Accessibility permission: guided setup on first use (required for auto replace)
- Menu bar tray: show/hide floating bar, open settings, quit
- Installer: DMG package
- Auto update: check for updates and download/install
- Features such as Chinese/English switching and sponsorship match the Windows version

Current status:

- macOS code and cloud build verification are complete;
- Preparing a real macOS device for testing;
- Once tested, the macOS installer will be published on Releases (timing to be confirmed).

> Note: the macOS version requires real-device interaction testing before release.

## Privacy

- No network requests except update checks.
- Update checks only access GitHub Releases and upload nothing.
- Transformed text lives only in memory and is never written to logs, config, or disk.

## Feedback & Sponsorship

- Feedback or suggestions: jxzlh1208@163.com
- If StringCraft helps you, consider sponsoring to support ongoing development and maintenance.
