
use wasm_bindgen::prelude::*;
use ui;

include!("./dist/simple.rs");

#[wasm_bindgen]
pub extern "C" fn simple(e: &web_sys::Element) -> simple::SimpleInterface {
	let component = simple::Simple {
		x: ui::Length::Px(100.0),
		y: ui::Length::Px(100.0),
		strings: ui::Iterable::from([
			"string1",
			"string2",
		]),
		show: true,
		text: "hello world".to_owned(),
	};
	component.attach_to_element(e)
}
