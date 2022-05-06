use std::fmt::Debug;
use std::collections::HashMap;

use super::{
	web::{RenderWeb, WebRenderer, HtmlElement},
	parser::Element as ParserElement,
	Module,
	Value,
};

pub type Constructor = fn(&Module, &mut ParserElement) -> Box<dyn ElementImpl>;

#[derive(Default, Debug)]
pub struct StandardProps {
	pub width: Value,
	pub height: Value,
	pub x: Value,
	pub y: Value,
	pub background: Value,
}

pub trait ElementImpl: Debug + RenderWeb {}

#[derive(Debug)]
pub struct Empty;
impl ElementImpl for Empty {}

#[derive(Debug, Clone)]
pub struct Repeater {
	pub index: Option<String>,
	pub item: String,
	pub collection: Value,
}

#[derive(Debug)]
pub struct Element {
	pub tag: String,
	pub condition: Option<Value>,
	pub repeater: Option<Repeater>,
	pub standard_props: StandardProps,
	pub temporary_hacky_click_handler: Option<Value>,
	pub children: Vec<Element>,
	pub element_impl: Box<dyn ElementImpl>,
}

pub struct ElementData<'a> {
	pub tag: &'a String,
	pub condition: &'a Option<Value>,
	pub repeater: &'a Option<Repeater>,
	pub standard_props: &'a StandardProps,
	pub temporary_hacky_click_handler: &'a Option<Value>,
	pub children: &'a Vec<Element>,
}

impl Default for Element {
	fn default() -> Self {
		Element {
			element_impl: Box::new(Empty),
			..Default::default()
		}
	}
}

impl Element {
	pub fn construct(module: &Module, mut parse_tree: ParserElement) -> Result<Self, String> {
		let standard_props = Self::init_props(&mut parse_tree.properties);
		let constructor = module.lookup(&parse_tree.path)?;
		let element_impl = constructor(&module, &mut parse_tree);
		let children = build_dom(module, parse_tree.children);
		let repeater = parse_tree.repeater.map(|e| Repeater {
			index: e.index.map(|e| e.into()),
			item: e.item.into(),
			collection: e.collection,
		});
		Ok(Element {
			tag: parse_tree.path.join("."),
			condition: parse_tree.condition,
			repeater,
			standard_props,
			temporary_hacky_click_handler: parse_tree.event_handlers.remove("click"),
			children,
			element_impl,
		})
	}

	pub fn init_props(props: &mut HashMap<&str, Value>) -> StandardProps {
		let mut standard_props = StandardProps::default();
		if let Some(value) = props.remove("width") {
			standard_props.width = value;
		}
		if let Some(value) = props.remove("height") {
			standard_props.height = value;
		}
		if let Some(value) = props.remove("x") {
			standard_props.x = value;
		}
		if let Some(value) = props.remove("y") {
			standard_props.y = value;
		}
		if let Some(value) = props.remove("background") {
			standard_props.background = value;
		}

		standard_props
	}

	pub fn render_web(&self, ctx: &mut WebRenderer) -> Option<HtmlElement> {
		RenderWeb::render(self.element_impl.as_ref(), self.data(), ctx)
	}

	pub fn data(&self) -> ElementData {
		ElementData {
			tag: &self.tag,
			condition: &self.condition,
			repeater: &self.repeater,
			standard_props: &self.standard_props,
			temporary_hacky_click_handler: &self.temporary_hacky_click_handler,
			children: &self.children,
		}
	}
}

fn build_dom(module: &Module, parse_tree: Vec<ParserElement>) -> Vec<Element> {
	let mut elements = Vec::new();
	for item in parse_tree {
		match Element::construct(module, item) {
			Ok(element) => elements.push(element),
			Err(message) => eprintln!("Error: {}", message)
		}
	}
	elements
}

pub fn build_element(module: &Module, parse_tree: ParserElement) -> Element {
	match Element::construct(module, parse_tree) {
		Ok(element) => element,
		Err(message) => {
			eprintln!("Error: {}", message);
			Element::default()
		}
	}
}

// #[derive(Debug)]
// pub struct Window {
// 	pub standard_props: StandardProps,
// 	pub title: Value,
// 	pub children: Vec<Box<dyn Element>>,
// }

// impl ElementImpl for Window {}

