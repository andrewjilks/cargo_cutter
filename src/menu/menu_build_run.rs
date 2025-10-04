// menu_build_run.rs (UPDATED)
use std::io::{self, Write};
use crate::build_system::BuildSystem;
use crate::ansi_theme::AnsiTheme;
use crate::config::ThemeConfig;

pub fn build_run_menu(build_system: &BuildSystem, project: &str, theme: &ThemeConfig) {
    loop {
        AnsiTheme::print_themed("\n", theme);
        AnsiTheme::print_yellow(&format!("Build & Run - {}\n", project), theme);
        AnsiTheme::print_themed("1) Build Project (debug)\n", theme);
        AnsiTheme::print_themed("2) Build Project (release)\n", theme);
        AnsiTheme::print_themed("3) Check Project (fast compile check)\n", theme);
        AnsiTheme::print_themed("4) Run Project\n", theme);
        AnsiTheme::print_themed("5) Run Project with Arguments\n", theme);
        AnsiTheme::print_themed("6) Run Tests\n", theme);
        AnsiTheme::print_themed("7) Clean Project\n", theme);
        AnsiTheme::print_themed("8) Show Build Info\n", theme);
        AnsiTheme::print_themed("9) Show Recent Logs\n", theme);
        AnsiTheme::print_themed("B) Back to main menu\n", theme);

        AnsiTheme::print_themed("Enter choice: ", theme);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        match choice {
            "1" => build_project(build_system, project, false, theme),
            "2" => build_project(build_system, project, true, theme),
            "3" => check_project(build_system, project, theme),
            "4" => run_project(build_system, project, &[], theme),
            "5" => run_with_args_menu(build_system, project, theme),
            "6" => test_project(build_system, project, theme),
            "7" => clean_project(build_system, project, theme),
            "8" => show_build_info(build_system, project, theme),
            "9" => show_recent_logs(theme),
            "B" | "b" => break,
            _ => AnsiTheme::print_themed("Invalid choice.\n", theme),
        }
    }
}

