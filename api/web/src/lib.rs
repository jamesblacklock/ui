use std::{
	collections::HashMap,
	cell::{Cell},
	rc::Rc,
	alloc::{alloc, dealloc, Layout},
};

pub use ui_base::*;

#[link(wasm_import_module = "runtime")]
extern "C" {
	fn __console_log(ptr: *const u8, len: usize);
	fn __create_text_node(ptr: *const u8, len: usize) -> usize;
	fn __create_element(ptr: *const u8, len: usize) -> usize;
	fn __next_sibling(node: HtmlNode) -> HtmlNode;
	fn __insert_before(node: HtmlNode, insert: HtmlNode, reference: HtmlNode);
	fn __append_child(node: HtmlNode, child: HtmlNode);
	fn __first_child(node: HtmlNode) -> HtmlNode;
	fn __remove(node: HtmlNode);
	fn __set_text_content(node: HtmlNode, ptr: *const u8, len: usize);
	fn __set_style(node: HtmlNode, pptr: *const u8, plen: usize, vptr: *const u8, vlen: usize);
	fn __add_event_listener(node: HtmlNode, ptr: *const u8, len: usize, callback: JsValue);
	fn __remove_event_listener(node: HtmlNode, ptr: *const u8, len: usize, callback: JsValue);
	fn __heap_object_as_bool(object: JsValue) -> isize;
	fn __heap_object_stage_string(object: JsValue) -> isize;
	fn __heap_object_load_string(dest: *const u8);
	fn __heap_object_as_f32(object: JsValue) -> f32;
	fn __heap_object_is_function(object: JsValue) -> bool;
	fn __heap_object_call_function(object: JsValue);
	fn __heap_object_is_array(object: JsValue) -> bool;
	fn __heap_object_get_property(object: JsValue, keyptr: *const u8, keylen: usize) -> usize;
	fn __heap_object_drop(object: JsValue);
	fn __send_bool(value: bool) -> JsValue;
	fn __send_f32(value: f32) -> JsValue;
	pub fn __send_string(ptr: *const u8, len: usize) -> JsValue;
}

#[repr(transparent)]
#[derive(Debug)]
pub struct HtmlNode(pub usize);

impl HtmlNode {
	pub fn next_sibling(&self) -> Option<HtmlNode> {
		let node = unsafe { __next_sibling(HtmlNode(self.0)) };
		if node.0 == 0 {
			None
		} else {
			Some(node)
		}
	}
	pub fn insert_before(&self, insert: &HtmlNode, reference: Option<HtmlNode>) {
		let reference = if let Some(reference) = reference {
			reference
		} else {
			HtmlNode(0)
		};
		unsafe { __insert_before(HtmlNode(self.0), HtmlNode(insert.0), reference); }
	}
	pub fn append_child(&self, child: &HtmlNode) {
		unsafe { __append_child(HtmlNode(self.0), HtmlNode(child.0)); }
	}
	pub fn first_child(&self) -> Option<HtmlNode> {
		let node = unsafe { __first_child(HtmlNode(self.0)) };
		if node.0 == 0 {
			None
		} else {
			Some(node)
		}
	}
	pub fn remove(&self) {
		unsafe { __remove(HtmlNode(self.0)); }
	}
	pub fn set_text_content<S: AsRef<str>>(&self, content: S) {
		string_into_js(&content, |p, len| unsafe { __set_text_content(HtmlNode(self.0), p, len) });
	}
	pub fn set_style<S1: AsRef<str>, S2: AsRef<str>>(&self, property: S1, value: S2) {
		string_into_js(&property, |pp, plen| {
			string_into_js(&value, |vp, vlen| {
				unsafe { __set_style(HtmlNode(self.0), pp, plen, vp, vlen) }
			});
		});
	}
	pub fn add_event_listener(&self, event: &str, callback: Callback) {
		let js_value = callback.as_js_value();
		string_into_js(&event, |p, len| unsafe { __add_event_listener(HtmlNode(self.0), p, len, js_value) });
	}
	pub fn remove_event_listener(&self, event: &str, callback: &Callback) {
		let js_value = callback.as_js_value();
		string_into_js(&event, |p, len| unsafe { __remove_event_listener(HtmlNode(self.0), p, len, JsValue(js_value.0)) });
		std::mem::drop(js_value);
	}
}

