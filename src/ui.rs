use iced::{
    widget::{
        button, canvas, canvas::Cache, canvas::Frame, canvas::Geometry,
        column, container, image, row, scrollable, slider, text, Space,
    },
    Alignment, Background, Border, Color, Element, Font, Length, Pixels, Point, Rectangle, Size,
    Theme,
};
use iced_fonts::{bootstrap::icon_to_char, Bootstrap, BOOTSTRAP_FONT};

use crate::{Message, WaveformData};
use crate::tags::TrackTags;
use crate::config::SendToApp;

const BG_DEEP: Color = Color::from_rgb(0.06, 0.07, 0.09);
const BG_CARD: Color = Color::from_rgb(0.0, 0.0, 0.0);
const WAVEFORM_UNPLAYED: Color = Color::from_rgb(0.30, 0.32, 0.38);
const WAVEFORM_UNPLAYED_REFL: Color = Color::from_rgba(0.30, 0.32, 0.38, 0.20);
const TEXT_PRIMARY: Color = Color::from_rgb(0.95, 0.95, 0.97);
const TEXT_SECONDARY: Color = Color::from_rgb(0.50, 0.53, 0.60);
const SCRUBBER: Color = Color::from_rgb(1.0, 1.0, 1.0);
const PLAYED_BG: Color = Color::from_rgba(0.35, 0.35, 0.35, 0.5);
const REGION_FILL: Color = Color::from_rgba(0.2, 0.8, 1.0, 0.15);
const REGION_BORDER: Color = Color::from_rgba(0.2, 0.8, 1.0, 0.9);

fn format_file_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

