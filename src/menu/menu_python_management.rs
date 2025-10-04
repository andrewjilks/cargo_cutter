// menu_python_management.rs
use std::io::{self, Write};
use crate::python_manager::PythonManager;
use crate::projects;
use crate::ansi_theme::AnsiTheme;
use crate::config::ThemeConfig;

pub fn python_management_menu(python_manager: &PythonManager, current_project: &Option<String>, theme: &ThemeConfig) {
    let project = match current_project {
        Some(p) => p.clone(),
        None => match projects::select_project() {
            Some(p) => p,
            None => return,
        },
    };

    loop {
        AnsiTheme::print_themed("\n", theme);
        AnsiTheme::print_green(&format!("Python Management - {}\n", project), theme);
        AnsiTheme::print_themed("1) Create Python File\n", theme);
        AnsiTheme::print_themed("2) Run Python Script\n", theme);
        AnsiTheme::print_themed("3) List Python Files\n", theme);
        AnsiTheme::print_themed("4) Create Virtual Environment\n", theme);
        AnsiTheme::print_themed("5) Install Python Package\n", theme);
        AnsiTheme::print_themed("6) Install Requirements\n", theme);
        AnsiTheme::print_themed("7) Start Python REPL\n", theme);
        AnsiTheme::print_themed("B) Back to main menu\n", theme);

        AnsiTheme::print_themed("Enter choice: ", theme);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        match choice {
            "1" => create_python_file_menu(python_manager, &project, theme),
            "2" => run_python_script_menu(python_manager, &project, theme),
            "3" => list_python_files(python_manager, &project, theme),
            "4" => create_venv_menu(python_manager, &project, theme),
            "5" => install_package_menu(python_manager, &project, theme),
            "6" => install_requirements_menu(python_manager, &project, theme),
            "7" => start_python_repl_menu(python_manager, &project, theme),
            "B" | "b" => break,
            _ => AnsiTheme::print_themed("Invalid choice.\n", theme),
        }
    }
}

fn create_python_file_menu(python_manager: &PythonManager, project: &str, theme: &ThemeConfig) {
    AnsiTheme::print_themed("\n", theme);
    AnsiTheme::print_green("Create Python file:\n", theme);
    AnsiTheme::print_themed("Enter file path (e.g., 'script.py', 'utils/helper.py'): ", theme);
    io::stdout().flush().unwrap();

    let mut path_input = String::new();
    io::stdin().read_line(&mut path_input).unwrap();
    let file_path = path_input.trim();

    if file_path.is_empty() {
        AnsiTheme::print_themed("File path cannot be empty.\n", theme);
        return;
    }

    AnsiTheme::print_themed("Enter Python code (end with 'EOF' on a new line):\n", theme);
    let mut content = String::new();
    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        
        if line.trim() == "EOF" {
            break;
        }
        content.push_str(&line);
    }

    match python_manager.create_python_file(project, file_path, &content, theme) {
        Ok(()) => AnsiTheme::print_success(" Python file created successfully!\n", theme),
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn run_python_script_menu(python_manager: &PythonManager, project: &str, theme: &ThemeConfig) {
    let file_path = match python_manager.select_file_from_list(project, theme) {
        Ok(Some(path)) => path,
        Ok(None) => return,
        Err(e) => {
            AnsiTheme::print_error(&format!("Error: {}\n", e), theme);
            return;
        }
    };

    AnsiTheme::print_themed("\n", theme);
    AnsiTheme::print_cyan("Enter arguments (space-separated, empty for no args):\n", theme);
    AnsiTheme::print_themed("Arguments: ", theme);
    io::stdout().flush().unwrap();

    let mut args_input = String::new();
    io::stdin().read_line(&mut args_input).unwrap();
    let args: Vec<String> = args_input.trim().split_whitespace().map(|s| s.to_string()).collect();

    match python_manager.run_python_script(project, &file_path, &args, theme) {
        Ok(result) => {
            AnsiTheme::print_themed("\n", theme);
            AnsiTheme::print_green("Python Output:\n", theme);
            if !result.stdout.is_empty() {
                AnsiTheme::print_themed(&result.stdout, theme);
            }
            if !result.stderr.is_empty() {
                AnsiTheme::print_error("Python Errors:\n", theme);
                AnsiTheme::print_themed(&result.stderr, theme);
            }
            AnsiTheme::print_blue(&format!(" Exit code: {}\n", result.exit_code), theme);
        }
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn list_python_files(python_manager: &PythonManager, project: &str, theme: &ThemeConfig) {
    match python_manager.list_python_files(project) {
        Ok(files) => {
            AnsiTheme::print_themed("\n", theme);
            AnsiTheme::print_blue("Python files in project:\n", theme);
            for (i, path) in files {
                let relative_path = path.strip_prefix(python_manager.get_project_path(project))
                    .unwrap_or(&path)
                    .display();
                AnsiTheme::print_themed(&format!("{:2}) {}\n", i, relative_path), theme);
            }
        }
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn create_venv_menu(python_manager: &PythonManager, project: &str, theme: &ThemeConfig) {
    match python_manager.create_venv(project, theme) {
        Ok(()) => AnsiTheme::print_success(" Virtual environment created\n", theme),
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn install_package_menu(python_manager: &PythonManager, project: &str, theme: &ThemeConfig) {
    AnsiTheme::print_themed("\n", theme);
    AnsiTheme::print_green("Install Python package:\n", theme);
    AnsiTheme::print_themed("Enter package name (e.g., 'requests', 'numpy==1.21.0'): ", theme);
    io::stdout().flush().unwrap();

    let mut package_input = String::new();
    io::stdin().read_line(&mut package_input).unwrap();
    let package = package_input.trim();

    if package.is_empty() {
        AnsiTheme::print_themed("Package name cannot be empty.\n", theme);
        return;
    }

    match python_manager.install_package(project, package, theme) {
        Ok(()) => AnsiTheme::print_success(" Package installed successfully\n", theme),
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn install_requirements_menu(python_manager: &PythonManager, project: &str, theme: &ThemeConfig) {
    match python_manager.install_requirements(project, theme) {
        Ok(()) => AnsiTheme::print_success(" Requirements installed successfully\n", theme),
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn start_python_repl_menu(python_manager: &PythonManager, project: &str, theme: &ThemeConfig) {
    match python_manager.start_python_repl(project, theme) {
        Ok(()) => AnsiTheme::print_success(" REPL session ended\n", theme),
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}