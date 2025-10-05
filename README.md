# 🗃️✂️ Cargo Cutter - Rust-Based Tooling Development Platform

Cargo Cutter is a CLI built entirely in Rust that enables and simplifies tool building on the Rust Platform, designed to integrate seamlessly with your development workflow. More than a simple CLI, this is a build tool that is easy to understand (I built it after learning Rust 2 days ago).

## 📁 Project Structure

```
rust_dev_terminal/
├── src/                                # Source code for the application
│   ├── text/                           # Text documents, primarily for development use
│   ├── menu/                           # Menu logic for modularity
│   │   ├── menu_build_run.rs           # Logic for build and run menu
│   │   ├── menu_cargo_management.rs    # Logic for cargo management menu
│   │   ├── menu_file_management.rs     # Logic for file management menu
│   │   ├── menu_git_management.rs      # Logic for git management menu
│   │   └── menu_project_creator.rs     # Logic for project creation menu
│   ├── analyzer.rs                     # Analyzer functionality (n/y/i)
│   ├── build_system.rs                 # Cargo building and scaffolding for new Rust projects
│   ├── config.rs                       # Configuration handling (e.g., project directory)
│   ├── file_manager.rs                 # File reading and management
│   ├── git_manager.rs                  # Git integration and controls
│   ├── main.rs                         # Application entry point
│   ├── menu.rs                         # Links menu logic to main application
│   ├── project_creator.rs              # Project creation and file handling
│   ├── projects.rs                     # Management of project files in other directories
│   └── self_update.rs                  # Self-build and update functionality
├── assets/                             # Static assets
│   └── my_icon.ico                     # Application icon
├── config/                             # Configuration files
│   └── workspace.txt                   # Legacy configuration artifact
├── .gitignore                          # Git ignore file
├── Cargo.toml                          # Project dependencies and metadata
├── icon.rc                             # Configuration for setting the application icon
├── build.rs                            # Build script for project-level resources
└── README.md                           # This documentation file
```

## 🛠 Features

- **Project Management**: Quickly create and modify Rust code with simple text input.
- **File Management**: Efficient structuring and staging for new and existing files.
- **Build Capabilities**: Rapidly build and test Rust code.
- **Toml Editing**: View version and dependency information for any project in the path.
- **Meta Features**: Self-build and update capabilities for bootstrapping new builds quickly.
- **Python Scripting**: Allow quick building and launching python scripts for automation.
- **Configuration**: Customization baby at least it'll help me in my learning.

## 📌 To-Do List

- [ ] Fix cargo cleaning
- [ ] Terminal/input mode
- [ ] Fix python paste crashiing bug
- [ ] Shortcuts/true CLI
- [ ] Text file clean up
- [ ] Dev docs rework
- [ ] Framework for text-config integration
- [ ] Create a simple GUI (optional)

## 📜 License

This project is licensed under the MIT License.

## 🏗 Contributors

**Andrew Jilks** - Creator & Developer
