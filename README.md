# WavePlayer

A Linux audio player built for music and audio production workflows, featuring a SoundCloud-style waveform display, MPRIS2 integration, and direct DAW send-to support.

## Features

### Waveform Display
- 2048-bucket waveform with peak amplitude and RMS rendering
- Reflection effect below the waveform
- Playhead scrubber with soft glow
- **Energy color mode** — each bar is colored by its frequency content: bass (red), mids (green), highs (blue). The mix of frequencies determines the final bar color, giving a visual fingerprint of the audio
- Played/unplayed sections clearly distinguished in both standard and energy modes

### Playback
- Play, pause, seek, and scrub via waveform click/drag
- **Loop track** — loops the current file indefinitely
- **Auto-advance** — automatically plays the next file in the same directory when the track ends
- **Region loop** — set a loop region with Shift+drag on the waveform; audio loops between the two markers. Clear with Escape or the header ✕ button
- End-of-track behavior: rewinds to start paused when neither loop nor auto-advance is active, ready to play again with Space

### File Management
- Right-click on the waveform to open the file dialog
- Single instance — opening WavePlayer with a file while it is already running passes the file to the running instance instead of opening a second one
- Remembers the last window height between sessions

### MPRIS2 Integration
- Full KDE taskbar media widget support — play, pause, next, previous, seek
- Live seekbar that tracks playback position in real time
- Album artwork displayed in the taskbar widget, updated per track
- Media keyboard keys work system-wide
- Track metadata (title, artist, album, duration) sent to the desktop

### Desktop Notifications
- Fires a notification when a new track loads, showing title, artist, album, and album artwork
- **Notify toggle** — enable or disable notifications from the settings panel
- **Minimum duration** — notifications only fire for tracks longer than a configurable threshold (default 60 seconds), preventing spam when browsing short files. Set `notification_min_duration=` in the config file

### Send to DAW / Application
- Open the current file directly in another application from within WavePlayer
- Automatically detects installed apps: Audacity, Tenacity, Ardour, REAPER, Bitwig Studio, LMMS, Mixxx, Qtractor, Rosegarden, Kdenlive, ocenaudio
- Add custom apps in the config file with full command-line argument support
- Toggle the send-to panel with the **→** button in the header or **Ctrl+E**

### File Info Window
- Click the filename in the header to open the info window
- Shows album artwork, title, artist, album, year, track number, genre, and comment tags
- Technical section: codec, sample rate, channels, bit depth, bitrate, duration, file size, and file location

### Settings Panel
Click the **⚙** gear icon in the header to open the settings panel.

| Setting | Description |
|---------|-------------|
| Volume slider | Adjusts playback volume 0–100% |
| Full Width | Resizes the window to match your screen width using xrandr |
| Energy: On/Off | Toggles frequency-based energy color mode on the waveform |
| Loop: On/Off | Loops the current track when it ends |
| Advance: On/Off | Automatically plays the next file in the directory when the track ends |
| Notify: On/Off | Enables or disables desktop notifications on track load |
| Accent: Default | Resets the accent color to the default orange |
| Accent: KDE | Reads your KDE accent color from ~/.config/kdeglobals |
| Accent: COSMIC | Reads your COSMIC desktop accent color |

## Controls

| Input | Action |
|-------|--------|
| Space | Play / Pause |
| Left Arrow | Seek back N seconds (default 5) |
| Right Arrow | Seek forward N seconds (default 5) |
| Up Arrow | Previous file in directory |
| Down Arrow | Next file in directory |
| Left-click waveform | Seek to position |
| Left-click drag | Scrub playhead |
| Shift + left-click drag | Set loop region |
| Escape | Clear loop region |
| Right-click waveform | Open file dialog |
| Click filename | Open file info window |
| Ctrl+E | Toggle send-to panel |

## Installation

### Build from source

```bash
git clone https://github.com/MrDorianJames/waveplayer
cd waveplayer
cargo build --release
./target/release/waveplayer /path/to/file.wav
```

### Desktop integration

Installs WavePlayer as a file manager handler for audio files:

```bash
cp waveplayer.desktop ~/.local/share/applications/
update-desktop-database ~/.local/share/applications/
```

Edit the `Exec=` line in `waveplayer.desktop` to point at your binary path if needed.

## Supported Formats

MP3, WAV, FLAC, OGG, AAC, M4A, AIFF

## Configuration

Config file: `~/.config/waveplayer/config.toml`

```toml
volume=0.8
full_width=false
energy_colors=true
loop_track=false
auto_advance=false
seek_secs=5
notifications_enabled=true
notification_min_duration=60
window_height=110
send_to_apps=
accent_r=1
accent_g=0.42
accent_b=0
low_r=1
low_g=0.2
low_b=0
mid_r=0.2
mid_g=0.8
mid_b=0.2
high_r=0.2
high_g=0.4
high_b=1
```

### Config options

| Key | Description |
|-----|-------------|
| `volume` | Playback volume, 0.0–1.0 |
| `full_width` | Start in full-width mode |
| `energy_colors` | Enable frequency energy color mode |
| `loop_track` | Loop the current track |
| `auto_advance` | Play next file in directory on track end |
| `seek_secs` | How many seconds Left/Right arrow keys seek |
| `notifications_enabled` | Enable desktop notifications |
| `notification_min_duration` | Minimum track length in seconds before a notification fires |
| `window_height` | Saved window height in pixels |
| `send_to_apps` | Comma-separated custom app entries (see below) |
| `accent_r/g/b` | Accent color as RGB floats 0.0–1.0 |
| `low_r/g/b` | Energy color for bass frequencies |
| `mid_r/g/b` | Energy color for mid frequencies |
| `high_r/g/b` | Energy color for high frequencies |

### Custom send-to apps

Format: `Name:command %f` where `%f` is replaced with the file path at launch.
If `%f` is omitted the file is appended as the last argument.

```toml
send_to_apps=REAPER:reaper %f,Bitwig:bitwig-studio %f,MyTool:/usr/local/bin/mytool --open %f
```

## Dependencies

- [iced](https://github.com/iced-rs/iced) 0.13 — UI framework
- [symphonia](https://github.com/pdeljanov/Symphonia) — audio decoding and byte-accurate seeking
- [rodio](https://github.com/RustAudio/rodio) — audio output
- [mpris-server](https://github.com/SeaDve/mpris-server) — MPRIS2 D-Bus integration
- [lofty](https://github.com/Serial-ATA/lofty-rs) — tag and artwork reading
- [notify-rust](https://github.com/hoodie/notify-rust) — desktop notifications
- [rfd](https://github.com/PolyMeilex/rfd) — native file dialog
- [rustfft](https://github.com/ejmahler/RustFFT) — frequency analysis for energy colors

## License

MIT