impl Drop for HtmlNode {
	fn drop(&mut self) {
		if self.0 != 0 {
			unsafe { __heap_object_drop(JsValue(self.0)) }
		}
	}
}

pub fn string_into_js<S: AsRef<str>, T, F: FnOnce(*const u8, usize) -> T>(s: S, f: F) -> T {
	let s = s.as_ref();
	if s.len() == 0 {
		return f(0 as *const u8, 0);
	}
	let size = s.len();
	let layout = Layout::array::<u8>(size).unwrap();
	unsafe {
		let buf = alloc(layout);
		let s = &s.as_bytes()[0];
		std::intrinsics::copy_nonoverlapping(s, buf, size);
		let result = f(buf, size);
		dealloc(buf, layout);
		return result;
	}
}

pub fn console_log<S: AsRef<str>>(message: S) {
	string_into_js(&message, |p, len| unsafe { __console_log(p, len); });
}

fn create_text_node<S: AsRef<str>>(content: S) -> HtmlNode {
	let result = string_into_js(&content, |p, len| unsafe { __create_text_node(p, len) });
	HtmlNode(result)
}

fn create_element<S: AsRef<str>>(tag: S) -> HtmlNode {
	let result = string_into_js(&tag, |p, len| unsafe { __create_element(p, len) });
	HtmlNode(result)
}

#[repr(transparent)]
#[derive(Debug)]
pub struct JsValue(usize);

pub struct JsArrayIter {
	c: JsValue,
	i: usize,
}

impl std::iter::Iterator for JsArrayIter {
	type Item = JsValue;
	fn next(&mut self) -> Option<JsValue> {
		let key = self.i.to_string();
		let item = self.c.get_property(key);
		self.i += 1;
		item
	}
}

impl JsValue {
	pub fn as_bool(&self) -> Option<bool> {
		let result = unsafe { __heap_object_as_bool(JsValue(self.0)) };
		if result == 1 {
			Some(true)
		} else if result == 0 {
			Some(false)
		} else {
			None
		}
	}
	pub fn as_string(&self) -> Option<String> {
		let size = unsafe { __heap_object_stage_string(JsValue(self.0)) };
		if size < 0 {
			return None;
		} else if size == 0 {
			return Some(String::new())
		}

		let size = size as usize;
		let layout = Layout::array::<u8>(size).unwrap();
		unsafe {
			let buf = alloc(layout);
			__heap_object_load_string(buf);
			let slice = std::slice::from_raw_parts(buf, size);
			let result = String::from_utf8_unchecked(slice.to_vec());
			dealloc(buf, layout);
			return Some(result);
		}
	}
	pub fn as_f32(&self) -> Option<f32> {
		let result = unsafe { __heap_object_as_f32(JsValue(self.0)) };
		if f32::is_nan(result) {
			None
		} else {
			Some(result)
		}
	}
	pub fn is_function(&self) -> bool {
		unsafe { __heap_object_is_function(JsValue(self.0)) }
	}
	pub fn call_function(&self) {
		unsafe { __heap_object_call_function(JsValue(self.0)) }
	}
	pub fn is_array(&self) -> bool {
		unsafe { __heap_object_is_array(JsValue(self.0)) }
	}
	pub fn into_iter(self) -> Option<JsArrayIter> {
		if self.is_array() {
			Some(JsArrayIter { c: self, i: 0 })
		} else {
			None
		}
	}
	pub fn get_property<S: AsRef<str>>(&self, key: S) -> Option<JsValue> {
		let result = string_into_js(&key, |p, len| unsafe { __heap_object_get_property(JsValue(self.0), p, len) });
		if result == 0 {
			None
		} else {
			Some(JsValue(result))
		}
	}
	pub fn from_str(value: &str) -> JsValue {
		let result = string_into_js(value, |ptr, len| unsafe { __send_string(ptr, len) });
		assert!(result.0 != 0);
		result
	}
	pub fn from_bool(value: bool) -> JsValue {
		unsafe { __send_bool(value) }
	}
	pub fn from_f32(value: f32) -> JsValue {
		unsafe { __send_f32(value) }
	}
}

impl Drop for JsValue {
	fn drop(&mut self) {
		if self.0 != 0 {
			unsafe { __heap_object_drop(JsValue(self.0)) }
		}
	}
}

impl HostAbi for JsValue {
	fn call(&self) {
		self.call_function()
	}
}

