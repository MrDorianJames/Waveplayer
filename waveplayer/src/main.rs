mod audio;
mod waveform;
mod ui;
mod ipc;
mod config;

use iced::{
    application, Element, Subscription, Task, Theme,
};

pub use audio::AudioEngine;
pub use waveform::WaveformData;

static IPC_RECEIVER: std::sync::OnceLock<ipc::IpcReceiver> = std::sync::OnceLock::new();

fn get_screen_width() -> f32 {
    if let Ok(output) = std::process::Command::new("xrandr").output() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.contains(" connected") {
                for part in line.split_whitespace() {
                    if let Some(w) = part.split('x').next() {
                        if let Ok(width) = w.parse::<f32>() {
                            if width > 100.0 {
                                return width;
                            }
                        }
                    }
                }
            }
        }
    }
    1920.0
}

fn read_kde_accent() -> Option<iced::Color> {
    let path = dirs::home_dir()?.join(".config/kdeglobals");
    let contents = std::fs::read_to_string(path).ok()?;
    for line in contents.lines() {
        if line.starts_with("AccentColor=") {
            let val = line.trim_start_matches("AccentColor=");
            let parts: Vec<&str> = val.split(',').collect();
            if parts.len() == 3 {
                let r = parts[0].trim().parse::<u8>().ok()? as f32 / 255.0;
                let g = parts[1].trim().parse::<u8>().ok()? as f32 / 255.0;
                let b = parts[2].trim().parse::<u8>().ok()? as f32 / 255.0;
                return Some(iced::Color::from_rgb(r, g, b));
            }
        }
    }
    None
}

fn read_cosmic_accent() -> Option<iced::Color> {
    let base = dirs::home_dir()?.join(".config/cosmic");
    for theme in &["com.system76.CosmicTheme.Dark", "com.system76.CosmicTheme.Light"] {
        let path = base.join(theme).join("v1").join("accent");
        if let Ok(contents) = std::fs::read_to_string(&path) {
            let cleaned: String = contents.chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',' || *c == ' ')
            .collect();
            let parts: Vec<f32> = cleaned.split(|c| c == ',' || c == ' ')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
            if parts.len() >= 3 {
                return Some(iced::Color::from_rgb(parts[0], parts[1], parts[2]));
            }
        }
    }
    None
}

fn main() -> iced::Result {
    let cli_path = std::env::args()
    .nth(1)
    .map(|a| std::path::PathBuf::from(a));

    match ipc::acquire_or_send(cli_path.as_ref()) {
        None => { return Ok(()); }
        Some(receiver) => { IPC_RECEIVER.set(receiver).ok(); }
    }

    let config = config::Config::load();
    let initial_width = if config.full_width {
        get_screen_width()
    } else {
        900.0
    };

    application("WavePlayer", WavePlayer::update, WavePlayer::view)
    .theme(WavePlayer::theme)
    .subscription(WavePlayer::subscription)
    .font(iced_fonts::BOOTSTRAP_FONT_BYTES)
    .window(iced::window::Settings {
        size: iced::Size::new(initial_width, 110.0),
            resizable: true,
            decorations: true,
            ..Default::default()
    })
    .run_with(WavePlayer::new)
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenFile,
    FileOpened(Option<std::path::PathBuf>),
    AudioLoaded(Result<WaveformData, String>),
    PlayPause,
    Seek(f32),
    VolumeChanged(f32),
    Tick,
    KeyPressed(iced::keyboard::Key),
    IpcFile(std::path::PathBuf),
    EndOfStream,
    ToggleSettings,
    ToggleFullWidth,
    ResizeWindow(f32),
    SetAccentKde,
    SetAccentCosmic,
    SetAccentDefault,
}

pub struct WavePlayer {
    engine: AudioEngine,
    waveform: Option<WaveformData>,
    file_name: Option<String>,
    volume: f32,
    autoplay: bool,
    show_settings: bool,
    full_width: bool,
    accent_color: iced::Color,
}

impl WavePlayer {
    fn new() -> (Self, Task<Message>) {
        let cli_path = std::env::args()
        .nth(1)
        .map(|a| std::path::PathBuf::from(a));

        let autoplay = cli_path.is_some();
        let config = config::Config::load();

        let initial_task = if let Some(ref path) = cli_path {
            let path = path.clone();
            Task::perform(
                async move { WaveformData::from_file(&path) },
                          Message::AudioLoaded,
            )
        } else {
            Task::none()
        };

        let file_name = cli_path.as_ref().and_then(|p| {
            p.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
        });

        (
            Self {
                engine: AudioEngine::new(),
         waveform: None,
         file_name,
         volume: config.volume,
         autoplay,
         show_settings: false,
         full_width: config.full_width,
         accent_color: config.accent_color,
            },
         initial_task,
        )
    }

    fn save_config(&self) {
        config::Config {
            volume: self.volume,
            accent_color: self.accent_color,
            full_width: self.full_width,
        }.save();
    }

