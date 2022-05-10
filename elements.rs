use std::fmt::Debug;
use std::collections::HashMap;

use super::{
	web::{RenderWeb, WebRenderer, HtmlElement},
	parser::Element as ParserElement,
	parser::Content as ParserContent,
	parser::Component as ParserComponent,
	Module,
	LookupScope,
	Value,
	Expr,
	Type,
};

pub type Constructor = fn(&LookupScope, &ParserElement) -> (Box<dyn ElementImpl>, Vec<Element>);

// #[derive(Default, Debug)]
// pub struct StandardProps {
// 	pub width: Value,
// 	pub height: Value,
// 	pub x: Value,
// 	pub y: Value,
// 	pub background: Value,
// }

pub trait ElementImpl: Debug + RenderWeb {
	fn set_properties(&mut self, _properties: &HashMap<String, Value>) {}
	fn property_types(&self) -> HashMap<String, Type> {
		HashMap::new()
	}
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
	// pub standard_props: StandardProps,
	pub data_types: HashMap<String, Type>,
	pub temporary_hacky_click_handler: Option<Value>,
	pub children: Vec<Element>,
	pub element_impl: Box<dyn ElementImpl>,
}

pub struct ElementData<'a> {
	pub tag: &'a String,
	pub condition: &'a Option<Value>,
	pub repeater: &'a Option<Repeater>,
	// pub standard_props: &'a StandardProps,
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



fn update_data_type(map: &mut HashMap<String, Type>, path: &[String], t: &Type) {
	if path.len() > 1 {
		if let Some(Type::Object(next_map)) = map.get_mut(&path[0]) {
			update_data_type(next_map, &path[1..], t);
		} else {
			let mut next_map = HashMap::new();
			update_data_type(&mut next_map, &path[1..], t);
			map.insert(path[0].clone(), Type::Object(next_map));
		}
		return;
	}
	map.insert(path[0].clone(), t.clone());
}

impl Element {
	fn construct_component(scope: LookupScope, parse_tree: &ParserComponent) -> Result<Self, String> {
		let scope = LookupScope { module: scope.module, imports: Some(&parse_tree.imports_map), instance: None };
		Element::construct_element(scope, &parse_tree.parse_tree)
	}

	fn construct_element(scope: LookupScope, parse_tree: &ParserElement) -> Result<Self, String> {
		let (mut element_impl, children) = scope.construct(&parse_tree)?;
		element_impl.set_properties(&parse_tree.properties);
		let repeater = parse_tree.repeater.as_ref().map(|e| Repeater {
			index: e.index.as_ref().map(|e| e.into()),
			item: e.item.clone(),
			collection: e.collection.clone(),
		});

		let property_types = element_impl.property_types();
		let mut data_types = HashMap::new();
		for (k, v) in &parse_tree.properties {
			match v {
				Value::Binding(Expr::Path(path)) => {
					if let Some(t) = property_types.get(k.as_str()) {
						update_data_type(&mut data_types, path, t);
					}
				},
				_ => {},
			}
		}

		for child in &children {
			for (k, child_t) in &child.data_types {
				if let Some(this_t) = data_types.get(k) {
					if this_t != child_t {
						eprintln!("type error: {:?} does not match {:?}", this_t, child_t);
					}
				} else {
					data_types.insert(k.clone(), child_t.clone());
				}
			}
		}

		if let Some(repeater) = repeater.as_ref() {
			repeater.index.as_ref().map(|e| data_types.remove(e));
			if let Some(item_type) = data_types.remove(&repeater.item) {
				match &repeater.collection {
					Value::Binding(Expr::Path(path)) => {
						update_data_type(&mut data_types, path, &Type::Iter(Box::new(item_type)));
					},
					_ => {},
				}
			}
		}

		Ok(Element {
			tag: parse_tree.path.join("."),
			condition: parse_tree.condition.clone(),
			repeater,
			data_types,
			temporary_hacky_click_handler: parse_tree.event_handlers.get("click").map(|e| e.clone()),
			children,
			element_impl,
		})
	}

	fn set_properties(&mut self, parse_tree: &ParserElement) {
		self.element_impl.set_properties(&parse_tree.properties);
	}

	// pub fn merge_props(mut props: StandardProps, new_props: &HashMap<String, Value>) -> StandardProps {
	// 	if let Some(value) = new_props.get("width") {
	// 		props.width = value.clone();
	// 	}
	// 	if let Some(value) = new_props.get("height") {
	// 		props.height = value.clone();
	// 	}
	// 	if let Some(value) = new_props.get("x") {
	// 		props.x = value.clone();
	// 	}
	// 	if let Some(value) = new_props.get("y") {
	// 		props.y = value.clone();
	// 	}
	// 	if let Some(value) = new_props.get("background") {
	// 		props.background = value.clone();
	// 	}
	// 	props
	// }

	// pub fn init_props(props: &HashMap<String, Value>) -> StandardProps {
	// 	Self::merge_props(StandardProps::default(), props)
	// }

	pub fn render_web(&self, ctx: &mut WebRenderer) -> Option<HtmlElement> {
		RenderWeb::render(self.element_impl.as_ref(), self.data(), ctx)
	}

