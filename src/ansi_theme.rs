// ansi_theme.rs
use std::io::{self, Write};
use crate::config::{ThemeConfig, TerminalColor};

pub struct AnsiTheme;

impl AnsiTheme {
    // Enable ANSI support on Windows
    pub fn enable_ansi_support() {
        #[cfg(windows)]
        {
            unsafe {
                use winapi::um::consoleapi::SetConsoleMode;
                use winapi::um::processenv::GetStdHandle;
                use winapi::um::winbase::STD_OUTPUT_HANDLE;
                use winapi::um::wincon::{ENABLE_VIRTUAL_TERMINAL_PROCESSING, ENABLE_PROCESSED_OUTPUT};
                
                let handle = GetStdHandle(STD_OUTPUT_HANDLE);
                let mut mode: u32 = 0;
                
                if winapi::um::consoleapi::GetConsoleMode(handle, &mut mode) != 0 {
                    mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING | ENABLE_PROCESSED_OUTPUT;
                    SetConsoleMode(handle, mode);
                }
            }
        }
    }

    // Apply the full theme (background + foreground) persistently
    pub fn apply_theme(theme: &ThemeConfig) {
        // Apply background and foreground colors without reset
        print!("{}{}",
            theme.background_color.to_ansi_bg_code(),
            theme.foreground_color.to_ansi_fg_code()
        );
        io::stdout().flush().unwrap();
    }

    // Reset to terminal defaults
    pub fn reset() {
        print!("\x1b[0m");
        io::stdout().flush().unwrap();
    }

    // Theme-aware text styling - NO RESET, maintains theme background
    pub fn styled_text(text: &str, ansi_code: &str, theme: &ThemeConfig) -> String {
        format!("{}{}{}",
            theme.background_color.to_ansi_bg_code(), // Maintain theme background
            ansi_code, // Apply specific foreground style
            text
        )
    }

    // Print styled text directly (maintains theme background)
    pub fn print_styled(text: &str, ansi_code: &str, theme: &ThemeConfig) {
        print!("{}{}{}",
            theme.background_color.to_ansi_bg_code(),
            ansi_code,
            text
        );
        io::stdout().flush().unwrap();
    }

    pub fn println_styled(text: &str, ansi_code: &str, theme: &ThemeConfig) {
        Self::print_styled(&format!("{}\n", text), ansi_code, theme);
    }

    // Logo with configurable color
    pub fn themed_logo(text: &str, theme: &ThemeConfig) -> String {
        let logo_code = match &theme.logo_color {
            TerminalColor::Red => "\x1b[31m",
            TerminalColor::Green => "\x1b[32m",
            TerminalColor::Blue => "\x1b[34m",
            TerminalColor::Yellow => "\x1b[33m",
            TerminalColor::Magenta => "\x1b[35m",
            TerminalColor::Cyan => "\x1b[36m",
            TerminalColor::White => "\x1b[37m",
            TerminalColor::Black => "\x1b[30m",
            TerminalColor::Default => "\x1b[39m",
            TerminalColor::Rgb(r, g, b) => return Self::rgb_text(text, *r, *g, *b, theme),
        };
        
        format!("{}{}{}",
            theme.background_color.to_ansi_bg_code(),
            logo_code,
            text
        )
    }

    pub fn print_logo(text: &str, theme: &ThemeConfig) {
        let logo_code = match &theme.logo_color {
            TerminalColor::Red => "\x1b[31m",
            TerminalColor::Green => "\x1b[32m",
            TerminalColor::Blue => "\x1b[34m",
            TerminalColor::Yellow => "\x1b[33m",
            TerminalColor::Magenta => "\x1b[35m",
            TerminalColor::Cyan => "\x1b[36m",
            TerminalColor::White => "\x1b[37m",
            TerminalColor::Black => "\x1b[30m",
            TerminalColor::Default => "\x1b[39m",
            TerminalColor::Rgb(r, g, b) => {
                Self::print_rgb(text, *r, *g, *b, theme);
                return;
            },
        };
        
        print!("{}{}{}",
            theme.background_color.to_ansi_bg_code(),
            logo_code,
            text
        );
        io::stdout().flush().unwrap();
    }

    // RGB text support
    pub fn rgb_text(text: &str, r: u8, g: u8, b: u8, theme: &ThemeConfig) -> String {
        format!("{}\x1b[38;2;{};{};{}m{}",
            theme.background_color.to_ansi_bg_code(),
            r, g, b,
            text
        )
    }

    pub fn print_rgb(text: &str, r: u8, g: u8, b: u8, theme: &ThemeConfig) {
        print!("{}\x1b[38;2;{};{};{}m{}",
            theme.background_color.to_ansi_bg_code(),
            r, g, b,
            text
        );
        io::stdout().flush().unwrap();
    }

