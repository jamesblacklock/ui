
use wasm_bindgen::prelude::*;
use ui;

include!("./dist/simple.rs");

#[wasm_bindgen]
pub fn simple(e: &web_sys::Element, props: &JsValue) -> simple::SimpleInterface {
	let component = simple::Simple::new(simple::Props::from(props));
	component.attach_to_element(e)
}