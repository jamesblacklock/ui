use std::fmt::Debug;
use std::collections::HashMap;

use super::{
	web::{RenderWeb, WebRenderer, HtmlElement},
	parser::Element as ParserElement,
	parser::Component as ParserComponent,
	Module,
	LookupScope,
	Value,
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

pub trait ElementImpl: Debug + RenderWeb {
	fn inherit_properties(&mut self, _parse_tree: &ParserElement) {}
}

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
	fn construct_component(scope: LookupScope, parse_tree: &ParserComponent) -> Result<Self, String> {
		let scope = LookupScope { module: scope.module, imports: Some(&parse_tree.imports_map) };
		Element::construct_element(scope, &parse_tree.parse_tree)
	}

	fn construct_element(scope: LookupScope, parse_tree: &ParserElement) -> Result<Self, String> {
		let standard_props = Self::init_props(&parse_tree.properties);
		let (element_impl, children) = scope.construct(&parse_tree)?;
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

	fn inherit_properties(&mut self, parse_tree: &ParserElement) {
		self.standard_props = Self::merge_props(
			std::mem::replace(&mut self.standard_props, Default::default()),
			&parse_tree.properties);
		self.element_impl.inherit_properties(parse_tree);
	}

	pub fn merge_props(mut props: StandardProps, new_props: &HashMap<String, Value>) -> StandardProps {
		if let Some(value) = new_props.get("width") {
			props.width = value.clone();
		}
		if let Some(value) = new_props.get("height") {
			props.height = value.clone();
		}
		if let Some(value) = new_props.get("x") {
			props.x = value.clone();
		}
		if let Some(value) = new_props.get("y") {
			props.y = value.clone();
		}
		if let Some(value) = new_props.get("background") {
			props.background = value.clone();
		}
		props
	}

	pub fn init_props(props: &HashMap<String, Value>) -> StandardProps {
		Self::merge_props(StandardProps::default(), props)
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

fn build_elements(scope: &LookupScope, parse_tree: &Vec<ParserElement>) -> Vec<Element> {
	let mut elements = Vec::new();
	for item in parse_tree {
		match Element::construct_element(scope.clone(), item) {
			Ok(element) => elements.push(element),
			Err(message) => eprintln!("Error: {}", message)
		}
	}
	elements
}

fn build_element(scope: &LookupScope, parse_tree: &ParserElement) -> Option<Element> {
	match Element::construct_element(scope.clone(), parse_tree) {
		Ok(element) => Some(element),
		Err(message) => { eprintln!("Error: {}", message); None }
	}
}

pub fn build_component(module: &Module, parse_tree: &ParserComponent) -> Element {
	let scope = LookupScope { module, imports: Some(&parse_tree.imports_map) };
	match Element::construct_component(scope, parse_tree) {
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
// 		let children = build_elements(module, parse_tree.children);
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
		(Box::new(Rect {}), build_elements(scope, &parse_tree.children))
	}
}

#[derive(Debug)]
pub struct Span {
	pub color: Value,
}

impl Span {
	pub fn construct(scope: &LookupScope, parse_tree: &ParserElement) -> (Box<dyn ElementImpl>, Vec<Element>) {
		let color = if let Some(color) = parse_tree.properties.get("color") {
			color.clone()
		} else {
			Value::Color(0,0,0)
		};
		(Box::new(Span { color }), build_elements(scope, &parse_tree.children))
	}
}

impl ElementImpl for Span {
	fn inherit_properties(&mut self, parse_tree: &ParserElement) {
		if let Some(color) = parse_tree.properties.get("color") {
			self.color = color.clone()
		}
	}
}

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
		(Box::new(Text { content }), build_elements(scope, &parse_tree.children))
	}
}

impl ElementImpl for Text {}

#[derive(Debug)]
pub struct Component {}

impl Component {
	pub fn construct(
		scope: &LookupScope,
		component_parse_tree: &ParserElement,
		instance_parse_tree: &ParserElement,
	) -> (Box<dyn ElementImpl>, Vec<Element>) {
		let mut children = build_element(scope, component_parse_tree).map(|e| vec![e]).unwrap_or(Vec::new());
		let element = &mut children[0];
		element.inherit_properties(instance_parse_tree);
		(Box::new(Component {}), children)
	}
}

impl ElementImpl for Component {}

#[derive(Debug)]
pub struct Layout {
	
}

impl Layout {
	pub fn construct(scope: &LookupScope, parse_tree: &ParserElement) -> (Box<dyn ElementImpl>, Vec<Element>) {
		(Box::new(Layout {}), build_elements(scope, &parse_tree.children))
	}
}

impl ElementImpl for Layout {}


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



