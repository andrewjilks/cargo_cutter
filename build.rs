fn main() {
    winresource::WindowsResource::new()
        .set_icon("assets/my_icon.ico")
        .compile()
        .expect("Failed to compile resources");
}
