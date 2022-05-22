use winit::window::WindowBuilder;
use ui;

include!("./dist/simple.rs");

fn main() {
	let window_builder = WindowBuilder::new().with_title("hello world");
	let window = ui::native::ComponentWindow::new(
		window_builder,
		simple::Simple {
			x: ui::Length::Px(180.0),
			y: ui::Length::Px(260.0),
			show: true,
			strings: ui::Iterable::from(["x", "y", "z"]),
			text: "hello world".to_owned(),
		});
	pollster::block_on(window.run());
}