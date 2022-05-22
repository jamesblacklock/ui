use wasm_bindgen::{ JsCast, prelude::* };

use super::{
	Element,
	Root,
	Rect,
	Span,
	Text,
};

pub struct WebElement {
	pub node: web_sys::Node,
	pub children: Vec<WebElement>,
	pub is_in: bool,
	pub last_in: Option<web_sys::Node>,
}

impl WebElement {
	pub fn new(e: web_sys::Node) -> WebElement {
		WebElement {
			node: e,
			children: Vec::new(),
			is_in: false,
			last_in: None,
		}
	}
}

pub trait RenderWeb {
    fn render<'a>(&mut self, parent: &'a mut WebElement, _i: usize, _show: bool) -> Option<&'a mut WebElement> {
		Some(parent)
	}
}

impl RenderWeb for Element {
    fn render<'a>(&mut self, parent: &'a mut WebElement, i: usize, _show: bool) -> Option<&'a mut WebElement> {
		if let Some(mut parent) = RenderWeb::render(self.element_impl.as_mut(), parent, i, self.show) {
			for (i, e) in self.children.iter_mut().enumerate() {
				e.render(&mut parent, i, e.show);
			}
			Some(parent)
		} else {
			None
		}
    }
}

fn get_html<'a>(parent: &'a mut WebElement, tag_or_content: &str, i: usize, is_text: bool) -> &'a mut WebElement {
	let result: Result<(), JsValue> = try {
		if parent.children.len() == i {
			let window = web_sys::window().ok_or(JsValue::from("window object is missing"))?;
			let document = window.document().ok_or(JsValue::from("document object is missing"))?;
			let e = if is_text {
				document.create_text_node(tag_or_content).dyn_into::<web_sys::Node>()?
			} else {
				document.create_element(tag_or_content)?.dyn_into::<web_sys::Node>()?
			};
			parent.children.push(WebElement::new(e))
		} else if i > parent.children.len() {
			Err(&JsValue::from(&format!("i > parent.children.len() this should never happen!")))?;
		}
		return &mut parent.children[i];
	};
	
	web_sys::console::error_1(&result.unwrap_err());
	panic!();
}

fn html_in<'a>(parent: &'a mut WebElement, tag_or_content: &str, i: usize, is_text: bool) -> web_sys::Node {
	let result: Result<(), JsValue> = try {
		let last_in = parent.last_in.clone();
		let parent_node = parent.node.clone();
		
		let node = {
			let e = get_html(parent, tag_or_content, i, is_text);
			if !e.is_in {
				if let Some(l) = last_in {
					if let Some(sibling) = l.next_sibling() {
						parent_node.insert_before(&e.node, Some(&sibling))?;
					} else {
						parent_node.append_child(&e.node)?;
					} 
				} else {
					parent_node.insert_before(&e.node, parent_node.first_child().as_ref())?;
				}
			}
			e.is_in = true;
			e.node.clone()
		};

		parent.last_in = Some(node.clone());
		return node;
	};

	web_sys::console::error_1(&result.unwrap_err());
	panic!();
}

fn html_out<'a>(parent: &'a mut WebElement, tag: &str, i: usize, is_text: bool) {
	let result: Result<(), JsValue> = try {
		let e = get_html(parent, tag, i, is_text);
		if e.is_in {
			if is_text { 
				e.node.clone().dyn_into::<web_sys::Text>()?.remove();
			} else {
				e.node.clone().dyn_into::<web_sys::Element>()?.remove();
			}
			e.is_in = false;
		}
		return;
	};

	web_sys::console::error_1(&result.unwrap_err());
	panic!();
}

fn html_element_in<'a>(parent: &'a mut WebElement, tag: &str, i: usize) -> web_sys::HtmlElement {
	let result: Result<(), JsValue> = try {
		return html_in(parent, tag, i, false).dyn_into::<web_sys::HtmlElement>()?;
	};
	web_sys::console::error_1(&result.unwrap_err());
	panic!();
}
fn html_element_out<'a>(parent: &'a mut WebElement, tag: &str, i: usize) {
	html_out(parent, tag, i, false);
}

fn html_text_in<'a>(parent: &'a mut WebElement, content: &str, i: usize) -> web_sys::Text {
	let result: Result<(), JsValue> = try {
		let text = html_in(parent, content, i, true).dyn_into::<web_sys::Text>()?;
		text.set_text_content(Some(content));
		return text;
	};
	web_sys::console::error_1(&result.unwrap_err());
	panic!();
}

fn html_text_out<'a>(parent: &'a mut WebElement, content: &str, i: usize) {
	html_out(parent, content, i, true);
}

impl RenderWeb for Root {}

impl RenderWeb for Rect {
	fn render<'a>(&mut self, parent: &'a mut WebElement, i: usize, show: bool) -> Option<&'a mut WebElement> {
		let result: Result<(), JsValue> = try {
			if show {
				let e = html_element_in(parent, "div", i);
				let r = (self.color.r * 255.0) as u8;
				let g = (self.color.g * 255.0) as u8;
				let b = (self.color.b * 255.0) as u8;
				let a = self.color.a;
				e.style().set_property("position", "absolute")?;
				e.style().set_property("background", &format!("rgba({},{},{},{})", r,g,b,a))?;
				e.style().set_property("width", &format!("{}px", self.bounds.width))?;
				e.style().set_property("height", &format!("{}px", self.bounds.height))?;
				e.style().set_property("left", &format!("{}px", self.bounds.x))?;
				e.style().set_property("top", &format!("{}px", self.bounds.y))?;
			} else {
				html_element_out(parent, "div", i);
			}
			return Some(&mut parent.children[i])
		};

		web_sys::console::error_1(&result.unwrap_err());
		panic!();
    }
}

impl RenderWeb for Span {
	fn render<'a>(&mut self, parent: &'a mut WebElement, i: usize, show: bool) -> Option<&'a mut WebElement> {
		let result: Result<(), JsValue> = try {
			if show {
				let e = html_element_in(parent, "span", i);
				if let Some(max_width) = self.max_width {
					e.style().set_property("maxWidth", &format!("{}px", max_width))?;
				}
				e.style().set_property("left", &format!("{}px", self.x))?;
				e.style().set_property("top", &format!("{}px", self.y))?;
			} else {
				html_element_out(parent, "span", i);
			}
			return Some(&mut parent.children[i])
		};

		web_sys::console::error_1(&result.unwrap_err());
		panic!();
    }
}

impl RenderWeb for Text {
	fn render<'a>(&mut self, parent: &'a mut WebElement, i: usize, show: bool) -> Option<&'a mut WebElement> {
		// let result: Result<(), JsValue> = try {
			if show {
				web_sys::console::log_1(&JsValue::from(&self.content));
				html_text_in(parent, &self.content, i);
			} else {
				html_text_out(parent, &self.content, i);
			}
			return Some(&mut parent.children[i])
		// };

		// web_sys::console::error_1(&result.unwrap_err());
		// panic!();
    }
}