    fn theme(&self) -> Theme {
        Theme::Custom(
            iced::theme::Custom::new(
                "WavePlayer".to_string(),
                                     iced::theme::Palette {
                                         background: iced::Color::from_rgb(0.06, 0.07, 0.09),
                                     text: iced::Color::from_rgb(0.95, 0.95, 0.97),
                                     primary: self.accent_color,
                                     success: iced::Color::from_rgb(0.2, 0.8, 0.4),
                                     danger: iced::Color::from_rgb(0.9, 0.2, 0.2),
                                     },
            )
            .into(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFile => {
                return Task::perform(
                    async {
                        let handle = rfd::AsyncFileDialog::new()
                        .add_filter("Audio", &["mp3", "wav", "flac", "ogg", "aac", "m4a", "aiff"])
                        .set_title("Open Audio File")
                        .pick_file()
                        .await;
                        handle.map(|h| h.path().to_path_buf())
                    },
                    Message::FileOpened,
                );
            }
            Message::FileOpened(Some(path)) => {
                self.autoplay = false;
                self.waveform = None;
                self.file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string());
                let p = path.clone();
                return Task::perform(
                    async move { WaveformData::from_file(&p) },
                                     Message::AudioLoaded,
                );
            }
            Message::FileOpened(None) => {}
            Message::AudioLoaded(Ok(data)) => {
                let path = data.path.clone();
                self.file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string());
                self.waveform = Some(data);
                self.engine.load(&path);
                self.engine.set_volume(self.volume);
                if self.autoplay {
                    self.engine.play();
                    self.autoplay = false;
                }
            }
            Message::AudioLoaded(Err(e)) => {
                eprintln!("Failed to load audio: {e}");
            }
            Message::PlayPause => { self.engine.toggle_play_pause(); }
            Message::Seek(pos) => {
                if let Some(ref wf) = self.waveform {
                    self.engine.seek(pos as f64 * wf.duration_secs as f64);
                }
            }
            Message::VolumeChanged(v) => {
                self.volume = v;
                self.engine.set_volume(v);
                self.save_config();
            }
            Message::Tick => {
                if self.engine.take_ended() {
                    return self.update(Message::EndOfStream);
                }
                if let Some(receiver) = IPC_RECEIVER.get() {
                    if let Some(path) = receiver.try_recv() {
                        return self.update(Message::IpcFile(path));
                    }
                }
            }
            Message::KeyPressed(key) => {
                if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Space) {
                    self.engine.toggle_play_pause();
                }
            }
            Message::IpcFile(path) => {
                self.autoplay = true;
                self.waveform = None;
                self.file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string());
                let p = path.clone();
                return Task::perform(
                    async move { WaveformData::from_file(&p) },
                                     Message::AudioLoaded,
                );
            }
            Message::EndOfStream => {
                self.engine.rewind();
            }
            Message::ToggleSettings => {
                self.show_settings = !self.show_settings;
            }
            Message::ToggleFullWidth => {
                self.full_width = !self.full_width;
                self.save_config();
                let new_width = if self.full_width {
                    get_screen_width()
                } else {
                    900.0
                };
                return iced::window::get_oldest().then(move |id_opt| {
                    if let Some(id) = id_opt {
                        iced::window::resize(id, iced::Size::new(new_width, 110.0))
                    } else {
                        Task::none()
                    }
                });
            }
            Message::ResizeWindow(_) => {}
            Message::SetAccentKde => {
                if let Some(color) = read_kde_accent() {
                    self.accent_color = color;
                    self.save_config();
                } else {
                    eprintln!("Could not read KDE accent color");
                }
            }
            Message::SetAccentCosmic => {
                if let Some(color) = read_cosmic_accent() {
                    self.accent_color = color;
                    self.save_config();
                } else {
                    eprintln!("Could not read COSMIC accent color");
                }
            }
            Message::SetAccentDefault => {
                self.accent_color = iced::Color::from_rgb(1.0, 0.42, 0.0);
                self.save_config();
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick = iced::time::every(std::time::Duration::from_millis(33))
        .map(|_| Message::Tick);
        let keys = iced::keyboard::on_key_press(|key, _modifiers| {
            Some(Message::KeyPressed(key))
        });
        Subscription::batch(vec![tick, keys])
    }

    fn view(&self) -> Element<'_, Message> {
        let playback_pos = self.engine.position_secs();
        let duration = self.waveform.as_ref().map(|w| w.duration_secs).unwrap_or(0.0);
        let progress = if duration > 0.0 {
            (playback_pos / duration as f64).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };

        ui::build_ui(
            self.waveform.as_ref(),
                     self.file_name.as_deref(),
                     self.engine.is_playing(),
                     progress,
                     playback_pos,
                     duration,
                     self.volume,
                     self.show_settings,
                     self.full_width,
                     self.accent_color,
        )
    }
}