pub fn build_info_window<'a>(
    tags: Option<&TrackTags>,
    file_name: Option<&'a str>,
    accent_color: Color,
    waveform: Option<&WaveformData>,
    current_path: Option<&std::path::PathBuf>,
) -> Element<'a, Message> {
    let tags = match tags {
        Some(t) => t,
        None => {
            return container(
                text("No tag information available.")
                    .color(TEXT_SECONDARY)
                    .size(14),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .style(|_| container::Style {
                background: Some(Background::Color(BG_DEEP)),
                ..Default::default()
            })
            .into();
        }
    };

    let art: Element<Message> = if let Some(ref bytes) = tags.artwork {
        let handle = image::Handle::from_bytes(bytes.clone());
        container(
            image(handle)
                .width(Length::Fixed(180.0))
                .height(Length::Fixed(180.0)),
        )
        .style(|_| container::Style {
            border: Border { radius: 6.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
    } else {
        container(
            text(icon_to_char(Bootstrap::MusicNoteBeamed).to_string())
                .font(BOOTSTRAP_FONT)
                .size(56)
                .color(TEXT_SECONDARY),
        )
        .width(Length::Fixed(180.0))
        .height(Length::Fixed(180.0))
        .center_x(Length::Fixed(180.0))
        .center_y(Length::Fixed(180.0))
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.12))),
            border: Border { radius: 6.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
    };

    let left_col = column![art].spacing(0);
    let title_text = tags.title.as_deref().or(file_name).unwrap_or("Unknown");

    let info_block = |label: &'static str, val: Option<&String>| -> Element<Message> {
        if let Some(v) = val {
            column![
                text(label).size(10).color(TEXT_SECONDARY),
                text(v.clone()).size(13).color(TEXT_PRIMARY),
                Space::with_height(8),
            ]
            .spacing(1)
            .into()
        } else {
            Space::with_height(0).into()
        }
    };

    let info_block_str = |label: &'static str, val: Option<String>| -> Element<Message> {
        if let Some(v) = val {
            column![
                text(label).size(10).color(TEXT_SECONDARY),
                text(v).size(13).color(TEXT_PRIMARY),
                Space::with_height(8),
            ]
            .spacing(1)
            .into()
        } else {
            Space::with_height(0).into()
        }
    };

    let tech_section: Element<Message> = if let Some(wf) = waveform {
        let channel_str = match wf.channels {
            1 => "Mono".to_string(),
            2 => "Stereo".to_string(),
            n => format!("{} channels", n),
        };
        let sample_rate_str = format!("{} Hz", wf.sample_rate);
        let bit_depth_str = wf.bit_depth.map(|b| format!("{}-bit", b));
        let bitrate_str = wf.bitrate_kbps.map(|b| format!("{} kbps", b));
        let codec_str = Some(wf.codec.clone());
        let total = wf.duration_secs as u32;
        let duration_str = Some(format!(
            "{}:{:02}.{}",
            total / 60, total % 60,
            ((wf.duration_secs - total as f32) * 10.0) as u32
        ));
        let file_size_str = if wf.file_size > 0 { Some(format_file_size(wf.file_size)) } else { None };
        let file_path_str = current_path
            .and_then(|p| p.parent())
            .and_then(|p| p.to_str())
            .map(|s| s.to_string());

        column![
            container(Space::with_height(1))
                .width(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08))),
                    ..Default::default()
                }),
            Space::with_height(10),
            text("TECHNICAL").size(10).color(TEXT_SECONDARY),
            Space::with_height(8),
            info_block_str("CODEC", codec_str),
            info_block_str("SAMPLE RATE", Some(sample_rate_str)),
            info_block_str("CHANNELS", Some(channel_str)),
            info_block_str("BIT DEPTH", bit_depth_str),
            info_block_str("BITRATE", bitrate_str),
            info_block_str("DURATION", duration_str),
            info_block_str("FILE SIZE", file_size_str),
            info_block_str("LOCATION", file_path_str),
        ]
        .spacing(0)
        .into()
    } else {
        Space::with_height(0).into()
    };

    let right_col = column![
        text(title_text.to_string())
            .size(15)
            .color(TEXT_PRIMARY)
            .font(Font { weight: iced::font::Weight::Bold, ..Font::DEFAULT }),
        Space::with_height(12),
        info_block("ARTIST", tags.artist.as_ref()),
        info_block("ALBUM", tags.album.as_ref()),
        info_block("YEAR", tags.year.as_ref()),
        info_block("TRACK", tags.track.as_ref()),
        info_block("GENRE", tags.genre.as_ref()),
        info_block("COMMENT", tags.comment.as_ref()),
        tech_section,
    ]
    .spacing(0)
    .width(Length::Fill);

    let close_btn = button(text("Close").size(13).color(TEXT_PRIMARY))
        .on_press(Message::CloseInfo)
        .style(move |_, status| button::Style {
            background: Some(Background::Color(match status {
                button::Status::Hovered | button::Status::Pressed =>
                    Color::from_rgba(accent_color.r, accent_color.g, accent_color.b, 0.8),
                _ => Color { a: 1.0, ..accent_color },
            })),
            border: Border { radius: 4.0.into(), ..Default::default() },
            text_color: TEXT_PRIMARY,
            ..Default::default()
        })
        .padding([6, 20]);

    let two_col = row![
        left_col,
        Space::with_width(20),
        scrollable(right_col).height(Length::Fill),
    ]
    .align_y(Alignment::Start)
    .height(Length::Fill);

    let content = column![
        two_col,
        Space::with_height(12),
        container(close_btn).width(Length::Fill).center_x(Length::Fill),
    ]
    .padding([20, 20])
    .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(BG_DEEP)),
            ..Default::default()
        })
        .into()
}

pub fn build_ui<'a>(
    waveform: Option<&WaveformData>,
    file_name: Option<&'a str>,
    is_playing: bool,
    progress: f32,
    position_secs: f64,
    duration_secs: f32,
    volume: f32,
    show_settings: bool,
    show_send_to: bool,
    full_width: bool,
    accent_color: Color,
    energy_colors: bool,
    low_color: Color,
    mid_color: Color,
    high_color: Color,
    loop_track: bool,
    auto_advance: bool,
    notifications_enabled: bool,
    loop_region: Option<(f32, f32)>,
    shift_held: bool,
    send_to_apps: &[SendToApp],
    has_file: bool,
) -> Element<'a, Message> {
    let header = build_header(
        file_name, position_secs, duration_secs, is_playing,
        show_settings, show_send_to, accent_color, loop_region,
    );
    let wave = build_waveform_canvas(
        waveform, progress, accent_color, energy_colors,
        low_color, mid_color, high_color, loop_region, shift_held,
    );

    let mut col = column![header, wave].spacing(0).width(Length::Fill);

    if show_settings {
        col = col.push(build_settings_panel(
            volume, full_width, accent_color, energy_colors,
            loop_track, auto_advance, notifications_enabled,
        ));
    }

    if show_send_to {
        col = col.push(build_send_to_panel(send_to_apps, accent_color, has_file));
    }

    container(col)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(BG_DEEP)),
            ..Default::default()
        })
        .into()
}