	pub fn data(&self) -> ElementData {
		ElementData {
			tag: &self.tag,
			condition: &self.condition,
			repeater: &self.repeater,
			// standard_props: &self.standard_props,
			temporary_hacky_click_handler: &self.temporary_hacky_click_handler,
			children: &self.children,
		}
	}
}

fn build_elements(scope: &LookupScope, parse_tree: &[ParserContent]) -> Vec<Element> {
	let mut elements = Vec::new();
	for item in parse_tree {
		match item {
			ParserContent::Element(e) => {
				match Element::construct_element(scope.clone(), e) {
					Ok(element) => elements.push(element),
					Err(message) => eprintln!("Error: {}", message)
				}
			},
			ParserContent::Children(c) => {
				if let Some((scope, instance)) = scope.instance {
					let children_elements = build_elements(scope, &instance.children);
						// .into_iter()
						// .for_each(|e| elements.push(e));
					let mut count = 0;
					let limit = if c.single { 1 } else { i32::MAX };
					for e in children_elements {
						if count >= limit {
							break;
						}
						elements.push(e);
						count += 1;
					}
				}
			},
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
	let scope = LookupScope { module, imports: Some(&parse_tree.imports_map), instance: None };
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
pub struct Rect {
	pub width: Value,
	pub height: Value,
	pub x: Value,
	pub y: Value,
	pub background: Value,
}

impl ElementImpl for Rect {
	fn property_types(&self) -> HashMap<String, Type> {
		hashmap![
			"width".into() => Type::Length,
			"height".into() => Type::Length,
			"x".into() => Type::Length,
			"y".into() => Type::Length,
			"background".into() => Type::Brush,
		]
	}

	fn set_properties(&mut self, properties: &HashMap<String, Value>) {
		if let Some(width) = properties.get("width") {
			self.width = width.clone();
		}
		if let Some(height) = properties.get("height") {
			self.height = height.clone();
		}
		if let Some(x) = properties.get("x") {
			self.x = x.clone();
		}
		if let Some(y) = properties.get("y") {
			self.y = y.clone();
		}
		if let Some(background) = properties.get("background") {
			self.background = background.clone();
		}
	}
}

impl Rect {
	pub fn construct(scope: &LookupScope, parse_tree: &ParserElement) -> (Box<dyn ElementImpl>, Vec<Element>) {
		let data = Rect {
			width: Value::Px(0),
			height: Value::Px(0),
			x: Value::Px(0),
			y: Value::Px(0),
			background: Value::Color(0,0,0),
		};
		(Box::new(data), build_elements(scope, &parse_tree.children))
	}
}

#[derive(Debug)]
pub struct Span {
	pub color: Value,
}

impl Span {
	pub fn construct(scope: &LookupScope, parse_tree: &ParserElement) -> (Box<dyn ElementImpl>, Vec<Element>) {
		(Box::new(Span { color: Value::Color(0,0,0) }), build_elements(scope, &parse_tree.children))
	}
}

impl ElementImpl for Span {
	fn property_types(&self) -> HashMap<String, Type> {
		hashmap!["color".into() => Type::Brush]
	}

	fn set_properties(&mut self, properties: &HashMap<String, Value>) {
		if let Some(color) = properties.get("color") {
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
		(Box::new(Text { content: Value::String("".to_owned()) }), build_elements(scope, &parse_tree.children))
	}
}

impl ElementImpl for Text {
	fn property_types(&self) -> HashMap<String, Type> {
		hashmap!["content".into() => Type::String]
	}

	fn set_properties(&mut self, properties: &HashMap<String, Value>) {
		if let Some(content) = properties.get("content") {
			self.content = content.clone()
		}
	}
}

#[derive(Debug)]
pub struct Component {}

impl Component {
	pub fn construct(scope: &LookupScope, parse_tree: &ParserElement) -> (Box<dyn ElementImpl>, Vec<Element>) {
		let mut children = build_element(scope, parse_tree).map(|e| vec![e]).unwrap_or(Vec::new());
		let element = &mut children[0];
		element.set_properties(scope.instance.unwrap().1);
		(Box::new(Component {}), children)
	}
}

impl ElementImpl for Component {}

#[derive(Debug)]
pub struct Layout {
	pub width: Value,
	pub height: Value,
	pub x: Value,
	pub y: Value,
}

impl Layout {
	pub fn construct(scope: &LookupScope, parse_tree: &ParserElement) -> (Box<dyn ElementImpl>, Vec<Element>) {
		let data = Layout {
			x: Value::Px(0),
			y: Value::Px(0),
			width: Value::Px(0),
			height: Value::Px(0),
		};
		(Box::new(data), build_elements(scope, &parse_tree.children))
	}
}

impl ElementImpl for Layout {
	fn property_types(&self) -> HashMap<String, Type> {
		hashmap![
			"x".into() => Type::Length,
			"y".into() => Type::Length,
			"width".into() => Type::Length,
			"height".into() => Type::Length,
		]
	}

	fn set_properties(&mut self, properties: &HashMap<String, Value>) {
		if let Some(x) = properties.get("x") {
			self.x = x.clone();
		}
		if let Some(y) = properties.get("y") {
			self.y = y.clone();
		}
		if let Some(width) = properties.get("width") {
			self.width = width.clone();
		}
		if let Some(height) = properties.get("height") {
			self.height = height.clone();
		}
	}
}


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