type Element = ui_base::Element<JsValue>;
type Callback = ui_base::Callback<JsValue>;

#[no_mangle]
pub fn render_html(root: &mut Element, web_element: &mut WebElement) {
	RenderWeb::render(root, web_element, 0, true);
}

fn length_as_css(this: &Length) -> String {
	match this {
		Length::Px(px) => format!("{px}px"),
		Length::In(nn) => format!("{nn}in"),
		Length::Cm(cm) => format!("{cm}cm"),
		Length::Mm(mm) => format!("{mm}mm"),
	}
}

pub trait ConvertJsValue {
	fn from_js_value(value: JsValue) -> Self;
	fn as_js_value(&self) -> JsValue;
}

impl ConvertJsValue for Callback {
	fn as_js_value(&self) -> JsValue {
		let f = self.0.take();
		let result = match f {
			CallbackInner::Empty => unimplemented!(),
			CallbackInner::Native(_) => unimplemented!(),
			CallbackInner::HostAbi(ref js_value) => JsValue(js_value.0),
		};
		self.0.set(f);
		result
	}

	fn from_js_value(value: JsValue) -> Callback {
		if value.is_function() {
			Callback(Rc::new(Cell::new(CallbackInner::HostAbi(value))))
		} else {
			Callback::default()
		}
	}
}

impl ConvertJsValue for Length {
	fn as_js_value(&self) -> JsValue {
		JsValue::from_str(&length_as_css(self))
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
		} else if let Some(f) = value.as_f32() {
			return Length::Px(f);
		}
		Length::Px(0.0)
	}
}

impl ConvertJsValue for bool {
	fn as_js_value(&self) -> JsValue {
		JsValue::from_bool(*self)
	}

	fn from_js_value(value: JsValue) -> bool {
		value.as_bool().unwrap_or_default()
	}
}

impl ConvertJsValue for String {
	fn as_js_value(&self) -> JsValue {
		JsValue::from_str(self)
	}

	fn from_js_value(value: JsValue) -> String {
		value.as_string().unwrap_or_default()
	}
}

impl ConvertJsValue for i32 {
	fn as_js_value(&self) -> JsValue {
		JsValue::from_f32(*self as f32)
	}

	fn from_js_value(value: JsValue) -> i32 {
		value.as_f32().unwrap_or_default() as i32
	}
}

impl <T: ConvertJsValue> ConvertJsValue for Iterable<T> {
	fn as_js_value(&self) -> JsValue {
		// match self {
		// 	Iterable::Int(n) => n.js_value(),
		// 	Iterable::Array(a) => {
		// 		a.iter().enumerate().fold(
		// 			JsArray::new_with_length(a.len() as u32),
		// 			|acc, (i, e)| { acc.set(i as u32, e.js_value()); acc }
		// 		)
		// 		.into()
		// 	},
		// }
		unimplemented!()
	}

