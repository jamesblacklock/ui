use ui_compiler;

fn main() {
    ui_compiler::build("ui", "src/simple.ui", std::env::var("BUILD_WEB").is_ok()).unwrap();
}