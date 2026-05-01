# WavePlayer

A very minimal looking, fast Linux audio player with a SoundCloud-style waveform display. Built with Rust and iced.

![WavePlayer Screenshot](/waveplayer/screenshot.png)

## Features

- **Waveform display** — high-resolution SoundCloud-style waveform with played/unplayed coloring and reflection effect
- **Accent color theming** — waveform and UI elements match your chosen accent color
- **KDE and COSMIC integration** — automatically read your desktop accent color
- **Click/drag to scrub** — click or drag anywhere on the waveform to seek
- **Right-click to open** — right-click the waveform to open a file
- **Space bar** to play/pause
- **Single instance** — opening a file from the file manager sends it to the existing window via IPC
- **Persistent settings** — volume, accent color, and full-width mode saved across sessions
- **Full-width mode** — stretch the window to the full width of your screen
- **Settings panel** — gear icon opens inline settings without a separate window
- **Tiny footprint** — window is under 120px tall by default

## Supported Formats

MP3, WAV, FLAC, OGG, AAC, M4A, AIFF, and more via Symphonia.

## Installation

### Build from source

You need Rust installed. If you don't have it:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then clone and build:

```bash
git clone https://github.com/yourusername/waveplayer.git
cd waveplayer
cargo build --release
```

The binary will be at `target/release/waveplayer`.

### Install to PATH

```bash
sudo cp target/release/waveplayer /usr/local/bin/
```

## Usage

```bash
# Open a file directly
waveplayer /path/to/file.mp3

# Open without a file (right-click waveform to open)
waveplayer
```

### Keyboard shortcuts

| Key | Action |
|-----|--------|
| `Space` | Play / Pause |

### Mouse controls

| Action | Result |
|--------|--------|
| Left click waveform | Seek to position |
| Left drag waveform | Scrub through track |
| Right click waveform | Open file dialog |

## Settings

Click the gear icon (⚙) in the top right to open the settings panel:

- **Volume** — adjust playback volume
- **Full Width** — stretch the window to the full screen width
- **Accent color** — choose Default (orange), KDE (reads from `~/.config/kdeglobals`), or COSMIC (reads from `~/.config/cosmic/`)

Settings are saved automatically to `~/.config/waveplayer/config.toml`.

## File Manager Integration

To open files in WavePlayer from your file manager, set WavePlayer as the default application for audio files, or create a `.desktop` file:

```ini
[Desktop Entry]
Name=WavePlayer
Exec=waveplayer %f
Type=Application
MimeType=audio/mpeg;audio/wav;audio/flac;audio/ogg;audio/aac;audio/x-aiff;
Icon=multimedia-player
Categories=AudioVideo;Audio;Player;
```

Save this to `~/.local/share/applications/waveplayer.desktop`.

## Configuration

Config file location: `~/.config/waveplayer/config.toml`

```toml
volume=0.8
full_width=false
accent_r=1.0
accent_g=0.42
accent_b=0.0
```

## Dependencies

- [iced](https://github.com/iced-rs/iced) — UI framework
- [Symphonia](https://github.com/pdeljanov/Symphonia) — audio decoding and seeking
- [rodio](https://github.com/RustAudio/rodio) — audio playback
- [rfd](https://github.com/PolyMeilex/rfd) — file dialog
- [iced_fonts](https://github.com/iced-rs/iced_fonts) — Bootstrap icons

## License

MIT
