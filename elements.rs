use std::fmt::Debug;
use std::collections::HashMap;

use super::{
	web::{RenderWeb, WebRenderer, HtmlElement},
	parser::Element as ParserElement,
	Module,
	LookupScope,
	Value,
	Component,
};

pub type Constructor = fn(&LookupScope, &ParserElement) -> (Box<dyn ElementImpl>, Vec<Element>);

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
	fn construct(scope: LookupScope, c: ParserElementOrComponent) -> Result<Self, String> {
		let (parse_tree, scope) = match c {
			ParserElementOrComponent::Component(c) => {
				(&c.parse_tree, LookupScope { module: scope.module, imports: Some(&c.imports_map) })
			}
			ParserElementOrComponent::Element(e) => {
				(e, scope)
			}
		};
		let standard_props = Self::init_props(&parse_tree.properties);
		let (element_impl, children) = scope.construct(&parse_tree)?;
		// let element_impl = constructor(&scope, &parse_tree);
		// let children = build_dom(scope, &parse_tree.children);
		let repeater = parse_tree.repeater.as_ref().map(|e| Repeater {
			index: e.index.as_ref().map(|e| e.into()),
			item: e.item.clone(),
			collection: e.collection.clone(),
		});
		Ok(Element {
			tag: parse_tree.path.join("."),
			condition: parse_tree.condition.clone(),
			repeater,
			standard_props,
			temporary_hacky_click_handler: parse_tree.event_handlers.get("click").map(|e| e.clone()),
			children,
			element_impl,
		})
	}

	pub fn init_props(props: &HashMap<String, Value>) -> StandardProps {
		let mut standard_props = StandardProps::default();
		if let Some(value) = props.get("width") {
			standard_props.width = value.clone();
		}
		if let Some(value) = props.get("height") {
			standard_props.height = value.clone();
		}
		if let Some(value) = props.get("x") {
			standard_props.x = value.clone();
		}
		if let Some(value) = props.get("y") {
			standard_props.y = value.clone();
		}
		if let Some(value) = props.get("background") {
			standard_props.background = value.clone();
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

enum ParserElementOrComponent<'a> {
	Element(&'a ParserElement),
	Component(&'a Component),
}

fn build_dom(scope: &LookupScope, parse_tree: &Vec<ParserElement>) -> Vec<Element> {
	let mut elements = Vec::new();
	for item in parse_tree {
		match Element::construct(scope.clone(), ParserElementOrComponent::Element(item)) {
			Ok(element) => elements.push(element),
			Err(message) => eprintln!("Error: {}", message)
		}
	}
	elements
}

fn build_element(scope: &LookupScope, parse_tree: &ParserElement) -> Option<Element> {
	match Element::construct(scope.clone(), ParserElementOrComponent::Element(parse_tree)) {
		Ok(element) => Some(element),
		Err(message) => { eprintln!("Error: {}", message); None }
	}
}

pub fn build_component(module: &Module, component: &Component) -> Element {
	let scope = LookupScope { module, imports: Some(&component.imports_map) };
	match Element::construct(scope, ParserElementOrComponent::Component(component)) {
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
	pub fn construct(scope: &LookupScope, parse_tree: &ParserElement) -> (Box<dyn ElementImpl>, Vec<Element>) {
		(Box::new(Rect {}), build_dom(scope, &parse_tree.children))
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
	pub fn construct(scope: &LookupScope, parse_tree: &ParserElement) -> (Box<dyn ElementImpl>, Vec<Element>) {
		(Box::new(Span {}), build_dom(scope, &parse_tree.children))
	}
}

impl ElementImpl for Span {}

#[derive(Debug)]
pub struct Text {
	pub content: Value,
}

impl Text {
	pub fn construct(scope: &LookupScope, parse_tree: &ParserElement) -> (Box<dyn ElementImpl>, Vec<Element>) {
		let content = parse_tree.properties
			.get("content")
			.map(|e| e.clone())
			.unwrap_or(Value::String("".to_owned()));
		(Box::new(Text { content }), build_dom(scope, &parse_tree.children))
	}
}

impl ElementImpl for Text {}

#[derive(Debug)]
pub struct Component2 {}

impl Component2 {
	pub fn construct(scope: &LookupScope, parse_tree: &ParserElement) -> (Box<dyn ElementImpl>, Vec<Element>) {
		let children = build_element(scope, parse_tree).map(|e| vec![e]).unwrap_or(Vec::new());
		(Box::new(Component2 {}), children)
	}
}

impl ElementImpl for Component2 {}

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



