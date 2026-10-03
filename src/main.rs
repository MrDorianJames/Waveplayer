mod audio;
mod waveform;
mod ui;
mod ipc;
mod config;
mod tags;
mod mpris;

use iced::{
    Element, Subscription, Task, Theme,
};
use iced::window;

pub use audio::AudioEngine;
pub use waveform::WaveformData;
pub use tags::TrackTags;

static IPC_RECEIVER: std::sync::OnceLock<ipc::IpcReceiver> = std::sync::OnceLock::new();
static MPRIS_TX: std::sync::OnceLock<mpris::MprisUpdateSender> = std::sync::OnceLock::new();
static MPRIS_RX: std::sync::OnceLock<std::sync::Mutex<mpris::MprisCommandReceiver>> =
    std::sync::OnceLock::new();

fn get_screen_width() -> f32 {
    if let Ok(output) = std::process::Command::new("xrandr").output() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.contains(" connected") {
                for part in line.split_whitespace() {
                    if let Some(w) = part.split('x').next() {
                        if let Ok(width) = w.parse::<f32>() {
                            if width > 100.0 { return width; }
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
        if line.starts_with("AccentColor=") || line.starts_with("LastUsedCustomAccentColor=") {
            let val = if line.starts_with("AccentColor=") {
                line.trim_start_matches("AccentColor=")
            } else {
                line.trim_start_matches("LastUsedCustomAccentColor=")
            };
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

fn audio_files_in_dir(current: &std::path::Path) -> Vec<std::path::PathBuf> {
    let extensions = ["mp3", "wav", "flac", "ogg", "aac", "m4a", "aiff"];
    let dir = match current.parent() {
        Some(d) => d,
        None => return vec![],
    };
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| extensions.contains(&e.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .collect();
    files.sort();
    files
}

fn notify_track(
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    artwork: Option<Vec<u8>>,
    track_number: u32,
) {
    std::thread::spawn(move || {
        let summary = title.as_deref().unwrap_or("WavePlayer");
        let mut body = String::new();
        if let Some(ref a) = artist { body.push_str(a); }
        if let Some(ref al) = album {
            if !body.is_empty() { body.push_str(" — "); }
            body.push_str(al);
        }
        let icon_path = if let Some(ref art) = artwork {
            let path = std::env::temp_dir()
                .join(format!("waveplayer-artwork-{}.jpg", track_number));
            if !path.exists() { let _ = std::fs::write(&path, art); }
            Some(path)
        } else {
            None
        };
        let mut notif = notify_rust::Notification::new();
        notif.summary(summary)
             .body(&body)
             .appname("WavePlayer")
             .timeout(notify_rust::Timeout::Milliseconds(3000));
        if let Some(ref path) = icon_path {
            notif.icon(path.to_string_lossy().as_ref());
        } else {
            notif.icon("audio-x-generic");
        }
        let _ = notif.show();
    });
}

fn main() -> iced::Result {
    let cli_path = std::env::args()
        .nth(1)
        .map(|a| std::path::PathBuf::from(a));

    match ipc::acquire_or_send(cli_path.as_ref()) {
        None => { return Ok(()); }
        Some(receiver) => { IPC_RECEIVER.set(receiver).ok(); }
    }

    let (tx, rx) = mpris::spawn();
    MPRIS_TX.set(tx).ok();
    MPRIS_RX.set(std::sync::Mutex::new(rx)).ok();

    let config = config::Config::load();
    let initial_width = if config.full_width { get_screen_width() } else { 900.0 };
    let initial_height = config.window_height;

    iced::daemon("WavePlayer", WavePlayer::update, WavePlayer::view)
        .theme(WavePlayer::theme)
        .subscription(WavePlayer::subscription)
        .font(iced_fonts::BOOTSTRAP_FONT_BYTES)
        .run_with(move || {
            let cli_path = std::env::args()
                .nth(1)
                .map(|a| std::path::PathBuf::from(a));
            let config = config::Config::load();
            let autoplay = cli_path.is_some();

            let load_task = if let Some(ref path) = cli_path {
                let path = path.clone();
                Task::perform(
                    async move { WaveformData::from_file(&path) },
                    Message::AudioLoaded,
                )
            } else {
                Task::none()
            };

            let file_name = cli_path.as_ref().and_then(|p| {
                p.file_name().and_then(|n| n.to_str()).map(|s| s.to_string())
            });

            let send_to_apps = config::available_send_to_apps(&config.send_to_apps);

            let state = WavePlayer {
                engine: AudioEngine::new(),
                waveform: None,
                file_name,
                current_path: cli_path.clone(),
                volume: config.volume,
                autoplay,
                loading: cli_path.is_some(),
                show_settings: false,
                show_send_to: false,
                full_width: config.full_width,
                accent_color: config.accent_color,
                energy_colors: config.energy_colors,
                low_color: config.low_color,
                mid_color: config.mid_color,
                high_color: config.high_color,
                loop_track: config.loop_track,
                auto_advance: config.auto_advance,
                seek_secs: config.seek_secs,
                notifications_enabled: config.notifications_enabled,
                notification_min_duration: config.notification_min_duration,
                window_height: config.window_height,
                send_to_apps,
                custom_send_to_apps: config.send_to_apps.clone(),
                info_window: None,
                main_window: None,
                track_tags: cli_path.as_ref().map(|p| TrackTags::from_file(p)),
                track_number: 0,
                loop_region: None,
                shift_held: false,
            };

            let (_, open_task) = window::open(window::Settings {
                size: iced::Size::new(initial_width, initial_height),
                resizable: true,
                decorations: true,
                exit_on_close_request: true,
                ..Default::default()
            });

            (state, Task::batch(vec![
                load_task,
                open_task.map(Message::MainWindowOpened),
            ]))
        })
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
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
    KeyReleased(iced::keyboard::Key, iced::keyboard::Modifiers),
    IpcFile(std::path::PathBuf),
    EndOfStream,
    ToggleSettings,
    ToggleSendTo,
    SendFileTo(String), // command string
    ToggleFullWidth,
    ToggleEnergyColors,
    ToggleLoopTrack,
    ToggleAutoAdvance,
    ToggleNotifications,
    ResizeWindow(f32, f32),
    SetAccentKde,
    SetAccentCosmic,
    SetAccentDefault,
    ShowInfo,
    CloseInfo,
    MainWindowOpened(window::Id),
    InfoWindowOpened(window::Id),
    WindowClosed(window::Id),
    PlayNext,
    PlayPrev,
    MprisCommand(mpris::MprisCommand),
    SetLoopStart(f32),
    SetLoopEnd(f32),
    ClearLoopRegion,
}

pub struct WavePlayer {
    engine: AudioEngine,
    waveform: Option<WaveformData>,
    file_name: Option<String>,
    current_path: Option<std::path::PathBuf>,
    volume: f32,
    autoplay: bool,
    loading: bool,
    show_settings: bool,
    show_send_to: bool,
    full_width: bool,
    accent_color: iced::Color,
    energy_colors: bool,
    low_color: iced::Color,
    mid_color: iced::Color,
    high_color: iced::Color,
    loop_track: bool,
    auto_advance: bool,
    seek_secs: f64,
    notifications_enabled: bool,
    notification_min_duration: f64,
    window_height: f32,
    send_to_apps: Vec<config::SendToApp>,
    custom_send_to_apps: Vec<String>,
    info_window: Option<window::Id>,
    main_window: Option<window::Id>,
    track_tags: Option<TrackTags>,
    track_number: u32,
    loop_region: Option<(f32, f32)>,
    shift_held: bool,
}

impl WavePlayer {
    fn save_config(&self) {
        config::Config {
            volume: self.volume,
            accent_color: self.accent_color,
            full_width: self.full_width,
            energy_colors: self.energy_colors,
            low_color: self.low_color,
            mid_color: self.mid_color,
            high_color: self.high_color,
            loop_track: self.loop_track,
            auto_advance: self.auto_advance,
            seek_secs: self.seek_secs,
            notifications_enabled: self.notifications_enabled,
            notification_min_duration: self.notification_min_duration,
            window_height: self.window_height,
            send_to_apps: self.custom_send_to_apps.clone(),
        }.save();
    }

    fn theme(&self, _window: window::Id) -> Theme {
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

    fn load_path(&mut self, path: std::path::PathBuf, autoplay: bool) -> Task<Message> {
        self.autoplay = autoplay;
        self.loading = true;
        self.waveform = None;
        self.loop_region = None;
        self.engine.set_loop_region(None);
        self.current_path = Some(path.clone());
        self.file_name = path.file_name().and_then(|n| n.to_str()).map(|s| s.to_string());
        self.track_tags = Some(TrackTags::from_file(&path));
        self.track_number = self.track_number.wrapping_add(1);
        Task::perform(
            async move { WaveformData::from_file(&path) },
            Message::AudioLoaded,
        )
    }

    fn play_adjacent(&mut self, next: bool) -> Task<Message> {
        let current = match &self.current_path {
            Some(p) => p.clone(),
            None => return Task::none(),
        };
        let files = audio_files_in_dir(&current);
        if files.is_empty() { return Task::none(); }
        let pos = files.iter().position(|f| f == &current);
        let new_path = match pos {
            Some(i) => {
                if next {
                    files.get(i + 1).or_else(|| files.first())
                } else {
                    if i == 0 { files.last() } else { files.get(i - 1) }
                }
            }
            None => files.first(),
        };
        if let Some(path) = new_path.cloned() {
            return self.load_path(path, true);
        }
        Task::none()
    }

    fn send_mpris_metadata(&self) {
        if let Some(tx) = MPRIS_TX.get() {
            let _ = tx.try_send(mpris::MprisUpdate::Metadata {
                title: self.track_tags.as_ref().and_then(|t| t.title.clone())
                    .or_else(|| self.file_name.clone()),
                artist: self.track_tags.as_ref().and_then(|t| t.artist.clone()),
                album: self.track_tags.as_ref().and_then(|t| t.album.clone()),
                duration_secs: self.waveform.as_ref()
                    .map(|w| w.duration_secs as f64).unwrap_or(0.0),
                is_playing: self.engine.is_playing(),
                artwork: self.track_tags.as_ref().and_then(|t| t.artwork.clone()),
                track_number: self.track_number,
            });
        }
    }

    fn send_mpris_status(&self) {
        if let Some(tx) = MPRIS_TX.get() {
            let _ = tx.try_send(mpris::MprisUpdate::PlaybackStatus(
                self.engine.is_playing()
            ));
        }
    }

    fn progress_to_secs(&self, progress: f32) -> f64 {
        let dur = self.waveform.as_ref().map(|w| w.duration_secs as f64).unwrap_or(0.0);
        progress as f64 * dur
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
                return self.load_path(path, false);
            }
            Message::FileOpened(None) => {}
            Message::AudioLoaded(Ok(data)) => {
                let path = data.path.clone();
                let duration = data.duration_secs as f64;
                self.file_name = path.file_name().and_then(|n| n.to_str()).map(|s| s.to_string());
                self.waveform = Some(data);
                self.loading = false;
                self.engine.load(&path);
                self.engine.set_volume(self.volume);
                if self.autoplay {
                    self.engine.play();
                    self.autoplay = false;
                }
                self.send_mpris_metadata();
                if self.notifications_enabled && duration >= self.notification_min_duration {
                    notify_track(
                        self.track_tags.as_ref().and_then(|t| t.title.clone())
                            .or_else(|| self.file_name.clone()),
                        self.track_tags.as_ref().and_then(|t| t.artist.clone()),
                        self.track_tags.as_ref().and_then(|t| t.album.clone()),
                        self.track_tags.as_ref().and_then(|t| t.artwork.clone()),
                        self.track_number,
                    );
                }
            }
            Message::AudioLoaded(Err(e)) => {
                eprintln!("Failed to load audio: {e}");
                self.loading = false;
            }
            Message::PlayPause => {
                self.engine.toggle_play_pause();
                self.send_mpris_status();
            }
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
                if !self.loading && self.engine.take_ended() {
                    return self.update(Message::EndOfStream);
                }
                if let Some(rx) = MPRIS_RX.get() {
                    if let Ok(rx) = rx.try_lock() {
                        if let Ok(cmd) = rx.try_recv() {
                            drop(rx);
                            return self.update(Message::MprisCommand(cmd));
                        }
                    }
                }
                if let Some(receiver) = IPC_RECEIVER.get() {
                    if let Some(path) = receiver.try_recv() {
                        return self.update(Message::IpcFile(path));
                    }
                }
                if self.engine.is_playing() {
                    if let Some(tx) = MPRIS_TX.get() {
                        let _ = tx.try_send(mpris::MprisUpdate::Position(
                            self.engine.position_secs()
                        ));
                    }
                }
            }
            Message::MprisCommand(cmd) => {
                use mpris::MprisCommand;
                match cmd {
                    MprisCommand::Play => { self.engine.play(); self.send_mpris_status(); }
                    MprisCommand::Pause => {
                        if self.engine.is_playing() { self.engine.toggle_play_pause(); self.send_mpris_status(); }
                    }
                    MprisCommand::PlayPause => { self.engine.toggle_play_pause(); self.send_mpris_status(); }
                    MprisCommand::Stop => { self.engine.rewind(); self.send_mpris_status(); }
                    MprisCommand::Next => { return self.play_adjacent(true); }
                    MprisCommand::Previous => { return self.play_adjacent(false); }
                    MprisCommand::Seek(offset_secs) => {
                        let dur = self.waveform.as_ref().map(|w| w.duration_secs as f64).unwrap_or(0.0);
                        let pos = (self.engine.position_secs() + offset_secs).clamp(0.0, dur);
                        self.engine.seek(pos);
                    }
                    MprisCommand::SetPosition(secs) => {
                        let dur = self.waveform.as_ref().map(|w| w.duration_secs as f64).unwrap_or(0.0);
                        self.engine.seek(secs.clamp(0.0, dur));
                    }
                }
            }
            Message::KeyPressed(key, modifiers) => {
                if modifiers.shift() { self.shift_held = true; }
                use iced::keyboard::key::Named;
                match key {
                    iced::keyboard::Key::Named(Named::Space) => {
                        self.engine.toggle_play_pause();
                        self.send_mpris_status();
                    }
                    iced::keyboard::Key::Named(Named::ArrowLeft) => {
                        let pos = (self.engine.position_secs() - self.seek_secs).max(0.0);
                        self.engine.seek(pos);
                    }
                    iced::keyboard::Key::Named(Named::ArrowRight) => {
                        let dur = self.waveform.as_ref().map(|w| w.duration_secs as f64).unwrap_or(0.0);
                        let pos = (self.engine.position_secs() + self.seek_secs).min(dur);
                        self.engine.seek(pos);
                    }
                    iced::keyboard::Key::Named(Named::ArrowUp) => {
                        return self.play_adjacent(false);
                    }
                    iced::keyboard::Key::Named(Named::ArrowDown) => {
                        return self.play_adjacent(true);
                    }
                    iced::keyboard::Key::Named(Named::Escape) => {
                        if self.loop_region.is_some() {
                            return self.update(Message::ClearLoopRegion);
                        }
                    }
                    iced::keyboard::Key::Named(Named::Shift) => {
                        self.shift_held = true;
                    }
                    iced::keyboard::Key::Character(ref c) if c.as_str() == "e" && modifiers.control() => {
                        return self.update(Message::ToggleSendTo);
                    }
                    _ => {}
                }
            }
            Message::KeyReleased(key, _modifiers) => {
                use iced::keyboard::key::Named;
                if let iced::keyboard::Key::Named(Named::Shift) = key {
                    self.shift_held = false;
                }
            }
            Message::IpcFile(path) => {
                return self.load_path(path, true);
            }
            Message::EndOfStream => {
                if self.loop_track {
                    self.engine.restart_from_start(true);
                    if let Some(tx) = MPRIS_TX.get() {
                        let _ = tx.try_send(mpris::MprisUpdate::Position(0.0));
                    }
                } else if self.auto_advance {
                    return self.play_adjacent(true);
                } else {
                    self.engine.rewind();
                }
                self.send_mpris_status();
            }
            Message::SetLoopStart(pos) => {
                self.loop_region = Some((pos, pos));
                self.engine.set_loop_region(None);
            }
            Message::SetLoopEnd(pos) => {
                if let Some((start, _)) = self.loop_region {
                    let (a, b) = if pos >= start { (start, pos) } else { (pos, start) };
                    if b - a > 0.01 {
                        self.loop_region = Some((a, b));
                        let start_secs = self.progress_to_secs(a);
                        let end_secs = self.progress_to_secs(b);
                        self.engine.set_loop_region(Some((start_secs, end_secs)));
                        let pos_secs = self.engine.position_secs();
                        if pos_secs < start_secs || pos_secs > end_secs {
                            self.engine.seek(start_secs);
                        }
                    }
                }
            }
            Message::ClearLoopRegion => {
                self.loop_region = None;
                self.engine.set_loop_region(None);
            }
            Message::ToggleSettings => {
                self.show_settings = !self.show_settings;
                if self.show_settings { self.show_send_to = false; }
            }
            Message::ToggleSendTo => {
                self.show_send_to = !self.show_send_to;
                if self.show_send_to { self.show_settings = false; }
            }
            Message::SendFileTo(cmd) => {
                if let Some(ref path) = self.current_path {
                    config::send_file_to_app(&cmd, path);
                }
                self.show_send_to = false;
            }
            Message::ToggleEnergyColors => { self.energy_colors = !self.energy_colors; self.save_config(); }
            Message::ToggleLoopTrack => {
                self.loop_track = !self.loop_track;
                if self.loop_track { self.auto_advance = false; }
                self.save_config();
            }
            Message::ToggleAutoAdvance => {
                self.auto_advance = !self.auto_advance;
                if self.auto_advance { self.loop_track = false; }
                self.save_config();
            }
            Message::ToggleNotifications => { self.notifications_enabled = !self.notifications_enabled; self.save_config(); }
            Message::ToggleFullWidth => {
                self.full_width = !self.full_width;
                self.save_config();
                let new_width = if self.full_width { get_screen_width() } else { 900.0 };
                if let Some(id) = self.main_window {
                    return iced::window::resize(id, iced::Size::new(new_width, self.window_height));
                }
            }
            Message::ResizeWindow(w, h) => {
                // Save height whenever window is resized
                if h >= 80.0 && (h - self.window_height).abs() > 1.0 {
                    self.window_height = h;
                    self.save_config();
                }
            }
            Message::SetAccentKde => {
                if let Some(color) = read_kde_accent() { self.accent_color = color; self.save_config(); }
            }
            Message::SetAccentCosmic => {
                if let Some(color) = read_cosmic_accent() { self.accent_color = color; self.save_config(); }
            }
            Message::SetAccentDefault => {
                self.accent_color = iced::Color::from_rgb(1.0, 0.42, 0.0);
                self.save_config();
            }
            Message::ShowInfo => {
                if self.info_window.is_some() { return Task::none(); }
                let (id, task) = window::open(window::Settings {
                    size: iced::Size::new(420.0, 520.0),
                    resizable: false,
                    decorations: true,
                    exit_on_close_request: false,
                    ..Default::default()
                });
                self.info_window = Some(id);
                return task.map(Message::InfoWindowOpened);
            }
            Message::MainWindowOpened(id) => { self.main_window = Some(id); }
            Message::InfoWindowOpened(_) => {}
            Message::CloseInfo => {
                if let Some(id) = self.info_window.take() { return window::close(id); }
            }
            Message::WindowClosed(id) => {
                if Some(id) == self.info_window {
                    self.info_window = None;
                    return window::close(id);
                } else if Some(id) == self.main_window || self.main_window.is_none() {
                    std::process::exit(0);
                }
            }
            Message::PlayNext => { return self.play_adjacent(true); }
            Message::PlayPrev => { return self.play_adjacent(false); }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick = iced::time::every(std::time::Duration::from_millis(33))
            .map(|_| Message::Tick);
        let keys = iced::keyboard::on_key_press(|key, modifiers| {
            Some(Message::KeyPressed(key, modifiers))
        });
        let keys_up = iced::keyboard::on_key_release(|key, modifiers| {
            Some(Message::KeyReleased(key, modifiers))
        });
        let window_events = iced::event::listen_with(|event, _status, id| {
            match event {
                iced::Event::Window(iced::window::Event::CloseRequested) => {
                    Some(Message::WindowClosed(id))
                }
                iced::Event::Window(iced::window::Event::Closed) => {
                    Some(Message::WindowClosed(id))
                }
                iced::Event::Window(iced::window::Event::Resized(size)) => {
                    Some(Message::ResizeWindow(size.width, size.height))
                }
                _ => None,
            }
        });
        Subscription::batch(vec![tick, keys, keys_up, window_events])
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if Some(window_id) != self.main_window {
            return ui::build_info_window(
                self.track_tags.as_ref(),
                self.file_name.as_deref(),
                self.accent_color,
                self.waveform.as_ref(),
                self.current_path.as_ref(),
            );
        }

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
            self.show_send_to,
            self.full_width,
            self.accent_color,
            self.energy_colors,
            self.low_color,
            self.mid_color,
            self.high_color,
            self.loop_track,
            self.auto_advance,
            self.notifications_enabled,
            self.loop_region,
            self.shift_held,
            &self.send_to_apps,
            self.current_path.is_some(),
        )
    }
}
