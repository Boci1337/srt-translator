# SRT Translator

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/Boci1337/srt-translator?color=green)](https://github.com/Boci1337/srt-translator/releases/latest)
[![Platform](https://img.shields.io/badge/platform-Windows-0078d4?logo=windows)](https://github.com/Boci1337/srt-translator/releases/latest)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange?logo=rust)](https://www.rust-lang.org)

A simple, portable Windows desktop app that translates `.srt` subtitle files from **English to Hungarian** using Google Translate — no account, no API key required.

---

## Download

👉 **[Download latest release (.exe)](https://github.com/Boci1337/srt-translator/releases/latest)**

Single `.exe`, no installer, no dependencies. Just run it.

---

## Screenshot

![SRT Translator screenshot](https://github.com/Boci1337/srt-translator/assets/screenshot.png)

> _Window showing input/output file pickers, a live progress bar, and the Start button._

---

## Features

- 📂 **File picker** — browse for input and output `.srt` files
- 🔄 **Auto-suggests output path** — fills in `filename.hu.srt` automatically
- 📊 **Live progress bar** — shows `N / Total subtitles translated…` in real time
- ✅ **Preserves all timestamps** — only the subtitle text is translated, nothing else changes
- ⚡ **Portable** — single `.exe`, no runtime or Visual C++ redistributable needed
- 🆓 **Free** — uses the Google Translate free endpoint, no API key required

---

## Usage

1. Download `srt-translator.exe` from the [Releases](https://github.com/Boci1337/srt-translator/releases/latest) page
2. Double-click to run — no installation needed
3. Click **Browse…** next to **Input SRT** and select your English subtitle file
4. The output path is filled in automatically (you can change it if you like)
5. Click **▶ Start Translation** and wait for the progress bar to complete
6. The translated `.hu.srt` file is saved to the output path

---

## Build from Source

Requires [Rust](https://rustup.rs) (stable, 1.70+).

```sh
git clone https://github.com/Boci1337/srt-translator.git
cd srt-translator

# Debug build
cargo build

# Portable release build (statically linked, no runtime DLL needed)
$env:RUSTFLAGS="-C target-feature=+crt-static"
cargo build --release --target x86_64-pc-windows-msvc
```

The binary will be at `target/x86_64-pc-windows-msvc/release/srt-translator.exe`.

---

## How It Works

- Parses the `.srt` file into subtitle blocks (index, timestamp, text)
- Sends subtitle text to Google Translate's free endpoint in batches of 20
- Reassembles the file with translated text and original timestamps intact
- Translation runs on a background thread so the UI stays responsive throughout

---

## License

[MIT](LICENSE)