fn build_header<'a>(
    file_name: Option<&'a str>,
    pos: f64,
    dur: f32,
    _is_playing: bool,
    show_settings: bool,
    show_send_to: bool,
    accent_color: Color,
    loop_region: Option<(f32, f32)>,
) -> Element<'a, Message> {
    let title_btn = button(
        text(file_name.unwrap_or("Right-click waveform to open a file"))
            .size(12)
            .color(TEXT_PRIMARY)
            .font(Font { weight: iced::font::Weight::Bold, ..Font::DEFAULT }),
    )
    .on_press(Message::ShowInfo)
    .style(|_, status| button::Style {
        background: match status {
            button::Status::Hovered => Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.05))),
            _ => None,
        },
        text_color: TEXT_PRIMARY,
        border: Border { radius: 3.0.into(), ..Default::default() },
        ..Default::default()
    })
    .padding([2, 4]);

    let time_label = text(format!(
        "{} / {}",
        format_time_ms(pos),
        format_time(dur)
    ))
    .size(11)
    .color(TEXT_SECONDARY);

    let region_label: Element<Message> = if let Some((a, b)) = loop_region {
        let region_pct = ((b - a) * 100.0) as u32;
        button(
            text(format!("⌁ {}%  ✕", region_pct))
                .size(11)
                .color(REGION_BORDER),
        )
        .on_press(Message::ClearLoopRegion)
        .style(|_, status| button::Style {
            background: match status {
                button::Status::Hovered => Some(Background::Color(Color::from_rgba(0.2, 0.8, 1.0, 0.1))),
                _ => None,
            },
            border: Border { radius: 3.0.into(), ..Default::default() },
            text_color: REGION_BORDER,
            ..Default::default()
        })
        .padding([2, 6])
        .into()
    } else {
        Space::with_width(0).into()
    };

    // Send-to arrow button
    let send_btn = button(
        text(icon_to_char(Bootstrap::BoxArrowRight).to_string())
            .font(BOOTSTRAP_FONT)
            .size(13)
            .color(if show_send_to { accent_color } else { TEXT_SECONDARY }),
    )
    .on_press(Message::ToggleSendTo)
    .style(|_, _| button::Style {
        background: None,
        text_color: TEXT_SECONDARY,
        ..Default::default()
    })
    .padding([2, 6]);

    let gear_btn = button(
        text(icon_to_char(Bootstrap::GearFill).to_string())
            .font(BOOTSTRAP_FONT)
            .size(13)
            .color(if show_settings { accent_color } else { TEXT_SECONDARY }),
    )
    .on_press(Message::ToggleSettings)
    .style(|_, _| button::Style {
        background: None,
        text_color: TEXT_SECONDARY,
        ..Default::default()
    })
    .padding([2, 6]);

    let r = row![
        title_btn,
        Space::with_width(8),
        time_label,
        Space::with_width(Length::Fill),
        region_label,
        Space::with_width(4),
        send_btn,
        Space::with_width(2),
        gear_btn,
    ]
    .align_y(Alignment::Center)
    .spacing(0)
    .padding([4u16, 10]);

    container(r)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(BG_CARD)),
            ..Default::default()
        })
        .into()
}

