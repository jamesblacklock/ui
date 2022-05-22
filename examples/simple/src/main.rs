use winit::window::WindowBuilder;
use ui;

include!("./dist/simple.rs");

fn main() {
	let window_builder = WindowBuilder::new().with_title("hello world");
	let window = ui::native::ComponentWindow::new(window_builder, simple::Simple { x: false });
	pollster::block_on(window.run());
}