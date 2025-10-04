// build_system.rs (CONVERTED)
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Utc;
use std::backtrace::{Backtrace, BacktraceStatus};
use crate::ansi_theme::AnsiTheme;
use crate::config::ThemeConfig;

#[derive(Debug, Clone)]
pub struct BuildLogger {
    log_file: Option<String>,
    verbose: bool,
}

impl BuildLogger {
    pub fn new(log_file: Option<String>, verbose: bool) -> Self {
        BuildLogger { log_file, verbose }
    }

    pub fn log(&self, level: &str, message: &str, context: Option<&str>, theme: &ThemeConfig) {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let context_str = context.map(|c| format!(" [{}]", c)).unwrap_or_default();
        let log_line = format!("{} {}{}: {}", timestamp, level, context_str, message);

        // Always print to console if verbose or if it's an error/warning
        if self.verbose || level == "ERROR" || level == "WARN" {
            match level {
                "ERROR" => AnsiTheme::print_error(&log_line, theme),
                "WARN" => AnsiTheme::print_warning(&log_line, theme),
                "INFO" => AnsiTheme::print_info(&log_line, theme),
                "DEBUG" => AnsiTheme::print_themed(&log_line, theme),
                _ => AnsiTheme::print_themed(&log_line, theme),
            }
        }

        // Write to log file if specified
        if let Some(ref log_path) = self.log_file {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
            {
                let _ = writeln!(file, "{}", log_line);
            }
        }
    }

    pub fn log_error_with_backtrace(&self, error: &str, context: Option<&str>, theme: &ThemeConfig) {
        let backtrace = Backtrace::capture();
        self.log("ERROR", error, context, theme);
        
        if backtrace.status() == BacktraceStatus::Captured {
            self.log("DEBUG", &format!("Backtrace:\n{}", backtrace), context, theme);
        }
    }

    pub fn log_command(&self, command: &str, args: &[String], context: Option<&str>, theme: &ThemeConfig) {
        let full_command = if args.is_empty() {
            command.to_string()
        } else {
            format!("{} {}", command, args.join(" "))
        };
        self.log("DEBUG", &format!("Executing: {}", full_command), context, theme);
    }
}

pub struct BuildSystem {
    logger: BuildLogger,
}

impl BuildSystem {
    pub fn new(log_file: Option<String>, verbose: bool) -> Self {
        let logger = BuildLogger::new(log_file, verbose);
        
        // Note: We can't log here since we don't have theme context
        // Logging will happen in methods that receive theme parameter

        BuildSystem { logger }
    }

    // Helper method to execute commands with consistent logging and backtrace capture
    fn execute_command(
        &self,
        command: &str,
        args: &[&str],
        project_path: &PathBuf,
        context: &str,
        capture_child_output: bool,
        theme: &ThemeConfig,
    ) -> Result<(std::process::Output, std::time::Duration), String> {
        let start_time = std::time::Instant::now();

        self.logger.log("DEBUG", &format!("Project path: {}", project_path.display()), Some(context), theme);
        
        let mut cmd = Command::new(command);
        cmd.current_dir(project_path)
           .env("RUST_BACKTRACE", "1");  // Enable backtraces in child processes

        for arg in args {
            cmd.arg(arg);
        }

        // Only capture output if requested (for operations that produce meaningful output)
        if capture_child_output {
            cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        }

        self.logger.log("DEBUG", &format!("Executing: {} {}", command, args.join(" ")), Some(context), theme);

        let output = cmd.output().map_err(|e| {
            let error_msg = format!("Failed to execute {} {}: {}", command, args.join(" "), e);
            self.logger.log_error_with_backtrace(&error_msg, Some(context), theme);
            error_msg
        })?;

        let duration = start_time.elapsed();

        // Log child process output if we captured it
        if capture_child_output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if !stdout.trim().is_empty() {
                self.logger.log("DEBUG", &format!("Child process stdout:\n{}", stdout), Some(context), theme);
            }
            if !stderr.trim().is_empty() {
                let log_level = if output.status.success() { "DEBUG" } else { "ERROR" };
                self.logger.log(log_level, &format!("Child process stderr:\n{}", stderr), Some(context), theme);
            }
        }