fn build_send_to_panel<'a>(
    apps: &[SendToApp],
    accent_color: Color,
    has_file: bool,
) -> Element<'a, Message> {
    let btn_style = move |status: button::Status, enabled: bool| button::Style {
        background: Some(Background::Color(match status {
            button::Status::Hovered | button::Status::Pressed if enabled =>
                Color::from_rgba(accent_color.r, accent_color.g, accent_color.b, 0.25),
            _ => Color::from_rgba(1.0, 1.0, 1.0, 0.05),
        })),
        border: Border { radius: 4.0.into(), ..Default::default() },
        text_color: if enabled { TEXT_PRIMARY } else { TEXT_SECONDARY },
        ..Default::default()
    };

    let label = text("Send to:")
        .size(12)
        .color(TEXT_SECONDARY);

    let mut app_buttons: Vec<Element<Message>> = vec![
        label.into(),
        Space::with_width(8).into(),
    ];

    if apps.is_empty() {
        app_buttons.push(
            text("No supported apps found. Add custom apps to config with send_to_apps=name:command")
                .size(11)
                .color(TEXT_SECONDARY)
                .into()
        );
    } else {
        for app in apps {
            let cmd = app.command_template.clone();
            let name = app.name.clone();
            let enabled = has_file;
            let btn = if enabled {
                button(text(name).size(12).color(TEXT_PRIMARY))
                    .on_press(Message::SendFileTo(cmd))
                    .style(move |_, status| btn_style(status, true))
                    .padding([3, 10])
            } else {
                button(text(name).size(12).color(TEXT_SECONDARY))
                    .style(move |_, status| btn_style(status, false))
                    .padding([3, 10])
            };
            app_buttons.push(btn.into());
            app_buttons.push(Space::with_width(4).into());
        }
    }

    let r = Row::from_vec(app_buttons)
        .align_y(Alignment::Center)
        .padding([8, 12]);

    container(r)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.06, 0.08, 0.10))),
            border: Border {
                width: 1.0,
                color: Color::from_rgba(0.2, 0.8, 1.0, 0.15),
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn build_settings_panel<'a>(
    volume: f32,
    full_width: bool,
    accent_color: Color,
    energy_colors: bool,
    loop_track: bool,
    auto_advance: bool,
    notifications_enabled: bool,
) -> Element<'a, Message> {
    let btn_style = |status: button::Status| button::Style {
        background: Some(Background::Color(match status {
            button::Status::Hovered | button::Status::Pressed =>
                Color::from_rgba(1.0, 1.0, 1.0, 0.15),
            _ => Color::from_rgba(1.0, 1.0, 1.0, 0.05),
        })),
        border: Border { radius: 4.0.into(), ..Default::default() },
        text_color: TEXT_PRIMARY,
        ..Default::default()
    };

    let vol_label = text(format!(
        "{} {:.0}%",
        icon_to_char(Bootstrap::VolumeUpFill),
        volume * 100.0
    ))
    .font(BOOTSTRAP_FONT)
    .size(12)
    .color(TEXT_SECONDARY);

    let vol_slider = slider(0.0..=1.0, volume, Message::VolumeChanged)
        .step(0.01)
        .width(100);

    let fw_btn = button(
        text(if full_width { "Full Width: On" } else { "Full Width: Off" })
            .size(12).color(if full_width { accent_color } else { TEXT_SECONDARY }),
    )
    .on_press(Message::ToggleFullWidth)
    .style(move |_, status| btn_style(status))
    .padding([3, 8]);

    let energy_btn = button(
        text(if energy_colors { "Energy: On" } else { "Energy: Off" })
            .size(12).color(if energy_colors { accent_color } else { TEXT_SECONDARY }),
    )
    .on_press(Message::ToggleEnergyColors)
    .style(move |_, status| btn_style(status))
    .padding([3, 8]);

    let loop_btn = button(
        text(if loop_track { "Loop: On" } else { "Loop: Off" })
            .size(12).color(if loop_track { accent_color } else { TEXT_SECONDARY }),
    )
    .on_press(Message::ToggleLoopTrack)
    .style(move |_, status| btn_style(status))
    .padding([3, 8]);

    let advance_btn = button(
        text(if auto_advance { "Advance: On" } else { "Advance: Off" })
            .size(12).color(if auto_advance { accent_color } else { TEXT_SECONDARY }),
    )
    .on_press(Message::ToggleAutoAdvance)
    .style(move |_, status| btn_style(status))
    .padding([3, 8]);

    let notify_btn = button(
        text(if notifications_enabled { "Notify: On" } else { "Notify: Off" })
            .size(12).color(if notifications_enabled { accent_color } else { TEXT_SECONDARY }),
    )
    .on_press(Message::ToggleNotifications)
    .style(move |_, status| btn_style(status))
    .padding([3, 8]);

    let accent_label = text("Accent:").size(12).color(TEXT_SECONDARY);

    let default_btn = button(text("Default").size(12).color(TEXT_PRIMARY))
        .on_press(Message::SetAccentDefault)
        .style(move |_, s| btn_style(s))
        .padding([3, 8]);

    let kde_btn = button(text("KDE").size(12).color(TEXT_PRIMARY))
        .on_press(Message::SetAccentKde)
        .style(move |_, s| btn_style(s))
        .padding([3, 8]);

    let cosmic_btn = button(text("COSMIC").size(12).color(TEXT_PRIMARY))
        .on_press(Message::SetAccentCosmic)
        .style(move |_, s| btn_style(s))
        .padding([3, 8]);

    let r = row![
        vol_label, Space::with_width(4), vol_slider, Space::with_width(8),
        fw_btn, Space::with_width(4), energy_btn, Space::with_width(4),
        loop_btn, Space::with_width(4), advance_btn, Space::with_width(4),
        notify_btn, Space::with_width(8), accent_label, Space::with_width(4),
        default_btn, Space::with_width(4), kde_btn, Space::with_width(4), cosmic_btn,
    ]
    .align_y(Alignment::Center)
    .padding([8, 12]);

    container(r)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.08, 0.09, 0.12))),
            border: Border {
                width: 1.0,
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

// ── Waveform canvas ──────────────────────────────────────────────────────────

struct WaveformCanvas {
    peaks: Vec<f32>,
    rms: Vec<f32>,
    low_energy: Vec<f32>,
    mid_energy: Vec<f32>,
    high_energy: Vec<f32>,
    progress: f32,
    cache: Cache,
    accent_color: Color,
    energy_colors: bool,
    low_color: Color,
    mid_color: Color,
    high_color: Color,
    loop_region: Option<(f32, f32)>,
    shift_held: bool,
}

#[derive(Debug, Clone, Default)]
struct WaveformState {
    left_dragging: bool,
    shift_dragging: bool,
}

impl canvas::Program<Message> for WaveformCanvas {
    type State = WaveformState;

    fn draw(
        &self,
        _state: &WaveformState,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            draw_waveform(
                frame, bounds.size(),
                &self.peaks, &self.rms,
                &self.low_energy, &self.mid_energy, &self.high_energy,
                self.progress, self.accent_color, self.energy_colors,
                self.low_color, self.mid_color, self.high_color,
                self.loop_region,
            );
        });
        vec![geometry]
    }

    fn update(
        &self,
        state: &mut WaveformState,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (iced::event::Status, Option<Message>) {
        let pos = cursor.position_in(bounds);
        let ratio = pos.map(|p| (p.x / bounds.width).clamp(0.0, 1.0));

        match event {
            canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(
                iced::mouse::Button::Left,
            )) => {
                let Some(r) = ratio else {
                    return (iced::event::Status::Ignored, None);
                };
                if self.shift_held {
                    state.shift_dragging = true;
                    state.left_dragging = false;
                    (iced::event::Status::Captured, Some(Message::SetLoopStart(r)))
                } else {
                    state.left_dragging = true;
                    state.shift_dragging = false;
                    (iced::event::Status::Captured, Some(Message::Seek(r)))
                }
            }
            canvas::Event::Mouse(iced::mouse::Event::ButtonReleased(
                iced::mouse::Button::Left,
            )) => {
                let was_shift = state.shift_dragging;
                let was_left = state.left_dragging;
                state.left_dragging = false;
                state.shift_dragging = false;
                if let Some(r) = ratio {
                    if was_shift {
                        return (iced::event::Status::Captured, Some(Message::SetLoopEnd(r)));
                    } else if was_left {
                        return (iced::event::Status::Captured, Some(Message::Seek(r)));
                    }
                }
                (iced::event::Status::Ignored, None)
            }
            canvas::Event::Mouse(iced::mouse::Event::CursorMoved { .. }) => {
                if let Some(r) = ratio {
                    if state.shift_dragging {
                        return (iced::event::Status::Captured, Some(Message::SetLoopEnd(r)));
                    } else if state.left_dragging {
                        return (iced::event::Status::Captured, Some(Message::Seek(r)));
                    }
                }
                (iced::event::Status::Ignored, None)
            }
            canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(
                iced::mouse::Button::Right,
            )) => {
                (iced::event::Status::Captured, Some(Message::OpenFile))
            }
            _ => (iced::event::Status::Ignored, None),
        }
    }

    fn mouse_interaction(
        &self,
        state: &WaveformState,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        if cursor.is_over(bounds) {
            if self.shift_held || state.shift_dragging {
                iced::mouse::Interaction::Crosshair
            } else {
                iced::mouse::Interaction::Pointer
            }
        } else {
            iced::mouse::Interaction::default()
        }
    }
}

