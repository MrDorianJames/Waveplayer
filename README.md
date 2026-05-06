# WavePlayer 🎧

A Linux desktop audio player with a SoundCloud-style waveform display, built in Rust with the [iced](https://github.com/iced-rs/iced) GUI toolkit.

## Features

- 🎵 Plays **MP3, FLAC, WAV, OGG, AAC** via rodio + symphonia
- 📊 Real waveform decoded from audio samples — not fake bars
- 🟠 Played vs. unplayed colouring with a scrubber line (SoundCloud style)
- 🪞 Reflection effect below the waveform
- ▶ Play / Pause / Stop controls
- 🔊 Volume slider
- ⏱ Current position / total duration display
- 📂 Native file-open dialog (via rfd)

## Prerequisites

```bash
# Rust (stable, 1.75+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# System libraries (Debian/Ubuntu)
sudo apt install -y \
    libasound2-dev \          # ALSA audio
    libgtk-3-dev \            # needed by rfd for the file dialog
    pkg-config \
    build-essential
```

## Fonts

The project embeds Inter fonts. Download them and place them at:

```
assets/fonts/Inter-Regular.ttf
assets/fonts/Inter-Bold.ttf
```

Get them from: https://rsms.me/inter/

Or run this one-liner:

```bash
mkdir -p assets/fonts
curl -L "https://github.com/rsms/inter/releases/download/v4.0/Inter-4.0.zip" \
     -o /tmp/inter.zip
unzip -j /tmp/inter.zip "Inter Desktop/Inter-Regular.ttf" \
                         "Inter Desktop/Inter-Bold.ttf" \
     -d assets/fonts/
```

## Build & Run

```bash
# Debug (faster compile)
cargo run

# Release (optimised)
cargo build --release
./target/release/waveplayer
```

## Project Layout

```
waveplayer/
├── Cargo.toml
├── assets/
│   └── fonts/
│       ├── Inter-Regular.ttf
│       └── Inter-Bold.ttf
└── src/
    ├── main.rs       — app entry point, message loop
    ├── audio.rs      — rodio playback engine
    ├── waveform.rs   — symphonia decoder → peak/RMS buckets
    └── ui.rs         — iced layout, canvas waveform renderer
```

## Architecture

| Layer | Crate | Role |
|---|---|---|
| GUI | `iced` 0.13 | Elm-style update/view, canvas 2D drawing |
| Playback | `rodio` 0.19 | Audio sink, volume, play/pause |
| Decoding | `symphonia` 0.5 | Decodes all formats → PCM samples for waveform |
| File dialog | `rfd` 0.15 | Native async file picker |

## Extending

- **Seek by clicking the waveform**: Add mouse-event handling to the `canvas::Program::update` method in `ui.rs`. Map the click X position → progress ratio → `Message::Seek`.
- **Playlist**: Add a `Vec<PathBuf>` to `WavePlayer` and prev/next buttons.
- **Metadata**: Use `lofty` crate to read ID3/Vorbis tags and display artist/album art.