fn build_project(build_system: &BuildSystem, project: &str, release: bool, theme: &ThemeConfig) {
    let result = if release {
        build_system.build_release(project, theme)
    } else {
        build_system.build_project(project, theme)
    };

    match result {
        Ok(build_result) => {
            if build_result.success {
                AnsiTheme::print_success(&format!(" Build successful! ({:.2?})\n", build_result.duration), theme);
            } else {
                AnsiTheme::print_error(&format!(" Build failed! ({:.2?})\n", build_result.duration), theme);
            }
            
            if !build_result.stderr.is_empty() {
                AnsiTheme::print_themed("\n", theme);
                AnsiTheme::print_yellow("Build output:\n", theme);
                AnsiTheme::print_themed(&build_result.stderr, theme);
            }
            
            if !build_result.stdout.is_empty() {
                AnsiTheme::print_themed(&build_result.stdout, theme);
            }
        }
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn check_project(build_system: &BuildSystem, project: &str, theme: &ThemeConfig) {
    match build_system.check_project(project, theme) {
        Ok(check_result) => {
            if check_result.success {
                AnsiTheme::print_success(&format!(" Check passed! ({:.2?})\n", check_result.duration), theme);
            } else {
                AnsiTheme::print_error(&format!(" Check failed! ({:.2?})\n", check_result.duration), theme);
            }
            
            if !check_result.stderr.is_empty() {
                AnsiTheme::print_themed("\n", theme);
                AnsiTheme::print_yellow("Check output:\n", theme);
                AnsiTheme::print_themed(&check_result.stderr, theme);
            }
            
            if !check_result.stdout.is_empty() {
                AnsiTheme::print_themed(&check_result.stdout, theme);
            }
        }
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn run_project(build_system: &BuildSystem, project: &str, args: &[String], theme: &ThemeConfig) {
    let result = if args.is_empty() {
        build_system.run_project(project, theme)
    } else {
        build_system.run_with_args(project, args, theme)
    };

    match result {
        Ok(run_result) => {
            AnsiTheme::print_themed("\n", theme);
            AnsiTheme::print_green("Program output:\n", theme);
            AnsiTheme::print_themed(&run_result.stdout, theme);
            
            if !run_result.stderr.is_empty() {
                AnsiTheme::print_error("Program errors:\n", theme);
                AnsiTheme::print_themed(&run_result.stderr, theme);
            }
            
            AnsiTheme::print_blue(&format!(" Exit code: {} (Duration: {:.2?})\n", run_result.exit_code, run_result.duration), theme);
        }
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn run_with_args_menu(build_system: &BuildSystem, project: &str, theme: &ThemeConfig) {
    AnsiTheme::print_themed("\n", theme);
    AnsiTheme::print_themed("Enter arguments (space-separated, empty to cancel):\n", theme);
    AnsiTheme::print_themed("Arguments: ", theme);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    if input.is_empty() {
        AnsiTheme::print_themed("Cancelled.\n", theme);
        return;
    }

    let args: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
    run_project(build_system, project, &args, theme);
}

fn test_project(build_system: &BuildSystem, project: &str, theme: &ThemeConfig) {
    match build_system.test_project(project, theme) {
        Ok(test_result) => {
            if test_result.success {
                AnsiTheme::print_success(&format!(" Tests passed! ({:.2?})\n", test_result.duration), theme);
            } else {
                AnsiTheme::print_error(&format!(" Tests failed! ({:.2?})\n", test_result.duration), theme);
            }
            
            AnsiTheme::print_themed("\n", theme);
            AnsiTheme::print_cyan("Test output:\n", theme);
            AnsiTheme::print_themed(&test_result.stdout, theme);
            
            if !test_result.stderr.is_empty() {
                AnsiTheme::print_themed(&test_result.stderr, theme);
            }
        }
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn clean_project(build_system: &BuildSystem, project: &str, theme: &ThemeConfig) {
    match build_system.clean_project(project, theme) {
        Ok(()) => AnsiTheme::print_success(" Project cleaned successfully\n", theme),
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn show_build_info(build_system: &BuildSystem, project: &str, theme: &ThemeConfig) {
    match build_system.get_build_info(project, theme) {
        Ok(build_info) => {
            AnsiTheme::print_themed("\n", theme);
            AnsiTheme::print_blue("Build Information:\n", theme);
            AnsiTheme::print_themed(&format!("Debug build: {}\n", 
                if build_info.debug_exists { "✓ Available" } else { "✗ Not built" }), theme);
            AnsiTheme::print_themed(&format!("Release build: {}\n", 
                if build_info.release_exists { "✓ Available" } else { "✗ Not built" }), theme);
            
            if build_info.debug_exists {
                AnsiTheme::print_themed(&format!("Debug binary: {}\n", build_info.debug_path.display()), theme);
            }
            if build_info.release_exists {
                AnsiTheme::print_themed(&format!("Release binary: {}\n", build_info.release_path.display()), theme);
            }
        }
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn show_recent_logs(theme: &ThemeConfig) {
    AnsiTheme::print_themed("\n", theme);
    AnsiTheme::print_cyan("Recent Logs:\n", theme);
    if let Ok(content) = std::fs::read_to_string("build_tool.log") {
        let lines: Vec<&str> = content.lines().collect();
        let recent_lines = if lines.len() > 20 {
            &lines[lines.len() - 20..]
        } else {
            &lines
        };
        
        for line in recent_lines {
            if line.contains("ERROR") {
                AnsiTheme::print_error(&format!("{}\n", line), theme);
            } else if line.contains("WARN") {
                AnsiTheme::print_warning(&format!("{}\n", line), theme);
            } else if line.contains("INFO") {
                AnsiTheme::print_info(&format!("{}\n", line), theme);
            } else {
                AnsiTheme::print_themed(&format!("{}\n", line), theme);
            }
        }
    } else {
        AnsiTheme::print_themed("No log file found.\n", theme);
    }
}