// impl Window {
// 	pub fn construct(module: &Module, parse_tree: ParserElement) -> Box<dyn Element> {
// 		let children = build_dom(module, parse_tree.children);
// 		let (standard_props, title, _) = props(
// 			parse_tree.properties,
// 			Default::default(),
// 			|props| Window::window_props(props));
// 		Box::new(Window { children, standard_props, title })
// 	}

// 	fn window_props(props: &mut HashMap<&str, Value>) -> Value {
// 		props.remove("title").unwrap_or_default()
// 	}
// }

#[derive(Debug)]
pub struct Rect {}

impl ElementImpl for Rect {}

impl Rect {
	pub fn construct(_: &Module, _: &mut ParserElement) -> Box<dyn ElementImpl> {
		Box::new(Rect {})
	}
}

// #[derive(Debug)]
// pub struct PanesH {
// 	pub children: Vec<Box<dyn Element>>,
// }


// impl PanesH {
// 	pub fn construct(module: &Module, parse_tree: ParserElement) -> Box<dyn Element> {
// 		let children = build_dom(module, parse_tree.children);
// 		Box::new(PanesH { children })
// 	}
// }

// #[derive(Debug)]
// pub struct PanesV {
// 	pub children: Vec<Box<dyn Element>>,
// }


// impl PanesV {
// 	pub fn construct(module: &Module, parse_tree: ParserElement) -> Box<dyn Element> {
// 		let children = build_dom(module, parse_tree.children);
// 		Box::new(PanesV { children })
// 	}
// }


#[derive(Debug)]
pub struct Span {}

impl Span {
	pub fn construct(_: &Module, _: &mut ParserElement) -> Box<dyn ElementImpl> {
		Box::new(Span {})
	}
}

impl ElementImpl for Span {}

#[derive(Debug)]
pub struct Text {
	pub content: Value,
}

impl Text {
	pub fn construct(_module: &Module, parse_tree: &mut ParserElement) -> Box<dyn ElementImpl> {
		let content = parse_tree.properties
			.remove("content")
			.unwrap_or(Value::String("".to_owned()));
		Box::new(Text { content })
	}
}

impl ElementImpl for Text {}

// #[derive(Debug)]
// pub struct ChildPropertySetter {
// 	pub children: Vec<Box<dyn Element>>,
// }

// impl ChildPropertySetter {
// 	pub fn construct_i<'a>(module: &'a Module, parse_tree: ParserElement<'a>) -> Box<dyn Element> {
// 		let children = build_dom_with_transform(module, parse_tree.children, move |mut child| {
// 			let mut props = parse_tree.properties.clone();
// 			props.insert("i", Value::Boolean(true));
// 			for (k, v) in child.properties.iter() {
// 				props.insert(k, v.clone());
// 			}
// 			child.properties = props;
// 			Some(child)
// 		});
// 		Box::new(ChildPropertySetter { children })
// 	}

// 	pub fn construct_b(module: &Module, parse_tree: ParserElement) -> Box<dyn Element> {
// 		let children = build_dom_with_transform(module, parse_tree.children, move |mut child| {
// 			let mut props = parse_tree.properties.clone();
// 			props.insert("b", Value::Boolean(true));
// 			for (k, v) in child.properties.iter() {
// 				props.insert(k, v.clone());
// 			}
// 			child.properties = props;
// 			Some(child)
// 		});
// 		Box::new(ChildPropertySetter { children })
// 	}
// }


// #[derive(Debug)]
// pub struct Img {
// 	pub standard_props: StandardProps,
// 	pub src: Value,
// }

// #[derive(Default)]
// struct ImgProps {
// 	pub src: Value,
// }

// impl Img {
// 	pub fn construct(_module: &Module, parse_tree: ParserElement) -> Box<dyn Element> {
// 		let (standard_props, img_props, _) = props(
// 			parse_tree.properties,
// 			Default::default(),
// 			|props| Self::get_props(props));
		
// 		Box::new(Img { standard_props, src: img_props.src })
// 	}

// 	fn get_props(props: &mut HashMap<&str, Value>) -> ImgProps {
// 		let mut img_props = ImgProps::default();
// 		img_props.src = props.remove(&"src").unwrap_or_default();
// 		img_props
// 	}
// }



