use iced::{
    widget::{
        button, canvas, canvas::Cache, canvas::Frame, canvas::Geometry,
        column, container, row, slider, text, Space,
    },
    Alignment, Background, Border, Color, Element, Font, Length, Pixels, Point, Rectangle, Size,
    Theme,
};
use iced_fonts::{bootstrap::icon_to_char, Bootstrap, BOOTSTRAP_FONT};

use crate::{Message, WaveformData};

const BG_DEEP: Color = Color::from_rgb(0.06, 0.07, 0.09);
const BG_CARD: Color = Color::from_rgb(0.10, 0.11, 0.14);
const WAVEFORM_UNPLAYED: Color = Color::from_rgb(0.30, 0.32, 0.38);
const WAVEFORM_UNPLAYED_REFL: Color = Color::from_rgba(0.30, 0.32, 0.38, 0.20);
const TEXT_PRIMARY: Color = Color::from_rgb(0.95, 0.95, 0.97);
const TEXT_SECONDARY: Color = Color::from_rgb(0.50, 0.53, 0.60);
const SCRUBBER: Color = Color::from_rgb(1.0, 1.0, 1.0);

pub fn build_ui<'a>(
    waveform: Option<&WaveformData>,
    file_name: Option<&'a str>,
    is_playing: bool,
    progress: f32,
    position_secs: f64,
    duration_secs: f32,
    volume: f32,
    show_settings: bool,
    full_width: bool,
    accent_color: Color,
) -> Element<'a, Message> {
    let header = build_header(file_name, position_secs, duration_secs, is_playing, show_settings, accent_color);
    let wave = build_waveform_canvas(waveform, progress, accent_color);

    let mut col = column![header, wave].spacing(0).width(Length::Fill);

    if show_settings {
        col = col.push(build_settings_panel(volume, full_width, accent_color));
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
    accent_color: Color,
) -> Element<'a, Message> {
    let title = text(file_name.unwrap_or("Right-click waveform to open a file"))
    .size(13)
    .color(TEXT_PRIMARY)
    .font(Font {
        weight: iced::font::Weight::Bold,
        ..Font::DEFAULT
    });

    let time_label = text(format!(
        "{} / {}",
        format_time(pos as f32),
            format_time(dur)
    ))
    .size(11)
    .color(TEXT_SECONDARY);

    let gear_btn = button(
        text(icon_to_char(Bootstrap::GearFill).to_string())
        .font(BOOTSTRAP_FONT)
        .size(14)
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
        title,
        Space::with_width(8),
        time_label,
        Space::with_width(Length::Fill),
        gear_btn,
    ]
    .align_y(Alignment::Center)
    .spacing(0)
    .padding([6u16, 12]);

    container(r)
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(Background::Color(BG_CARD)),
           ..Default::default()
    })
    .into()
}

fn build_settings_panel<'a>(volume: f32, full_width: bool, accent_color: Color) -> Element<'a, Message> {
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
    .width(120);

    let fw_btn = button(
        text(if full_width { "Full Width: On" } else { "Full Width: Off" })
        .size(12)
        .color(if full_width { accent_color } else { TEXT_SECONDARY }),
    )
    .on_press(Message::ToggleFullWidth)
    .style(move |_, status| btn_style(status))
    .padding([3, 10]);

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
        vol_label,
        Space::with_width(6),
        vol_slider,
        Space::with_width(16),
        fw_btn,
        Space::with_width(16),
        accent_label,
        Space::with_width(6),
        default_btn,
        Space::with_width(4),
        kde_btn,
        Space::with_width(4),
        cosmic_btn,
    ]
    .align_y(Alignment::Center)
    .padding([8, 16]);

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

struct WaveformCanvas {
    peaks: Vec<f32>,
    rms: Vec<f32>,
    progress: f32,
    cache: Cache,
    accent_color: Color,
}

#[derive(Debug, Clone, Default)]
struct WaveformState {
    dragging: bool,
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
        let accent = self.accent_color;
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            draw_waveform(frame, bounds.size(), &self.peaks, &self.rms, self.progress, accent);
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
        let pos = match cursor.position_in(bounds) {
            Some(p) => p,
            None => {
                if let canvas::Event::Mouse(iced::mouse::Event::ButtonReleased(
                    iced::mouse::Button::Left,
                )) = event {
                    state.dragging = false;
                }
                return (iced::event::Status::Ignored, None);
            }
        };

