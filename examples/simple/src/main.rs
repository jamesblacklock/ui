use winit::window::WindowBuilder;

include!("./dist/simple.rs");

impl ui::Component for Simple {}

fn main() {
	let window_builder = WindowBuilder::new().with_title("hello world");
	let component = Simple::new(Props {
		x: ui::Length::Px(100.0),
		y: ui::Length::Px(100.0),
		show: true,
		strings: ui::Iterable::from(["string1", "string2"]),
		text: "O, she hath misused me past the endurance of a block".to_owned(),
		toggle_show: ui::Callback::from(&|this: &mut Simple| this.show = !this.show),
	});
	let window = ui::ComponentWindow::new(window_builder, component);
	pollster::block_on(window.run());
}