
use wasm_bindgen::prelude::*;
use ui;

include!("./dist/simple.rs");

#[wasm_bindgen]
pub extern "C" fn simple(e: &web_sys::Element) -> simple::SimpleInterface {
	let component = simple::Simple {
		x: 100.0,
		y: 100.0,
		show: true,
		text: "Nice day for fucking you mother!".to_owned(),
	};
	component.attach_to_element(e)
}
