// config.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::io::{self, Write};
use crate::ansi_theme::AnsiTheme;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub workspace_path: String,
    pub log_file: String,
    pub verbose_logging: bool,
    pub theme: ThemeConfig,  // Added theme
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            workspace_path: "D:\\RustProjects".to_string(),
            log_file: "build_tool.log".to_string(),
            verbose_logging: true,
            theme: ThemeConfig::default(),  // Added default theme
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeConfig {
    pub background_color: TerminalColor,
    pub foreground_color: TerminalColor,
    pub logo_color: TerminalColor,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background_color: TerminalColor::Default,
            foreground_color: TerminalColor::Default,
            logo_color: TerminalColor::Red,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TerminalColor {
    Default,
    Black,
    White,
    Red,
    Green,
    Blue,
    Yellow,
    Magenta,
    Cyan,
    // Extended color palette
    Rgb(u8, u8, u8), // True color support
}

impl TerminalColor {
    pub fn apply_background(&self) {
        match self {
            TerminalColor::Default => print!("\x1b[49m"),
            TerminalColor::Black => print!("\x1b[40m"),
            TerminalColor::White => print!("\x1b[47m"), 
            TerminalColor::Red => print!("\x1b[41m"),
            TerminalColor::Green => print!("\x1b[42m"),
            TerminalColor::Blue => print!("\x1b[44m"),
            TerminalColor::Yellow => print!("\x1b[43m"),
            TerminalColor::Magenta => print!("\x1b[45m"),
            TerminalColor::Cyan => print!("\x1b[46m"),
            TerminalColor::Rgb(r, g, b) => print!("\x1b[48;2;{};{};{}m", r, g, b),
        }
        io::stdout().flush().unwrap();
    }
    
    pub fn apply_foreground(&self) {
        match self {
            TerminalColor::Default => print!("\x1b[39m"),
            TerminalColor::Black => print!("\x1b[30m"),
            TerminalColor::White => print!("\x1b[37m"),
            TerminalColor::Red => print!("\x1b[31m"), 
            TerminalColor::Green => print!("\x1b[32m"),
            TerminalColor::Blue => print!("\x1b[34m"),
            TerminalColor::Yellow => print!("\x1b[33m"),
            TerminalColor::Magenta => print!("\x1b[35m"),
            TerminalColor::Cyan => print!("\x1b[36m"),
            TerminalColor::Rgb(r, g, b) => print!("\x1b[38;2;{};{};{}m", r, g, b),
        }
        io::stdout().flush().unwrap();
    }

    pub fn to_ansi_bg_code(&self) -> &'static str {
        match self {
            TerminalColor::Default => "\x1b[49m",
            TerminalColor::Black => "\x1b[40m",
            TerminalColor::White => "\x1b[47m", 
            TerminalColor::Red => "\x1b[41m",
            TerminalColor::Green => "\x1b[42m",
            TerminalColor::Blue => "\x1b[44m",
            TerminalColor::Yellow => "\x1b[43m",
            TerminalColor::Magenta => "\x1b[45m",
            TerminalColor::Cyan => "\x1b[46m",
            TerminalColor::Rgb(_, _, _) => "\x1b[49m", // Simplified for static return
        }
    }

    pub fn to_ansi_fg_code(&self) -> &'static str {
        match self {
            TerminalColor::Default => "\x1b[39m",
            TerminalColor::Black => "\x1b[30m",
            TerminalColor::White => "\x1b[37m",
            TerminalColor::Red => "\x1b[31m", 
            TerminalColor::Green => "\x1b[32m",
            TerminalColor::Blue => "\x1b[34m",
            TerminalColor::Yellow => "\x1b[33m",
            TerminalColor::Magenta => "\x1b[35m",
            TerminalColor::Cyan => "\x1b[36m",
            TerminalColor::Rgb(_, _, _) => "\x1b[39m", // Simplified for static return
        }
    }
}

// Simple function to apply theme
pub fn apply_theme(theme: &ThemeConfig) {
    theme.background_color.apply_background();
    theme.foreground_color.apply_foreground();
}

// Function to reset terminal colors
pub fn reset_terminal_colors() {
    print!("\x1b[0m");
    io::stdout().flush().unwrap();
}

pub struct ConfigManager {
    config_path: PathBuf,
    config: AppConfig,
}

impl ConfigManager {
    pub fn new() -> Self {
        let config_path = Self::get_config_path();
        let config = Self::load_config(&config_path);
        
        Self {
            config_path,
            config,
        }
    }

