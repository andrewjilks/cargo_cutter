// python_manager.rs
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::fs;
use std::io::{self, Write};
use crate::ansi_theme::AnsiTheme;
use crate::config::ThemeConfig;

pub struct PythonManager {
    config: crate::config::AppConfig,
}

impl PythonManager {
    pub fn new_with_config(config: crate::config::AppConfig) -> Self {
        PythonManager { config }
    }

    pub fn new() -> Self {
        let config_manager = crate::config::ConfigManager::new();
        PythonManager { 
            config: config_manager.get_config().clone()
        }
    }

    pub fn get_project_path(&self, project_name: &str) -> PathBuf {
        self.config.get_project_path(project_name)
    }

    pub fn create_python_file(&self, project_name: &str, file_path: &str, content: &str, theme: &ThemeConfig) -> Result<(), String> {
        let full_path = self.config.get_project_path(project_name).join(file_path);
        
        // Ensure directory exists
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        fs::write(&full_path, content).map_err(|e| e.to_string())?;
        AnsiTheme::print_success(&format!(" Created Python file: {}\n", full_path.display()), theme);
        Ok(())
    }

    pub fn run_python_script(&self, project_name: &str, script_path: &str, args: &[String], theme: &ThemeConfig) -> Result<PythonRunResult, String> {
        let project_path = self.config.get_project_path(project_name);
        let full_script_path = project_path.join(script_path);
        
        if !full_script_path.exists() {
            return Err(format!("Python script not found: {}", full_script_path.display()));
        }

        AnsiTheme::print_themed(&format!("Running Python script: {}\n", script_path), theme);
        
        let mut command = Command::new("python");
        command.current_dir(&project_path).arg(&full_script_path);
        
        for arg in args {
            command.arg(arg);
        }

        command.stdout(Stdio::piped()).stderr(Stdio::piped());

        let output = command
            .output()
            .map_err(|e| format!("Failed to run Python script: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            AnsiTheme::print_success(" Python script executed successfully\n", theme);
        } else {
            AnsiTheme::print_error(" Python script execution failed\n", theme);
        }

        Ok(PythonRunResult {
            success: output.status.success(),
            stdout,
            stderr,
            exit_code: output.status.code().unwrap_or(-1),
        })
    }

    pub fn list_python_files(&self, project_name: &str) -> Result<Vec<(usize, PathBuf)>, String> {
        let project_path = self.config.get_project_path(project_name);
        let mut files = Vec::new();
        
        Self::walk_dir(&project_path, &mut files)?;
        
        let numbered_files = files.into_iter().enumerate().map(|(i, path)| (i + 1, path)).collect();
        Ok(numbered_files)
    }

    pub fn create_venv(&self, project_name: &str, theme: &ThemeConfig) -> Result<(), String> {
        let project_path = self.config.get_project_path(project_name);
        let _venv_path = project_path.join("venv");
        
        AnsiTheme::print_themed("Creating Python virtual environment...\n", theme);
        
        let output = Command::new("python")
            .current_dir(&project_path)
            .arg("-m")
            .arg("venv")
            .arg("venv")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to create virtual environment: {}", e))?;

        if output.status.success() {
            AnsiTheme::print_success(" Virtual environment created successfully\n", theme);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to create virtual environment: {}", stderr))
        }
    }

    pub fn install_package(&self, project_name: &str, package: &str, theme: &ThemeConfig) -> Result<(), String> {
        let project_path = self.config.get_project_path(project_name);
        let venv_python = project_path.join("venv").join("Scripts").join("python.exe");
        
        let python_cmd = if venv_python.exists() {
            venv_python.to_string_lossy().to_string()
        } else {
            "python".to_string()
        };

        AnsiTheme::print_themed(&format!("Installing package: {}\n", package), theme);
        
        let output = Command::new(&python_cmd)
            .current_dir(&project_path)
            .arg("-m")
            .arg("pip")
            .arg("install")
            .arg(package)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to install package: {}", e))?;

        if output.status.success() {
            AnsiTheme::print_success(" Package installed successfully\n", theme);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to install package: {}", stderr))
        }
    }

    pub fn install_requirements(&self, project_name: &str, theme: &ThemeConfig) -> Result<(), String> {
        let project_path = self.config.get_project_path(project_name);
        let requirements_path = project_path.join("requirements.txt");
        let venv_python = project_path.join("venv").join("Scripts").join("python.exe");
        
        if !requirements_path.exists() {
            return Err("requirements.txt not found".to_string());
        }

        let python_cmd = if venv_python.exists() {
            venv_python.to_string_lossy().to_string()
        } else {
            "python".to_string()
        };

        AnsiTheme::print_themed("Installing requirements...\n", theme);
        
        let output = Command::new(&python_cmd)
            .current_dir(&project_path)
            .arg("-m")
            .arg("pip")
            .arg("install")
            .arg("-r")
            .arg("requirements.txt")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to install requirements: {}", e))?;

        if output.status.success() {
            AnsiTheme::print_success(" Requirements installed successfully\n", theme);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to install requirements: {}", stderr))
        }
    }

    pub fn start_python_repl(&self, project_name: &str, theme: &ThemeConfig) -> Result<(), String> {
        let project_path = self.config.get_project_path(project_name);
        let venv_python = project_path.join("venv").join("Scripts").join("python.exe");
        
        let python_cmd = if venv_python.exists() {
            venv_python.to_string_lossy().to_string()
        } else {
            "python".to_string()
        };

        AnsiTheme::print_themed("Starting Python REPL...\n", theme);
        AnsiTheme::print_themed("Type 'exit()' or press Ctrl+Z then Enter to exit\n", theme);
        
        let status = Command::new(&python_cmd)
            .current_dir(&project_path)
            .status()
            .map_err(|e| format!("Failed to start Python REPL: {}", e))?;

        if status.success() {
            AnsiTheme::print_success(" Python REPL session ended\n", theme);
            Ok(())
        } else {
            Err("Python REPL exited with error".to_string())
        }
    }

    fn walk_dir(dir: &PathBuf, files: &mut Vec<PathBuf>) -> Result<(), String> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                
                // Skip the 'venv' directory
                if path.is_dir() && path.file_name().map(|name| name == "venv").unwrap_or(false) {
                    continue;
                }

                if path.is_dir() {
                    Self::walk_dir(&path, files)?;
                } else if path.extension().map_or(false, |ext| ext == "py") {
                    files.push(path);
                }
            }
        }
        Ok(())
    }

    pub fn select_file_from_list(&self, project_name: &str, theme: &ThemeConfig) -> Result<Option<String>, String> {
        let files = self.list_python_files(project_name)?;
        
        if files.is_empty() {
            AnsiTheme::print_warning(" No Python files found in project.\n", theme);
            return Ok(None);
        }

        AnsiTheme::print_themed("\n", theme);
        AnsiTheme::print_blue("Python files in project:\n", theme);
        for (i, path) in &files {
            let relative_path = path.strip_prefix(self.config.get_project_path(project_name))
                .unwrap_or(path)
                .display();
            AnsiTheme::print_themed(&format!("{:2}) {}\n", i, relative_path), theme);
        }

        AnsiTheme::print_themed(" 0) Cancel\n", theme);
        
        let selection = Self::get_number_input("Select file by number: ", 0, files.len(), theme);
        
        match selection {
            Some(0) => {
                AnsiTheme::print_themed("Selection cancelled.\n", theme);
                Ok(None)
            }
            Some(index) if index <= files.len() => {
                let selected_path = files[index - 1].1.clone();
                let relative_path = selected_path.strip_prefix(self.config.get_project_path(project_name))
                    .unwrap_or(&selected_path)
                    .to_string_lossy()
                    .to_string();
                Ok(Some(relative_path))
            }
            _ => {
                AnsiTheme::print_themed("Invalid selection.\n", theme);
                Ok(None)
            }
        }
    }

    fn get_number_input(prompt: &str, min: usize, max: usize, theme: &ThemeConfig) -> Option<usize> {
        AnsiTheme::print_themed(prompt, theme);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.eq_ignore_ascii_case("q") || input.eq_ignore_ascii_case("cancel") {
            return None;
        }

        match input.parse::<usize>() {
            Ok(num) if num >= min && num <= max => Some(num),
            _ => {
                AnsiTheme::print_themed(&format!("Please enter a number between {} and {}.\n", min, max), theme);
                None
            }
        }
    }
}

#[derive(Debug)]
pub struct PythonRunResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}