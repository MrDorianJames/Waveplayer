use std::path::PathBuf;
use iced::Color;

#[derive(Debug, Clone)]
pub struct Config {
    pub volume: f32,
    pub accent_color: Color,
    pub full_width: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            volume: 0.8,
            accent_color: Color::from_rgb(1.0, 0.42, 0.0),
            full_width: false,
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
                if let Ok(v) = val.parse::<f32>() {
                    config.volume = v.clamp(0.0, 1.0);
                }
            } else if let Some(val) = line.strip_prefix("full_width=") {
                config.full_width = val == "true";
            } else if let Some(val) = line.strip_prefix("accent_r=") {
                if let Ok(r) = val.parse::<f32>() {
                    config.accent_color.r = r;
                }
            } else if let Some(val) = line.strip_prefix("accent_g=") {
                if let Ok(g) = val.parse::<f32>() {
                    config.accent_color.g = g;
                }
            } else if let Some(val) = line.strip_prefix("accent_b=") {
                if let Ok(b) = val.parse::<f32>() {
                    config.accent_color.b = b;
                }
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
            "volume={}\nfull_width={}\naccent_r={}\naccent_g={}\naccent_b={}\n",
            self.volume,
            self.full_width,
            self.accent_color.r,
            self.accent_color.g,
            self.accent_color.b,
        );
        let _ = std::fs::write(&path, contents);
    }
}