        Ok((output, duration))
    }

    // Helper to validate project existence
    fn validate_project_exists(&self, project_name: &str, context: &str, theme: &ThemeConfig) -> Result<PathBuf, String> {
        let project_path = PathBuf::from("D:\\RustProjects").join(project_name);
        
        if !project_path.exists() {
            let error_msg = format!("Project '{}' does not exist!", project_name);
            self.logger.log_error_with_backtrace(&error_msg, Some(context), theme);
            return Err(error_msg);
        }

        Ok(project_path)
    }

    pub fn build_self(&self, theme: &ThemeConfig) -> Result<BuildResult, String> {
        let context = "self-build";
        self.logger.log("INFO", "Starting self-build", Some(context), theme);
        
        let current_dir = std::env::current_dir()
            .map_err(|e| {
                let error_msg = format!("Failed to get current directory: {}", e);
                self.logger.log_error_with_backtrace(&error_msg, Some(context), theme);
                error_msg
            })?;

        self.logger.log("DEBUG", &format!("Building in directory: {}", current_dir.display()), Some(context), theme);
        
        let (output, duration) = self.execute_command(
            "cargo",
            &["build", "--release"],
            &current_dir,
            context,
            true,  // Capture output for self-build
            theme,
        )?;

        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if success {
            self.logger.log("INFO", &format!("Self-build completed successfully in {:.2?}", duration), Some(context), theme);
        } else {
            self.logger.log("ERROR", &format!("Self-build failed after {:.2?}", duration), Some(context), theme);
        }

        Ok(BuildResult {
            success,
            stdout,
            stderr,
            duration,
        })
    }

    pub fn get_self_binary_path(&self, theme: &ThemeConfig) -> Result<PathBuf, String> {
        let context = "binary-path-lookup";

        let current_exe = std::env::current_exe()
            .map_err(|e| {
                let error_msg = format!("Failed to get current executable path: {}", e);
                self.logger.log_error_with_backtrace(&error_msg, Some(context), theme);
                error_msg
            })?;
        
        let current_dir = current_exe.parent()
            .ok_or_else(|| {
                let error_msg = "Failed to get parent directory of current executable".to_string();
                self.logger.log_error_with_backtrace(&error_msg, Some(context), theme);
                error_msg
            })?;
        
        let target_dir = current_dir.join("target").join("release");
        let binary_name = if cfg!(windows) { "rust_dev_terminal.exe" } else { "rust_dev_terminal" };
        let binary_path = target_dir.join(binary_name);
        
        self.logger.log("DEBUG", &format!("Looking for binary at: {}", binary_path.display()), Some(context), theme);
        
        if binary_path.exists() {
            self.logger.log("DEBUG", "Binary found at primary location", Some(context), theme);
            Ok(binary_path)
        } else {
            self.logger.log("WARN", "Binary not found at primary location, searching project root", Some(context), theme);
            if let Some(project_root) = Self::find_project_root(&current_dir.to_path_buf()) {
                let alt_binary_path = project_root.join("target").join("release").join(binary_name);
                self.logger.log("DEBUG", &format!("Alternative binary path: {}", alt_binary_path.display()), Some(context), theme);
                if alt_binary_path.exists() {
                    self.logger.log("DEBUG", "Binary found at alternative location", Some(context), theme);
                    return Ok(alt_binary_path);
                }
            }
            let error_msg = format!("Binary not found at: {}", binary_path.display());
            self.logger.log_error_with_backtrace(&error_msg, Some(context), theme);
            Err(error_msg)
        }
    }

    fn find_project_root(start_dir: &PathBuf) -> Option<PathBuf> {
        let mut current = start_dir.clone();
        
        while current.parent().is_some() {
            if current.join("Cargo.toml").exists() {
                return Some(current);
            }
            current = current.parent()?.to_path_buf();
        }
        
        None
    }

    pub fn build_project(&self, project_name: &str, theme: &ThemeConfig) -> Result<BuildResult, String> {
        let context = &format!("build:{}", project_name);
        self.logger.log("INFO", &format!("Starting build for project: {}", project_name), Some(context), theme);
        
        let project_path = self.validate_project_exists(project_name, context, theme)?;

        let (output, duration) = self.execute_command(
            "cargo",
            &["build"],
            &project_path,
            context,
            true,  // Capture build output
            theme,
        )?;

        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if success {
            self.logger.log("INFO", &format!("Build completed successfully in {:.2?}", duration), Some(context), theme);
        } else {
            self.logger.log("ERROR", &format!("Build failed after {:.2?}", duration), Some(context), theme);
        }

        Ok(BuildResult {
            success,
            stdout,
            stderr,
            duration,
        })
    }

    pub fn build_release(&self, project_name: &str, theme: &ThemeConfig) -> Result<BuildResult, String> {
        let context = &format!("release-build:{}", project_name);
        self.logger.log("INFO", &format!("Starting release build for project: {}", project_name), Some(context), theme);
        
        let project_path = self.validate_project_exists(project_name, context, theme)?;

        let (output, duration) = self.execute_command(
            "cargo",
            &["build", "--release"],
            &project_path,
            context,
            true,  // Capture build output
            theme,
        )?;

        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if success {
            self.logger.log("INFO", &format!("Release build completed successfully in {:.2?}", duration), Some(context), theme);
        } else {
            self.logger.log("ERROR", &format!("Release build failed after {:.2?}", duration), Some(context), theme);
        }

        Ok(BuildResult {
            success,
            stdout,
            stderr,
            duration,
        })
    }

    pub fn check_project(&self, project_name: &str, theme: &ThemeConfig) -> Result<BuildResult, String> {
        let context = &format!("check:{}", project_name);
        self.logger.log("INFO", &format!("Checking project: {}", project_name), Some(context), theme);
        
        let project_path = self.validate_project_exists(project_name, context, theme)?;

        let (output, duration) = self.execute_command(
            "cargo",
            &["check"],
            &project_path,
            context,
            true,  // Capture check output
            theme,
        )?;

        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if success {
            self.logger.log("INFO", &format!("Check completed successfully in {:.2?}", duration), Some(context), theme);
        } else {
            self.logger.log("ERROR", &format!("Check failed after {:.2?}", duration), Some(context), theme);
        }

        Ok(BuildResult {
            success,
            stdout,
            stderr,
            duration,
        })
    }

    pub fn run_project(&self, project_name: &str, theme: &ThemeConfig) -> Result<RunResult, String> {
        let context = &format!("run:{}", project_name);
        self.logger.log("INFO", &format!("Running project: {}", project_name), Some(context), theme);
        
        let project_path = self.validate_project_exists(project_name, context, theme)?;

        let (output, duration) = self.execute_command(
            "cargo",
            &["run"],
            &project_path,
            context,
            true,  // Capture run output (especially important for backtraces)
            theme,
        )?;

        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        self.logger.log("INFO", &format!("Execution completed in {:.2?} with exit code: {}", 
            duration, output.status.code().unwrap_or(-1)), Some(context), theme);

        Ok(RunResult {
            success,
            stdout,
            stderr,
            exit_code: output.status.code().unwrap_or(-1),
            duration,
        })
    }

    pub fn run_with_args(&self, project_name: &str, args: &[String], theme: &ThemeConfig) -> Result<RunResult, String> {
        let context = &format!("run-with-args:{}", project_name);
        self.logger.log("INFO", &format!("Running project with arguments: {}", project_name), Some(context), theme);
        self.logger.log("DEBUG", &format!("Arguments: {:?}", args), Some(context), theme);
        
        let project_path = self.validate_project_exists(project_name, context, theme)?;

        // Build command with dynamic args
        let mut cmd_args = vec!["run"];
        for arg in args {
            cmd_args.push(arg);
        }

        let (output, duration) = self.execute_command(
            "cargo",
            &cmd_args,
            &project_path,
            context,
            true,  // Capture run output with args
            theme,
        )?;

        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        self.logger.log("INFO", &format!("Execution with args completed in {:.2?} with exit code: {}", 
            duration, output.status.code().unwrap_or(-1)), Some(context), theme);

        Ok(RunResult {
            success,
            stdout,
            stderr,
            exit_code: output.status.code().unwrap_or(-1),
            duration,
        })
    }

    pub fn clean_project(&self, project_name: &str, theme: &ThemeConfig) -> Result<(), String> {
        let context = &format!("clean:{}", project_name);
        self.logger.log("INFO", &format!("Cleaning project: {}", project_name), Some(context), theme);
        
        let project_path = self.validate_project_exists(project_name, context, theme)?;

        let (output, duration) = self.execute_command(
            "cargo",
            &["clean"],
            &project_path,
            context,
            false,  // No need to capture clean output
            theme,
        )?;

        if output.status.success() {
            self.logger.log("INFO", &format!("Project cleaned successfully in {:.2?}", duration), Some(context), theme);
            Ok(())
        } else {
            let error_msg = "Failed to clean project".to_string();
            self.logger.log_error_with_backtrace(&error_msg, Some(context), theme);
            Err(error_msg)
        }
    }

    pub fn test_project(&self, project_name: &str, theme: &ThemeConfig) -> Result<TestResult, String> {
        let context = &format!("test:{}", project_name);
        self.logger.log("INFO", &format!("Running tests for: {}", project_name), Some(context), theme);
        
        let project_path = self.validate_project_exists(project_name, context, theme)?;

        let (output, duration) = self.execute_command(
            "cargo",
            &["test"],
            &project_path,
            context,
            true,  // Capture test output
            theme,
        )?;

        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if success {
            self.logger.log("INFO", &format!("Tests completed successfully in {:.2?}", duration), Some(context), theme);
        } else {
            self.logger.log("ERROR", &format!("Tests failed after {:.2?}", duration), Some(context), theme);
        }

        Ok(TestResult {
            success,
            stdout,
            stderr,
            duration,
        })
    }

    pub fn get_build_info(&self, project_name: &str, theme: &ThemeConfig) -> Result<BuildInfo, String> {
        let context = &format!("build-info:{}", project_name);
        self.logger.log("INFO", &format!("Getting build info for: {}", project_name), Some(context), theme);
        
        let project_path = self.validate_project_exists(project_name, context, theme)?;
        let debug_binary = project_path.join("target").join("debug").join(project_name);
        let release_binary = project_path.join("target").join("release").join(project_name);

        self.logger.log("DEBUG", &format!("Debug binary path: {}", debug_binary.display()), Some(context), theme);
        self.logger.log("DEBUG", &format!("Release binary path: {}", release_binary.display()), Some(context), theme);

        Ok(BuildInfo {
            debug_exists: debug_binary.exists(),
            release_exists: release_binary.exists(),
            debug_path: debug_binary,
            release_path: release_binary,
        })
    }

    // Debug method to test backtrace logging
    pub fn test_backtrace_logging(&self, theme: &ThemeConfig) -> Result<(), String> {
        let context = "backtrace-test";
        self.logger.log("INFO", "Testing backtrace logging...", Some(context), theme);
        
        // Force an error to demonstrate backtrace
        std::fs::read_to_string("THIS_FILE_DOES_NOT_EXIST_12345.txt")
            .map_err(|e| {
                let error_msg = format!("Forced file error: {}", e);
                self.logger.log_error_with_backtrace(&error_msg, Some(context), theme);
                error_msg
            })
            .map(|_| ()) // Discard the string content if it somehow succeeds
    }

    pub fn open_log_file(&self, theme: &ThemeConfig) -> Result<(), String> {
        let log_path = self.get_log_file_path();
        
        if !log_path.exists() {
            return Err(format!("Log file does not exist: {}", log_path.display()));
        }

        AnsiTheme::print_info(&format!("Opening log file: {}\n", log_path.display()), theme);
        
        #[cfg(target_os = "windows")]
        let status = std::process::Command::new("notepad.exe")
            .arg(&log_path)
            .status();

        #[cfg(target_family = "unix")]
        let status = std::process::Command::new("xdg-open")
            .arg(&log_path)
            .status();

        #[cfg(target_os = "macos")]
        let status = std::process::Command::new("open")
            .arg(&log_path)
            .status();

        match status {
            Ok(status) if status.success() => {
                AnsiTheme::print_success(" Log file opened successfully\n", theme);
                Ok(())
            }
            Ok(_) => Err("Failed to open log file (editor exited with error)".to_string()),
            Err(e) => Err(format!("Failed to open log file: {}", e)),
        }
    }

    pub fn get_log_file_path(&self) -> std::path::PathBuf {
        if let Some(ref log_path) = self.logger.log_file {
            std::path::PathBuf::from(log_path)
        } else {
            std::path::PathBuf::from("build_tool.log")
        }
    }

    pub fn show_log_info(&self, theme: &ThemeConfig) {
        let log_path = self.get_log_file_path();
        
        AnsiTheme::print_themed("\n", theme);
        AnsiTheme::print_info("Log File Information:\n", theme);
        AnsiTheme::print_themed(&format!("Path: {}\n", log_path.display()), theme);
        
        if log_path.exists() {
            if let Ok(metadata) = std::fs::metadata(&log_path) {
                AnsiTheme::print_themed(&format!("Size: {} bytes\n", metadata.len()), theme);
                
                if let Ok(contents) = std::fs::read_to_string(&log_path) {
                    let line_count = contents.lines().count();
                    AnsiTheme::print_themed(&format!("Lines: {}\n", line_count), theme);
                    
                    // Show last modified time if available
                    if let Ok(modified) = metadata.modified() {
                        use chrono::{DateTime, Local};
                        let datetime: DateTime<Local> = modified.into();
                        AnsiTheme::print_themed(&format!("Last modified: {}\n", datetime.format("%Y-%m-%d %H:%M:%S")), theme);
                    }
                }
            }
        } else {
            AnsiTheme::print_warning("Status: Not created yet\n", theme);
            AnsiTheme::print_themed("Run a build operation to create the log file\n", theme);
        }
    }
}

#[derive(Debug)]
pub struct BuildResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub duration: std::time::Duration,
}

#[derive(Debug)]
pub struct RunResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration: std::time::Duration,
}

#[derive(Debug)]
pub struct TestResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub duration: std::time::Duration,
}

#[derive(Debug)]
pub struct BuildInfo {
    pub debug_exists: bool,
    pub release_exists: bool,
    pub debug_path: PathBuf,
    pub release_path: PathBuf,
}