    pub fn get_config_path() -> PathBuf {
        if cfg!(target_os = "windows") {
            if let Ok(app_data) = std::env::var("APPDATA") {
                let mut path = PathBuf::from(app_data);
                path.push("rust_dev_terminal");
                path.push("config.toml");
                return path;
            }
        }
        
        // Fallback for other OS or if APPDATA is not set
        let mut path = std::env::current_dir().unwrap_or_default();
        path.push("rust_dev_terminal_config.toml");
        path
    }

    fn load_config(config_path: &PathBuf) -> AppConfig {
        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if config_path.exists() {
            match fs::read_to_string(config_path) {
                Ok(content) => {
                    // FIX: Specify the type explicitly for toml::from_str
                    match toml::from_str::<AppConfig>(&content) {
                        Ok(config) => {
                            AnsiTheme::print_themed(&format!("Loaded config from: {}\n", config_path.display()), &config.theme);
                            return config;
                        }
                        Err(e) => {
                            AnsiTheme::print_error(&format!("Invalid config file: {}. Using defaults.\n", e), &ThemeConfig::default());
                        }
                    }
                }
                Err(e) => {
                    AnsiTheme::print_error(&format!("Failed to read config file: {}. Using defaults.\n", e), &ThemeConfig::default());
                }
            }
        }

        // Create default config file
        let default_config = AppConfig::default();
        if let Err(e) = default_config.save_to_file(config_path) {
            AnsiTheme::print_error(&format!("Failed to create config file: {}\n", e), &ThemeConfig::default());
        } else {
            AnsiTheme::print_themed(&format!("Created default config at: {}\n", config_path.display()), &ThemeConfig::default());
        }

        default_config
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    pub fn update_config<F>(&mut self, updater: F) -> Result<(), String>
    where
        F: FnOnce(&mut AppConfig),
    {
        updater(&mut self.config);
        self.config.save_to_file(&self.config_path)
    }

    pub fn show_config(&self, theme: &ThemeConfig) {
        AnsiTheme::print_themed("\nCurrent Configuration:\n", theme);
        AnsiTheme::print_themed(&format!("Workspace Path: {}\n", self.config.workspace_path), theme);
        AnsiTheme::print_themed(&format!("Log File: {}\n", self.config.log_file), theme);
        AnsiTheme::print_themed(&format!("Verbose Logging: {}\n", 
            if self.config.verbose_logging { "Enabled" } else { "Disabled" }), theme);
        AnsiTheme::print_themed(&format!("Background Color: {:?}\n", self.config.theme.background_color), theme);
        AnsiTheme::print_themed(&format!("Foreground Color: {:?}\n", self.config.theme.foreground_color), theme);
        AnsiTheme::print_themed(&format!("Logo Color: {:?}\n", self.config.theme.logo_color), theme);
        AnsiTheme::print_themed(&format!("Config Location: {}\n", self.config_path.display()), theme);
    }

    pub fn edit_config_interactive(&mut self, theme: &ThemeConfig) -> Result<(), String> {
        use std::io::{self, Write};

        AnsiTheme::print_themed("\nInteractive Config Editor\n", theme);
        AnsiTheme::print_themed(&format!("Current workspace path: {}\n", self.config.workspace_path), theme);
        AnsiTheme::print_themed("Enter new workspace path (or press Enter to keep current): ", theme);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let new_path = input.trim();

        if !new_path.is_empty() {
            // Validate the path
            let path = PathBuf::from(new_path);
            if !path.exists() {
                AnsiTheme::print_themed("Path does not exist. Create it? (y/N): ", theme);
                io::stdout().flush().unwrap();
                
                let mut confirm = String::new();
                io::stdin().read_line(&mut confirm).unwrap();
                
                if confirm.trim().eq_ignore_ascii_case("y") {
                    fs::create_dir_all(&path)
                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                    AnsiTheme::print_themed(&format!("Created directory: {}\n", path.display()), theme);
                } else {
                    return Err("Configuration cancelled.".to_string());
                }
            }
            
            self.update_config(|config| {
                config.workspace_path = new_path.to_string();
            })?;
            AnsiTheme::print_themed("Workspace path updated\n", theme);
        }

        Ok(())
    }
}

impl AppConfig {
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), String> {
        let toml_content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        
        fs::write(path, toml_content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        
        Ok(())
    }

    pub fn get_workspace_path(&self) -> PathBuf {
        PathBuf::from(&self.workspace_path)
    }

    pub fn get_project_path(&self, project_name: &str) -> PathBuf {
        self.get_workspace_path().join(project_name)
    }

    pub fn get_src_path(&self, project_name: &str) -> PathBuf {
        self.get_project_path(project_name).join("src")
    }

    pub fn get_cargo_toml_path(&self, project_name: &str) -> PathBuf {
        self.get_project_path(project_name).join("Cargo.toml")
    }
}