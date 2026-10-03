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
    pub loop_track: bool,
    pub auto_advance: bool,
    pub seek_secs: f64,
    pub notifications_enabled: bool,
    pub notification_min_duration: f64,
    pub window_height: f32,
    /// Extra app entries, each "Name:command %f" or "Name:command"
    pub send_to_apps: Vec<String>,
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
            loop_track: false,
            auto_advance: false,
            seek_secs: 5.0,
            notifications_enabled: true,
            notification_min_duration: 60.0,
            window_height: 110.0,
            send_to_apps: vec![],
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
            } else if let Some(val) = line.strip_prefix("loop_track=") {
                config.loop_track = val == "true";
            } else if let Some(val) = line.strip_prefix("auto_advance=") {
                config.auto_advance = val == "true";
            } else if let Some(val) = line.strip_prefix("seek_secs=") {
                if let Ok(v) = val.parse::<f64>() { config.seek_secs = v.clamp(1.0, 60.0); }
            } else if let Some(val) = line.strip_prefix("notifications_enabled=") {
                config.notifications_enabled = val == "true";
            } else if let Some(val) = line.strip_prefix("notification_min_duration=") {
                if let Ok(v) = val.parse::<f64>() { config.notification_min_duration = v.max(0.0); }
            } else if let Some(val) = line.strip_prefix("window_height=") {
                if let Ok(v) = val.parse::<f32>() { config.window_height = v.clamp(80.0, 600.0); }
            } else if let Some(val) = line.strip_prefix("send_to_apps=") {
                config.send_to_apps = val.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
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
        let send_to = self.send_to_apps.join(",");
        let contents = format!(
            "volume={}\nfull_width={}\nenergy_colors={}\n\
             loop_track={}\nauto_advance={}\nseek_secs={}\n\
             notifications_enabled={}\nnotification_min_duration={}\n\
             window_height={}\nsend_to_apps={}\n\
             accent_r={}\naccent_g={}\naccent_b={}\n\
             low_r={}\nlow_g={}\nlow_b={}\n\
             mid_r={}\nmid_g={}\nmid_b={}\n\
             high_r={}\nhigh_g={}\nhigh_b={}\n",
            self.volume,
            self.full_width,
            self.energy_colors,
            self.loop_track,
            self.auto_advance,
            self.seek_secs,
            self.notifications_enabled,
            self.notification_min_duration,
            self.window_height,
            send_to,
            self.accent_color.r, self.accent_color.g, self.accent_color.b,
            self.low_color.r, self.low_color.g, self.low_color.b,
            self.mid_color.r, self.mid_color.g, self.mid_color.b,
            self.high_color.r, self.high_color.g, self.high_color.b,
        );
        let _ = std::fs::write(&path, contents);
    }
}

// ── Send-to app detection ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SendToApp {
    pub name: String,
    /// The executable name, used for `which` detection
    pub executable: String,
    /// Full command template. %f is replaced with the file path.
    /// If no %f present, file is appended as last argument.
    pub command_template: String,
}

/// Known DAWs/editors: (display name, executable, command template)
const KNOWN_APPS: &[(&str, &str, &str)] = &[
    ("Audacity",      "audacity",      "audacity %f"),
    ("Tenacity",      "tenacity",      "tenacity %f"),
    ("Ardour",        "ardour",        "ardour %f"),
    ("REAPER",        "reaper",        "reaper %f"),
    ("Bitwig Studio", "bitwig-studio", "bitwig-studio %f"),
    ("LMMS",          "lmms",          "lmms %f"),
    ("Mixxx",         "mixxx",         "mixxx %f"),
    ("Qtractor",      "qtractor",      "qtractor %f"),
    ("Rosegarden",    "rosegarden",    "rosegarden %f"),
    ("Kdenlive",      "kdenlive",      "kdenlive %f"),
    ("ocenaudio",     "ocenaudio",     "ocenaudio %f"),
];

fn command_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn available_send_to_apps(custom: &[String]) -> Vec<SendToApp> {
    // Start with detected known apps
    let mut apps: Vec<SendToApp> = KNOWN_APPS.iter()
        .filter(|(_, exe, _)| command_exists(exe))
        .map(|(name, exe, cmd)| SendToApp {
            name: name.to_string(),
            executable: exe.to_string(),
            command_template: cmd.to_string(),
        })
        .collect();

    // Add custom entries from config
    // Format: "Name:command %f" or "Name:command" or just "command %f"
    for entry in custom {
        let entry = entry.trim();
        if entry.is_empty() { continue; }

        let (name, cmd_template) = if let Some(pos) = entry.find(':') {
            (entry[..pos].trim().to_string(), entry[pos+1..].trim().to_string())
        } else {
            // No colon — treat the whole thing as the command, derive name from first word
            let name = entry.split_whitespace().next().unwrap_or(entry).to_string();
            (name, entry.to_string())
        };

        // Extract executable (first word of command template)
        let executable = cmd_template.split_whitespace()
            .next()
            .unwrap_or(&cmd_template)
            .to_string();

        // Only add if executable exists and not already in list
        if apps.iter().any(|a| a.executable == executable) { continue; }
        if !command_exists(&executable) { continue; }

        apps.push(SendToApp { name, executable, command_template: cmd_template });
    }

    apps
}

/// Launch an app with the file path substituted into the command template.
/// %f in the template is replaced with the quoted file path.
/// If no %f present, the file path is appended as the last argument.
pub fn send_file_to_app(command_template: &str, file_path: &std::path::Path) {
    let file_str = file_path.to_string_lossy();

    // Split template into argv — simple whitespace split, no shell quoting
    let parts: Vec<String> = if command_template.contains("%f") {
        // Replace %f token with the file path
        command_template
            .split_whitespace()
            .map(|part| {
                if part == "%f" {
                    file_str.to_string()
                } else {
                    part.replace("%f", &*file_str)
                }
            })
            .collect()
    } else {
        // No %f — append file at end
        let mut parts: Vec<String> = command_template.split_whitespace()
            .map(|s| s.to_string())
            .collect();
        parts.push(file_str.to_string());
        parts
    };

    if parts.is_empty() { return; }

    let _ = std::process::Command::new(&parts[0])
        .args(&parts[1..])
        .spawn();
}