fn energy_bar_color(
    low: f32, mid: f32, high: f32,
    low_color: Color, mid_color: Color, high_color: Color,
) -> Color {
    let total = low + mid + high + 0.001;
    let lw = low / total;
    let mw = mid / total;
    let hw = high / total;
    Color {
        r: (low_color.r * lw + mid_color.r * mw + high_color.r * hw).clamp(0.0, 1.0),
        g: (low_color.g * lw + mid_color.g * mw + high_color.g * hw).clamp(0.0, 1.0),
        b: (low_color.b * lw + mid_color.b * mw + high_color.b * hw).clamp(0.0, 1.0),
        a: 1.0,
    }
}

fn draw_waveform(
    frame: &mut Frame,
    size: Size,
    peaks: &[f32],
    rms: &[f32],
    low_energy: &[f32],
    mid_energy: &[f32],
    high_energy: &[f32],
    progress: f32,
    accent: Color,
    energy_colors: bool,
    low_color: Color,
    mid_color: Color,
    high_color: Color,
    loop_region: Option<(f32, f32)>,
) {
    let w = size.width;
    let h = size.height;
    let mid = h * 0.50;
    let refl_top = h * 0.54;
    let n = peaks.len();

    if n == 0 {
        let hint = canvas::Text {
            content: "Right-click to open  •  Left-click to seek  •  Shift+drag to set loop region".to_string(),
            position: Point::new(w / 2.0, h / 2.0),
            color: TEXT_SECONDARY,
            size: Pixels(13.0),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            vertical_alignment: iced::alignment::Vertical::Center,
            ..Default::default()
        };
        frame.fill_text(hint);
        return;
    }

    let bar_total_w = w / n as f32;
    let bar_w = bar_total_w;
    let played_x = w * progress;
    let max_amp = mid * 0.92;
    let refl_max = (h - refl_top) * 0.65;

    if energy_colors {
        frame.fill_rectangle(
            Point::new(played_x, 0.0),
            Size::new(w - played_x, h),
            Color::from_rgb(0.0, 0.0, 0.0),
        );
        if progress > 0.0 {
            frame.fill_rectangle(Point::new(0.0, 0.0), Size::new(played_x, h), PLAYED_BG);
        }
    }

    for (i, (&peak, &rms_val)) in peaks.iter().zip(rms.iter()).enumerate() {
        let x = i as f32 * bar_total_w;
        let bar_h = peak * max_amp;
        let refl_h = rms_val * refl_max;
        let is_played = x + bar_w <= played_x;
        let partial = x < played_x && x + bar_w > played_x;

        if energy_colors && !low_energy.is_empty() {
            let color = energy_bar_color(
                low_energy[i], mid_energy[i], high_energy[i],
                low_color, mid_color, high_color,
            );
            let refl_color = Color { a: 0.3, ..color };
            if partial {
                let split = played_x - x;
                frame.fill_rectangle(Point::new(x, mid - bar_h), Size::new(split, bar_h * 2.0), color);
                frame.fill_rectangle(Point::new(x + split, mid - bar_h), Size::new(bar_w - split, bar_h * 2.0), color);
                frame.fill_rectangle(Point::new(x, refl_top), Size::new(split, refl_h), refl_color);
                frame.fill_rectangle(Point::new(x + split, refl_top), Size::new(bar_w - split, refl_h), refl_color);
            } else {
                frame.fill_rectangle(Point::new(x, mid - bar_h), Size::new(bar_w, bar_h * 2.0), color);
                frame.fill_rectangle(Point::new(x, refl_top), Size::new(bar_w, refl_h), refl_color);
            }
        } else {
            let played_color = Color { a: 1.0, ..accent };
            let played_refl = Color { a: 0.25, ..accent };
            if partial {
                let split = played_x - x;
                frame.fill_rectangle(Point::new(x, mid - bar_h), Size::new(split, bar_h * 2.0), played_color);
                frame.fill_rectangle(Point::new(x + split, mid - bar_h), Size::new(bar_w - split, bar_h * 2.0), WAVEFORM_UNPLAYED);
                frame.fill_rectangle(Point::new(x, refl_top), Size::new(split, refl_h), played_refl);
                frame.fill_rectangle(Point::new(x + split, refl_top), Size::new(bar_w - split, refl_h), WAVEFORM_UNPLAYED_REFL);
            } else {
                let color = if is_played { played_color } else { WAVEFORM_UNPLAYED };
                let refl_color = if is_played { played_refl } else { WAVEFORM_UNPLAYED_REFL };
                frame.fill_rectangle(Point::new(x, mid - bar_h), Size::new(bar_w, bar_h * 2.0), color);
                frame.fill_rectangle(Point::new(x, refl_top), Size::new(bar_w, refl_h), refl_color);
            }
        }
    }

    if progress > 0.0 {
        frame.fill_rectangle(Point::new(played_x - 4.0, 0.0), Size::new(8.0, h), Color::from_rgba(1.0, 1.0, 1.0, 0.06));
        frame.fill_rectangle(Point::new(played_x - 1.0, 0.0), Size::new(2.0, h), SCRUBBER);
    }

    if let Some((a, b)) = loop_region {
        let rx = w * a;
        let rw = w * (b - a);
        frame.fill_rectangle(Point::new(rx, 0.0), Size::new(rw, h), REGION_FILL);
        frame.fill_rectangle(Point::new(rx, 0.0), Size::new(2.0, h), REGION_BORDER);
        frame.fill_rectangle(Point::new(rx + rw - 2.0, 0.0), Size::new(2.0, h), REGION_BORDER);
    }
}