        let ratio = (pos.x / bounds.width).clamp(0.0, 1.0);

        match event {
            canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(
                iced::mouse::Button::Left,
            )) => {
                state.dragging = true;
                (iced::event::Status::Captured, Some(Message::Seek(ratio)))
            }
            canvas::Event::Mouse(iced::mouse::Event::ButtonReleased(
                iced::mouse::Button::Left,
            )) => {
                state.dragging = false;
                (iced::event::Status::Captured, Some(Message::Seek(ratio)))
            }
            canvas::Event::Mouse(iced::mouse::Event::CursorMoved { .. }) => {
                if state.dragging {
                    (iced::event::Status::Captured, Some(Message::Seek(ratio)))
                } else {
                    (iced::event::Status::Ignored, None)
                }
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
        _state: &WaveformState,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        if cursor.is_over(bounds) {
            iced::mouse::Interaction::Pointer
        } else {
            iced::mouse::Interaction::default()
        }
    }
}

fn draw_waveform(
    frame: &mut Frame,
    size: Size,
    peaks: &[f32],
    rms: &[f32],
    progress: f32,
    accent: Color,
) {
    let w = size.width;
    let h = size.height;
    let mid = h * 0.50;
    let refl_top = h * 0.54;
    let n = peaks.len();

    let played = Color { a: 1.0, ..accent };
    let played_refl = Color { a: 0.25, ..accent };

    if n == 0 {
        let hint = canvas::Text {
            content: "Right-click to open an audio file".to_string(),
            position: Point::new(w / 2.0, h / 2.0),
            color: TEXT_SECONDARY,
            size: Pixels(14.0),
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

    for (i, (&peak, &rms_val)) in peaks.iter().zip(rms.iter()).enumerate() {
        let x = i as f32 * bar_total_w;
        let bar_h = peak * max_amp;
        let refl_h = rms_val * refl_max;
        let is_played = x + bar_w <= played_x;
        let partial = x < played_x && x + bar_w > played_x;

        if partial {
            let split = played_x - x;
            frame.fill_rectangle(Point::new(x, mid - bar_h), Size::new(split, bar_h * 2.0), played);
            frame.fill_rectangle(Point::new(x + split, mid - bar_h), Size::new(bar_w - split, bar_h * 2.0), WAVEFORM_UNPLAYED);
            frame.fill_rectangle(Point::new(x, refl_top), Size::new(split, refl_h), played_refl);
            frame.fill_rectangle(Point::new(x + split, refl_top), Size::new(bar_w - split, refl_h), WAVEFORM_UNPLAYED_REFL);
        } else {
            let color = if is_played { played } else { WAVEFORM_UNPLAYED };
            let refl_color = if is_played { played_refl } else { WAVEFORM_UNPLAYED_REFL };
            frame.fill_rectangle(Point::new(x, mid - bar_h), Size::new(bar_w, bar_h * 2.0), color);
            frame.fill_rectangle(Point::new(x, refl_top), Size::new(bar_w, refl_h), refl_color);
        }
    }

    if progress > 0.0 {
        frame.fill_rectangle(Point::new(played_x - 4.0, 0.0), Size::new(8.0, h), Color::from_rgba(1.0, 1.0, 1.0, 0.06));
        frame.fill_rectangle(Point::new(played_x - 1.0, 0.0), Size::new(2.0, h), SCRUBBER);
    }
}

fn build_waveform_canvas<'a>(
    waveform: Option<&WaveformData>,
    progress: f32,
    accent_color: Color,
) -> Element<'a, Message> {
    let peaks = waveform.map(|w| w.peaks.clone()).unwrap_or_default();
    let rms = waveform.map(|w| w.rms.clone()).unwrap_or_default();
    let canvas_widget = canvas(WaveformCanvas {
        peaks,
        rms,
        progress,
        cache: Cache::new(),
                               accent_color,
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
