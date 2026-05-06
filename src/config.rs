use std::path::PathBuf;
use iced::Color;

#[derive(Debug, Clone)]
pub struct Config {
    pub volume: f32,
    pub accent_color: Color,
    pub full_width: bool,
    pub energy_colors: bool,
    pub low_color: Color,
    pub mid_color: Color,
    pub high_color: Color,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            volume: 0.8,
            accent_color: Color::from_rgb(1.0, 0.42, 0.0),
            full_width: false,
            energy_colors: true,
            low_color: Color::from_rgb(1.0, 0.2, 0.0),
            mid_color: Color::from_rgb(0.2, 0.8, 0.2),
            high_color: Color::from_rgb(0.2, 0.4, 1.0),
        }
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
    .unwrap_or_else(|| PathBuf::from("."))
    .join("waveplayer")
    .join("config.toml")
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        let mut config = Self::default();
        for line in contents.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("volume=") {
                if let Ok(v) = val.parse::<f32>() { config.volume = v.clamp(0.0, 1.0); }
            } else if let Some(val) = line.strip_prefix("full_width=") {
                config.full_width = val == "true";
            } else if let Some(val) = line.strip_prefix("energy_colors=") {
                config.energy_colors = val == "true";
            } else if let Some(val) = line.strip_prefix("accent_r=") {
                if let Ok(v) = val.parse::<f32>() { config.accent_color.r = v; }
            } else if let Some(val) = line.strip_prefix("accent_g=") {
                if let Ok(v) = val.parse::<f32>() { config.accent_color.g = v; }
            } else if let Some(val) = line.strip_prefix("accent_b=") {
                if let Ok(v) = val.parse::<f32>() { config.accent_color.b = v; }
            } else if let Some(val) = line.strip_prefix("low_r=") {
                if let Ok(v) = val.parse::<f32>() { config.low_color.r = v; }
            } else if let Some(val) = line.strip_prefix("low_g=") {
                if let Ok(v) = val.parse::<f32>() { config.low_color.g = v; }
            } else if let Some(val) = line.strip_prefix("low_b=") {
                if let Ok(v) = val.parse::<f32>() { config.low_color.b = v; }
            } else if let Some(val) = line.strip_prefix("mid_r=") {
                if let Ok(v) = val.parse::<f32>() { config.mid_color.r = v; }
            } else if let Some(val) = line.strip_prefix("mid_g=") {
                if let Ok(v) = val.parse::<f32>() { config.mid_color.g = v; }
            } else if let Some(val) = line.strip_prefix("mid_b=") {
                if let Ok(v) = val.parse::<f32>() { config.mid_color.b = v; }
            } else if let Some(val) = line.strip_prefix("high_r=") {
                if let Ok(v) = val.parse::<f32>() { config.high_color.r = v; }
            } else if let Some(val) = line.strip_prefix("high_g=") {
                if let Ok(v) = val.parse::<f32>() { config.high_color.g = v; }
            } else if let Some(val) = line.strip_prefix("high_b=") {
                if let Ok(v) = val.parse::<f32>() { config.high_color.b = v; }
            }
        }
        config
    }

    pub fn save(&self) {
        let path = config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let contents = format!(
            "volume={}\nfull_width={}\nenergy_colors={}\n\
accent_r={}\naccent_g={}\naccent_b={}\n\
low_r={}\nlow_g={}\nlow_b={}\n\
mid_r={}\nmid_g={}\nmid_b={}\n\
high_r={}\nhigh_g={}\nhigh_b={}\n",
self.volume,
self.full_width,
self.energy_colors,
self.accent_color.r, self.accent_color.g, self.accent_color.b,
self.low_color.r, self.low_color.g, self.low_color.b,
self.mid_color.r, self.mid_color.g, self.mid_color.b,
self.high_color.r, self.high_color.g, self.high_color.b,
        );
        let _ = std::fs::write(&path, contents);
    }
}