    // Pre-defined colored text methods (maintain theme background)
    pub fn red(text: &str, theme: &ThemeConfig) -> String {
        Self::styled_text(text, "\x1b[31m", theme)
    }

    pub fn green(text: &str, theme: &ThemeConfig) -> String {
        Self::styled_text(text, "\x1b[32m", theme)
    }

    pub fn blue(text: &str, theme: &ThemeConfig) -> String {
        Self::styled_text(text, "\x1b[34m", theme)
    }

    pub fn cyan(text: &str, theme: &ThemeConfig) -> String {
        Self::styled_text(text, "\x1b[36m", theme)
    }

    pub fn yellow(text: &str, theme: &ThemeConfig) -> String {
        Self::styled_text(text, "\x1b[33m", theme)
    }

    pub fn magenta(text: &str, theme: &ThemeConfig) -> String {
        Self::styled_text(text, "\x1b[35m", theme)
    }

    pub fn white(text: &str, theme: &ThemeConfig) -> String {
        Self::styled_text(text, "\x1b[37m", theme)
    }

    pub fn black(text: &str, theme: &ThemeConfig) -> String {
        Self::styled_text(text, "\x1b[30m", theme)
    }

    // Direct print versions
    pub fn print_red(text: &str, theme: &ThemeConfig) {
        Self::print_styled(text, "\x1b[31m", theme);
    }

    pub fn print_green(text: &str, theme: &ThemeConfig) {
        Self::print_styled(text, "\x1b[32m", theme);
    }

    pub fn print_blue(text: &str, theme: &ThemeConfig) {
        Self::print_styled(text, "\x1b[34m", theme);
    }

    pub fn print_cyan(text: &str, theme: &ThemeConfig) {
        Self::print_styled(text, "\x1b[36m", theme);
    }

    pub fn print_yellow(text: &str, theme: &ThemeConfig) {
        Self::print_styled(text, "\x1b[33m", theme);
    }

    pub fn print_magenta(text: &str, theme: &ThemeConfig) {
        Self::print_styled(text, "\x1b[35m", theme);
    }

    pub fn print_white(text: &str, theme: &ThemeConfig) {
        Self::print_styled(text, "\x1b[37m", theme);
    }

    pub fn print_black(text: &str, theme: &ThemeConfig) {
        Self::print_styled(text, "\x1b[30m", theme);
    }

    // Bright colors
    pub fn bright_red(text: &str, theme: &ThemeConfig) -> String {
        Self::styled_text(text, "\x1b[91m", theme)
    }

    pub fn bright_green(text: &str, theme: &ThemeConfig) -> String {
        Self::styled_text(text, "\x1b[92m", theme)
    }

    pub fn bright_blue(text: &str, theme: &ThemeConfig) -> String {
        Self::styled_text(text, "\x1b[94m", theme)
    }

    pub fn bright_cyan(text: &str, theme: &ThemeConfig) -> String {
        Self::styled_text(text, "\x1b[96m", theme)
    }

    pub fn bright_yellow(text: &str, theme: &ThemeConfig) -> String {
        Self::styled_text(text, "\x1b[93m", theme)
    }

    // Status indicators
    pub fn print_success(text: &str, theme: &ThemeConfig) {
        Self::print_styled(text, "\x1b[92m", theme);
    }

    pub fn print_error(text: &str, theme: &ThemeConfig) {
        Self::print_styled(text, "\x1b[91m", theme);
    }

    pub fn print_warning(text: &str, theme: &ThemeConfig) {
        Self::print_styled(text, "\x1b[93m", theme);
    }

    pub fn print_info(text: &str, theme: &ThemeConfig) {
        Self::print_styled(text, "\x1b[96m", theme);
    }

    pub fn success(text: &str, theme: &ThemeConfig) -> String {
        Self::bright_green(text, theme)
    }

    pub fn warning(text: &str, theme: &ThemeConfig) -> String {
        Self::bright_yellow(text, theme)
    }

    pub fn error(text: &str, theme: &ThemeConfig) -> String {
        Self::bright_red(text, theme)
    }

    pub fn info(text: &str, theme: &ThemeConfig) -> String {
        Self::bright_cyan(text, theme)
    }

    // Method to print regular text that respects the theme (uses theme foreground)
    pub fn print_themed(text: &str, theme: &ThemeConfig) {
        print!("{}{}",
            theme.background_color.to_ansi_bg_code(),
            theme.foreground_color.to_ansi_fg_code()
        );
        print!("{}", text);
        io::stdout().flush().unwrap();
    }

    pub fn println_themed(text: &str, theme: &ThemeConfig) {
        Self::print_themed(&format!("{}\n", text), theme);
    }

    pub fn themed(text: &str, theme: &ThemeConfig) -> String {
        format!("{}{}{}",
            theme.background_color.to_ansi_bg_code(),
            theme.foreground_color.to_ansi_fg_code(),
            text
        )
    }
}