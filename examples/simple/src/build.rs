use ui_compiler;

fn main() {
    ui_compiler::build("ui", "src/simple.ui", true).unwrap();
}