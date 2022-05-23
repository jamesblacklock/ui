use wasm_bindgen::{ JsCast, prelude::* };
use js_sys::Array as JsArray;

use super::{
	Element,
	Group,
	Root,
	Rect,
	Span,
	Text,
	Length,
	Iterable,
};

impl Length {
	fn as_css(&self) -> String {
		match self {
			Length::Px(px) => format!("{px}px"),
			Length::In(nn) => format!("{nn}in"),
			Length::Cm(cm) => format!("{cm}cm"),
			Length::Mm(mm) => format!("{mm}mm"),
		}
	}
}

pub trait ConvertJsValue {
	fn from_js_value(value: JsValue) -> Self;
	fn js_value(&self) -> JsValue;
}

impl ConvertJsValue for Length {
	fn js_value(&self) -> JsValue {
		JsValue::from(self.as_css())
	}

	fn from_js_value(value: JsValue) -> Length {
		use regex::Regex;
		if let Some(s) = value.as_string() {
			let re = Regex::new(r"^(\d+(?:\.\d+)?|\.\d+)(px|in|cm|mm)$").unwrap();
			if let Some(captures) = re.captures(&s) {
				let f = str::parse::<f32>(&captures[1]).unwrap();
				return match &captures[2] {
					"px" => Length::Px(f),
					"in" => Length::In(f),
					"cm" => Length::Cm(f),
					"mm" => Length::Mm(f),
					_ => unreachable!(),
				}
			}
		} else if let Some(f) = value.as_f64() {
			return Length::Px(f as f32);
		}
		Length::Px(0.0)
	}
}

impl ConvertJsValue for bool {
	fn js_value(&self) -> JsValue {
		JsValue::from(*self)
	}

	fn from_js_value(value: JsValue) -> bool {
		value.as_bool().unwrap_or_default()
	}
}

impl ConvertJsValue for String {
	fn js_value(&self) -> JsValue {
		JsValue::from(self)
	}

	fn from_js_value(value: JsValue) -> String {
		value.as_string().unwrap_or_default()
	}
}

impl ConvertJsValue for i32 {
	fn js_value(&self) -> JsValue {
		JsValue::from_f64(*self as f64)
	}

	fn from_js_value(value: JsValue) -> i32 {
		value.as_f64().unwrap_or_default() as i32
	}
}

impl <T: ConvertJsValue> ConvertJsValue for Iterable<T> {
	fn js_value(&self) -> JsValue {
		match self {
			Iterable::Int(n) => n.js_value(),
			Iterable::Array(a) => {
				a.iter().enumerate().fold(
					JsArray::new_with_length(a.len() as u32),
					|acc, (i, e)| { acc.set(i as u32, e.js_value()); acc }
				)
				.into()
			},
		}
	}

	fn from_js_value(value: JsValue) -> Iterable<T> {
		if let Some(n) = value.as_f64() {
			<Iterable<T>>::Int(n as i32)
		} else if JsArray::is_array(&value) {
			let array = JsArray::from(&value);
			let vector = array.iter().fold(
				Vec::new(),
				|mut acc, e| { acc.push(T::from_js_value(e)); acc }
			);
			Iterable::Array(vector)
		} else {
			Iterable::Int(0)
		}
	}
}

#[derive(Debug)]
pub struct WebElement {
	pub node: Option<web_sys::Node>,
	pub active_group: Option<usize>,
	pub children: Vec<WebElement>,
	pub is_in: bool,
	pub last_in: Option<web_sys::Node>,
}

impl WebElement {
	pub fn new(e: Option<web_sys::Node>) -> WebElement {
		WebElement {
			node: e,
			active_group: None,
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
		console_error_panic_hook::set_once();
		if let Some(mut parent) = RenderWeb::render(self.element_impl.as_mut(), parent, i, self.show) {
			if self.group {
				group_in(parent, i);
			}
			for (i, e) in self.children.iter_mut().enumerate() {
				e.render(&mut parent, i, e.show);
			}
			parent.active_group = None;
			Some(parent)
		} else {
			None
		}
    }
}

fn group_in<'a>(parent: &'a mut WebElement, i: usize) {
	let result: Result<(), JsValue> = try {
		if parent.children.len() == i {
			parent.children.push(WebElement::new(None))
		} else if i > parent.children.len() {
			Err(&JsValue::from(&format!("i > parent.children.len() this should never happen!")))?;
		}
		parent.active_group = Some(i);
		return;
	};
	
	web_sys::console::error_1(&result.unwrap_err());
	panic!();
}

fn get_web_element<'a>(parent: &'a mut WebElement, i: usize) -> &'a mut WebElement {
	assert!(parent.node.is_some());

	let children = if let Some(group_index) = parent.active_group {
		&mut parent.children[group_index].children
	} else {
		&mut parent.children
	};
	&mut children[i]
}

