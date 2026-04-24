[![Downloads](https://img.shields.io/github/downloads/Zynix-Scripts/Blur-AutoClicker-Linux/total?style=for-the-badge&label=downloads)](https://github.com/Zynix-Scripts/Blur-AutoClicker-Linux/releases)

# Blur Auto Clicker | Linux Port

> Source code is on the [linux branch](https://github.com/Zynix-Scripts/Blur-AutoClicker-Linux/tree/linux).

## Supports Wayland AND X11

## Why

A lot of popular auto clickers like OP Auto Clicker and Speed Auto Clicker are inaccurate at higher speeds | setting 50 CPS might give you 40 or 60. This project aims for precision: click exactly at the CPS you set, even at high speeds.

Performance is a core focus. RAM usage stays around 50 MB and is designed to never exceed 100 MB.

---

## Features

<div align="center">
    <img src="https://github.com/Blur009/Blur-AutoClicker/blob/main/public/30s_500cps_Speed_Test.png" width="600"/>
</div>
<p align="center"><em>Blur Auto Clicker reaching 500 CPS steadily</em></p>

Simple Mode:
- On / Off indicator (Blur logo turns green when active)
- Individual mouse button settings (left, right, middle)
- Hold / Toggle activation modes
- Customizable hotkeys

Advanced Mode (includes all Simple Mode features plus):
- Adjustable click timing (duty cycle)
- Speed Range Mode (randomizes CPS within a range)
- Corner and edge stopping (not fully implemented yet)
- Click and Time limits (stop after a set number of clicks or elapsed time)
- Double clicks
- Position Clicking (pick a position | the mouse moves there and clicks)
- Clicks adjustable to per Second, Minute, Hour, or Day

Other Features:
- Click stats (total clicks, clicks per second, etc.) | not yet implemented on Linux

---

## Installation

<div align="center">
  <a href="https://github.com/Zynix-Scripts/Blur-AutoClicker-Linux/releases/latest">
    <img src="https://github.com/machiav3lli/oandbackupx/blob/034b226cea5c1b30eb4f6a6f313e4dadcbb0ece4/badge_github.png" alt="Download from GitHub" height="75">
  </a>
</div>

This is a portable binary | no installer needed.

Config and stats are stored in `~/.local/share/BlurAutoClicker/`.

---

## Building From Source

Requirements:
- Node.js 20 or newer
- Rust via `rustup`
- Linux system dependencies for Tauri (see [Tauri prerequisites](https://tauri.app/start/prerequisites/))

Setup:
```bash
git clone https://github.com/Zynix-Scripts/Blur-AutoClicker-Linux.git
cd Blur-AutoClicker-Linux
npm install
rustup default stable
```

Run the app in development:
```bash
npm exec tauri dev
```

Build a release bundle:
```bash
npm exec tauri build
```

Useful validation commands:
```bash
npm run lint
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

The built binary and packages are written to `src-tauri/target/release/bundle/`.

---

## Support the Project

[![ko-fi](https://www.ko-fi.com/img/donate_sm.png)](https://ko-fi.com/blur009)

You can also support the project by starring the repository and sharing it with friends. Thank you!

---

## License

This project is licensed under the [GNU General Public License v3.0](https://www.gnu.org/licenses/gpl-3.0.en.html#license-text).