fn build_waveform_canvas<'a>(
    waveform: Option<&WaveformData>,
    progress: f32,
    accent_color: Color,
    energy_colors: bool,
    low_color: Color,
    mid_color: Color,
    high_color: Color,
    loop_region: Option<(f32, f32)>,
    shift_held: bool,
) -> Element<'a, Message> {
    let peaks = waveform.map(|w| w.peaks.clone()).unwrap_or_default();
    let rms = waveform.map(|w| w.rms.clone()).unwrap_or_default();
    let low_energy = waveform.map(|w| w.low_energy.clone()).unwrap_or_default();
    let mid_energy = waveform.map(|w| w.mid_energy.clone()).unwrap_or_default();
    let high_energy = waveform.map(|w| w.high_energy.clone()).unwrap_or_default();

    let canvas_widget = canvas(WaveformCanvas {
        peaks, rms, low_energy, mid_energy, high_energy,
        progress,
        cache: Cache::new(),
        accent_color,
        energy_colors,
        low_color, mid_color, high_color,
        loop_region,
        shift_held,
    })
    .width(Length::Fill)
    .height(Length::Fill);

    container(canvas_widget)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(BG_DEEP)),
            ..Default::default()
        })
        .into()
}

fn format_time(secs: f32) -> String {
    let total = secs as u32;
    format!("{}:{:02}", total / 60, total % 60)
}

fn format_time_ms(secs: f64) -> String {
    let total = secs as u32;
    let ms = ((secs - total as f64) * 1000.0) as u32;
    format!("{}:{:02}.{:03}", total / 60, total % 60, ms)
}

// Need to import Row explicitly for the dynamic vec case
use iced::widget::Row;