fn get_html<'a>(parent: &'a mut WebElement, tag_or_content: &str, i: usize, is_text: bool) -> &'a mut WebElement {
	assert!(parent.node.is_some());

	let result: Result<(), JsValue> = try {
		let children = if let Some(group_index) = parent.active_group {
			&mut parent.children[group_index].children
		} else {
			&mut parent.children
		};
		if children.len() == i {
			let window = web_sys::window().ok_or(JsValue::from("window object is missing"))?;
			let document = window.document().ok_or(JsValue::from("document object is missing"))?;
			let e = if is_text {
				document.create_text_node(tag_or_content).dyn_into::<web_sys::Node>()?
			} else {
				document.create_element(tag_or_content)?.dyn_into::<web_sys::Node>()?
			};
			children.push(WebElement::new(Some(e)))
		} else if i > children.len() {
			Err(&JsValue::from(&format!("i > children.len() ({} > {}) this should never happen!", i, children.len())))?;
		}
		return &mut children[i];
	};
	
	web_sys::console::error_1(&result.unwrap_err());
	panic!();
}

fn html_in(parent: &mut WebElement, tag_or_content: &str, i: usize, is_text: bool) -> web_sys::Node {
	let result: Result<(), JsValue> = try {
		let last_in = parent.last_in.clone();
		let parent_node = parent.node.clone().unwrap();
		
		let node = {
			let e = get_html(parent, tag_or_content, i, is_text);
			e.last_in = None;
			if !e.is_in {
				if let Some(l) = last_in {
					if let Some(sibling) = l.next_sibling() {
						parent_node.insert_before(&e.node.as_ref().unwrap(), Some(&sibling))?;
					} else {
						parent_node.append_child(&e.node.as_ref().unwrap())?;
					} 
				} else {
					parent_node.insert_before(&e.node.as_ref().unwrap(), parent_node.first_child().as_ref())?;
				}
			}
			e.is_in = true;
			e.node.clone()
		};

		parent.last_in = node.clone();
		return node.unwrap();
	};

	web_sys::console::error_1(&result.unwrap_err());
	panic!();
}

fn html_out(parent: &mut WebElement, tag: &str, i: usize, is_text: bool) {
	let result: Result<(), JsValue> = try {
		let e = get_html(parent, tag, i, is_text);
		if e.is_in {
			if is_text {
				e.node.clone().unwrap().dyn_into::<web_sys::Text>()?.remove();
			} else {
				e.node.clone().unwrap().dyn_into::<web_sys::Element>()?.remove();
			}
			e.is_in = false;
		}
		return;
	};

	web_sys::console::error_1(&result.unwrap_err());
	panic!();
}

fn html_element_in(parent: &mut WebElement, tag: &str, i: usize) -> web_sys::HtmlElement {
	let result: Result<(), JsValue> = try {
		return html_in(parent, tag, i, false).dyn_into::<web_sys::HtmlElement>()?;
	};
	web_sys::console::error_1(&result.unwrap_err());
	panic!();
}
fn html_element_out(parent: &mut WebElement, tag: &str, i: usize) {
	html_out(parent, tag, i, false);
}

fn html_text_in(parent: &mut WebElement, content: &str, i: usize) -> web_sys::Text {
	let result: Result<(), JsValue> = try {
		let text = html_in(parent, content, i, true).dyn_into::<web_sys::Text>()?;
		text.set_text_content(Some(content));
		return text;
	};
	web_sys::console::error_1(&result.unwrap_err());
	panic!();
}

fn html_text_out(parent: &mut WebElement, content: &str, i: usize) {
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
				e.style().set_property("width", &self.bounds.width.as_css())?;
				e.style().set_property("height", &self.bounds.height.as_css())?;
				e.style().set_property("left", &self.bounds.x.as_css())?;
				e.style().set_property("top", &self.bounds.y.as_css())?;
			} else {
				html_element_out(parent, "div", i);
			}
			return Some(get_web_element(parent, i));
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
				e.style().set_property("left", &self.x.as_css())?;
				e.style().set_property("top", &self.y.as_css())?;
			} else {
				html_element_out(parent, "span", i);
			}
			return Some(get_web_element(parent, i));
		};

		web_sys::console::error_1(&result.unwrap_err());
		panic!();
    }
}

impl RenderWeb for Text {
	fn render<'a>(&mut self, parent: &'a mut WebElement, i: usize, show: bool) -> Option<&'a mut WebElement> {
		if show {
			html_text_in(parent, &self.content, i);
		} else {
			html_text_out(parent, &self.content, i);
		}
		return Some(get_web_element(parent, i));
    }
}

impl RenderWeb for Group {
    fn render<'a>(&mut self, parent: &'a mut WebElement, _i: usize, _show: bool) -> Option<&'a mut WebElement> {
		return Some(parent);
    }
}
