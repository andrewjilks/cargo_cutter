// terminal_controller.rs
use crate::config::TerminalColor;

pub struct TerminalThemeManager {
    controller: TerminalController,
}

impl TerminalThemeManager {
    pub fn new(background: &TerminalColor, foreground: &TerminalColor) -> Self {
        let controller = TerminalController::new();
        controller.set_colors(background, foreground);
        TerminalThemeManager { controller }
    }
}

impl Drop for TerminalThemeManager {
    fn drop(&mut self) {
        self.controller.reset_colors();
    }
}

pub struct TerminalController;

impl TerminalController {
    pub fn new() -> Self {
        TerminalController
    }

    pub fn set_colors(&self, background: &TerminalColor, foreground: &TerminalColor) {
        // Set background color
        match background {
            TerminalColor::Default => print!("\x1b[49m"),
            TerminalColor::Black => print!("\x1b[40m"),
            TerminalColor::White => print!("\x1b[47m"),
            TerminalColor::Red => print!("\x1b[41m"),
            TerminalColor::Green => print!("\x1b[42m"),
            TerminalColor::Blue => print!("\x1b[44m"),
            TerminalColor::Rgb(r, g, b) => print!("\x1b[48;2;{};{};{}m", r, g, b),
        }

        // Set foreground color
        match foreground {
            TerminalColor::Default => print!("\x1b[39m"),
            TerminalColor::Black => print!("\x1b[30m"),
            TerminalColor::White => print!("\x1b[37m"),
            TerminalColor::Red => print!("\x1b[31m"),
            TerminalColor::Green => print!("\x1b[32m"),
            TerminalColor::Blue => print!("\x1b[34m"),
            TerminalColor::Rgb(r, g, b) => print!("\x1b[38;2;{};{};{}m", r, g, b),
        }
        io::stdout().flush().unwrap();
    }

    pub fn reset_colors(&self) {
        print!("\x1b[0m");
	io::stdout().flush().unwrap();
    }
}