	fn from_js_value(value: JsValue) -> Iterable<T> {
		if let Some(n) = value.as_f32() {
			<Iterable<T>>::Int(n as i32)
		} else if let Some(iter) = value.into_iter() {
			let vector = iter.fold(
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
	pub node: Option<Rc<HtmlNode>>,
	pub events: HashMap<String, Callback>,
	pub active_group: Option<usize>,
	pub children: Vec<WebElement>,
	pub is_in: bool,
	pub last_in: Option<Rc<HtmlNode>>,
}

impl WebElement {
	pub fn new(e: Option<Rc<HtmlNode>>) -> WebElement {
		WebElement {
			node: e,
			events: HashMap::new(),
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
		if let Some(mut parent) = RenderWeb::render(&mut self.element_impl, parent, i, self.show) {
			if let Some(callback) = self.events.pointer_click.as_ref() {
				let node = parent.node.as_ref().unwrap();
				let current_callback = parent.events.get("click");
				if current_callback.map(|c| c != callback).unwrap_or(true) {
					if let Some(current_callback) = current_callback {
						node.remove_event_listener("click", &current_callback);
					}
					parent.events.insert("click".to_owned(), callback.clone());
					node.add_event_listener("click", callback.clone());
				}
			}
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

impl RenderWeb for ElementImpl {
	fn render<'a>(&mut self, parent: &'a mut WebElement, i: usize, show: bool) -> Option<&'a mut WebElement> {
		match self {
			ElementImpl::Root|ElementImpl::Group => Some(parent),
			ElementImpl::Rect(rect) => RenderWeb::render(rect, parent, i, show),
			ElementImpl::Span(span) => RenderWeb::render(span, parent, i, show),
			ElementImpl::Text(text) => RenderWeb::render(text, parent, i, show),
		}
	}
}

fn group_in<'a>(parent: &'a mut WebElement, i: usize) {
	if parent.children.len() == i {
		parent.children.push(WebElement::new(None))
	} else if i > parent.children.len() {
		console_log("i > parent.children.len() this should never happen!");
		panic!();
	}
	parent.active_group = Some(i);
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

	let children = if let Some(group_index) = parent.active_group {
		&mut parent.children[group_index].children
	} else {
		&mut parent.children
	};
	if children.len() == i {
		let e = if is_text {
			create_text_node(tag_or_content)
		} else {
			create_element(tag_or_content)
		};
		children.push(WebElement::new(Some(Rc::new(e))))
	} else if i > children.len() {
		console_log(format!("i > children.len() ({} > {}) this should never happen!", i, children.len()));
		panic!();
	}

	&mut children[i]
}

fn html_in(parent: &mut WebElement, tag_or_content: &str, i: usize, is_text: bool) -> Rc<HtmlNode> {
	let last_in = parent.last_in.clone();
	let parent_node = parent.node.clone().unwrap();
	
	let node = {
		let e = get_html(parent, tag_or_content, i, is_text);

		e.last_in = None;
		if !e.is_in {
			if let Some(l) = last_in {
				if let Some(sibling) = l.next_sibling() {
					parent_node.insert_before(e.node.as_ref().unwrap(), Some(sibling));
				} else {
					parent_node.append_child(e.node.as_ref().unwrap());
				} 
			} else {
				parent_node.insert_before(e.node.as_ref().unwrap(), parent_node.first_child());
			}
		}
		e.is_in = true;
		e.node.clone()
	};

	parent.last_in = node.clone();
	node.unwrap()
}

fn html_out(parent: &mut WebElement, tag: &str, i: usize, is_text: bool) {
	let e = get_html(parent, tag, i, is_text);
	if e.is_in {
		if is_text {
			e.node.as_ref().unwrap().remove();
		} else {
			e.node.as_ref().unwrap().remove();
		}
		e.is_in = false;
	}
}

fn html_element_in(parent: &mut WebElement, tag: &str, i: usize) -> Rc<HtmlNode> {
	html_in(parent, tag, i, false)
}

fn html_element_out(parent: &mut WebElement, tag: &str, i: usize) {
	html_out(parent, tag, i, false);
}

fn html_text_in(parent: &mut WebElement, content: &str, i: usize) -> Rc<HtmlNode> {
	let text = html_in(parent, content, i, true);
	text.set_text_content(content);
	text
}

fn html_text_out(parent: &mut WebElement, content: &str, i: usize) {
	html_out(parent, content, i, true);
}

impl RenderWeb for Rect {
	fn render<'a>(&mut self, parent: &'a mut WebElement, i: usize, show: bool) -> Option<&'a mut WebElement> {
		if show {
			let e = html_element_in(parent, "div", i);
			let r = (self.color.r * 255.0) as u8;
			let g = (self.color.g * 255.0) as u8;
			let b = (self.color.b * 255.0) as u8;
			let a = self.color.a;
			e.set_style("position", "absolute");
			e.set_style("background", &format!("rgba({},{},{},{})", r,g,b,a));
			e.set_style("width", length_as_css(&self.bounds.width));
			e.set_style("height", length_as_css(&self.bounds.height));
			e.set_style("left", length_as_css(&self.bounds.x));
			e.set_style("top", length_as_css(&self.bounds.y));
		} else {
			html_element_out(parent, "div", i);
		}
		return Some(get_web_element(parent, i));
	}
}

impl RenderWeb for Span {
	fn render<'a>(&mut self, parent: &'a mut WebElement, i: usize, show: bool) -> Option<&'a mut WebElement> {
		if show {
			let e = html_element_in(parent, "span", i);
			if let Some(max_width) = self.max_width {
				e.set_style("maxWidth", &format!("{}px", max_width));
			}
			e.set_style("left", length_as_css(&self.x));
			e.set_style("top", length_as_css(&self.y));
		} else {
			html_element_out(parent, "span", i);
		}
		Some(get_web_element(parent, i))
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
