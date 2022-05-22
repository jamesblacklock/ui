
use wasm_bindgen::prelude::*;
use ui;

include!("./dist/simple.rs");

#[wasm_bindgen]
pub extern "C" fn simple(e: &web_sys::Element) -> simple::SimpleInterface {
	let props = simple::Props {
		x: ui::Length::Px(100.0),
		y: ui::Length::Px(100.0),
		strings: ui::Iterable::from([
			"string1",
			"string2",
		]),
		show: true,
		text: "hello world".to_owned(),
	};
	let component = simple::Simple::new(props);
	component.attach_to_element(e)
}
