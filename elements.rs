use std::fmt::Debug;
use std::collections::HashMap;

use super::{
	web::{RenderWeb, WebRenderer, HtmlContent},
	parser::Element as ParserElement,
	parser::Content as ParserContent,
	parser::Component as ParserComponent,
	Module,
	Value,
	Alignment,
	Expr,
	Type,
	Ctx,
};

pub use super::parser::Children;

#[derive(Debug, Clone)]
pub enum AddedProperties {
	Layout(LayoutItem),
	None,
}

impl AddedProperties {
	fn set_property(&mut self, k: &String, v: &Value) -> SetPropertyResult {
		match self {
			AddedProperties::Layout(item) => item.set_property(k, v),
			AddedProperties::None => SetPropertyResult::Ignore,
		}
	}
}

pub struct ConstructedElementImpl {
	element_impl: Box<dyn ElementImpl>,
	children: Vec<Content>,
}

impl ConstructedElementImpl {
	fn new(
		element_impl: Box<dyn ElementImpl>,
		children: Vec<Content>
	) -> Self {
		ConstructedElementImpl {
			element_impl,
			children,
		}
	}
}

pub type Constructor = fn(&Module, &ParserElement) -> ConstructedElementImpl;

pub trait ElementImpl: Debug + RenderWeb {
	fn set_property(&mut self, _k: &String, _v: &Value) -> SetPropertyResult { SetPropertyResult::Ignore }
	fn property_types(&self) -> HashMap<String, Type> { HashMap::new() }
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
pub enum Content {
	Element(Element),
	Children(Children)
}

#[derive(Debug)]
pub struct Element {
	pub tag: String,
	pub condition: Option<Value>,
	pub repeater: Option<Repeater>,
	pub data_types: HashMap<String, Type>,
	pub temporary_hacky_click_handler: Option<Value>,
	pub children: Vec<Content>,
	pub element_impl: Box<dyn ElementImpl>,
	pub added_properties: AddedProperties,
}

#[derive(Debug)]
pub struct Component {
	pub element: Element,
	pub name: String,
}

pub struct ElementData<'a> {
	pub tag: &'a String,
	pub condition: &'a Option<Value>,
	pub repeater: &'a Option<Repeater>,
	pub temporary_hacky_click_handler: &'a Option<Value>,
	pub children: &'a Vec<Content>,
	pub added_properties: &'a AddedProperties,
}

impl Default for Element {
	fn default() -> Self {
		Element {
			tag: String::from("<empty>"),
			condition: None,
			repeater: None,
			data_types: HashMap::new(),
			temporary_hacky_click_handler: None,
			children: Vec::new(),
			element_impl: Box::new(Empty),
			added_properties: AddedProperties::None,
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

pub enum SetPropertyResult {
	Set,
	Ignore,
	TypeError,
}

fn set_properties(
	properties: &HashMap<String, Value>,
	element_impl: &mut Box<dyn ElementImpl>,
	added_properties: &mut AddedProperties,
) {
	for (k, v) in properties {
		if k == "self" {
			match v {
				Value::Object(map) => {
					set_properties(map, element_impl, &mut AddedProperties::None);
					continue;
				},
				_ => {
					eprintln!("tried to set `self` as a property");
					continue;
				},
			}
		}
		match added_properties.set_property(k, v) {
			SetPropertyResult::Set       => { continue },
			SetPropertyResult::Ignore    => {},
			SetPropertyResult::TypeError => {
				eprintln!("type error when setting property `{k}`");
				continue;
			},
		}
		match element_impl.set_property(k, v) {
			SetPropertyResult::Set       => {},
			SetPropertyResult::Ignore    => {
				eprintln!("tried to set nonexistent property `{k}`");
			},
			SetPropertyResult::TypeError => {
				eprintln!("type error when setting property `{k}`");
			},
		}
	}
}

fn try_type_merge(t1: &Type, t2: &Type) -> Result<Type, ()> {
	if *t1 == *t2 {
		return Ok(t1.clone());
	}

	let (t1, t2) = match (t1, t2) {
		(Type::String, t) => { return Ok(t.clone()) },
		(t, Type::String) => { return Ok(t.clone()) },
		(Type::Object(t1), Type::Object(t2)) => (t1, t2),
		_ => { return Err(()) }
	};

	let mut new_t = t1.clone();
	for (k, v2) in t2 {
		let t = if let Some(v1) = new_t.get(k) {
			try_type_merge(v1, v2)?
		} else {
			v2.clone()
		};
		new_t.insert(k.clone(), t);
	}

	Ok(Type::Object(new_t))
}

impl Element {
	fn construct_element(
		scope: &Module,
		parse_tree: &ParserElement,
		mut added_properties: AddedProperties,
	) -> Result<Self, String> {
		let ConstructedElementImpl {
			mut element_impl,
			children,
		} = scope.construct(&parse_tree)?;

		// println!("{:#?}", parse_tree.properties);
		set_properties(&parse_tree.properties, &mut element_impl, &mut added_properties);


		let repeater = parse_tree.repeater.as_ref().map(|e| Repeater {
			index: e.index.as_ref().map(|e| e.into()),
			item: e.item.clone(),
			collection: e.collection.clone(),
		});
		let condition = parse_tree.condition.clone();
		let data = parse_tree.data.clone();

		let property_types = element_impl.property_types();
		let mut data_types = HashMap::new();//element_impl.base_data_types();
		for (k, v) in &parse_tree.properties {
			match v {
				Value::Binding(Expr::Path(path, ctx)) => {
					if *ctx != Ctx::Component {
						continue;
					} else if let Some(t) = property_types.get(k.as_str()) {
						update_data_type(&mut data_types, path, t);
					}
				},
				_ => {},
			}
		}

		for child in &children {
			let child = match child { Content::Element(e) => e, _ => continue };
			for (k, child_t) in &child.data_types {
				if let Some(this_t) = data_types.get(k) {
					if this_t != child_t {
						match try_type_merge(this_t, child_t) {
							Ok(t) => {
								data_types.insert(k.clone(), t);
							}
							Err(_) => {
								eprintln!("type error: {:?} does not match {:?}", this_t, child_t);
							}
						}
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
					Value::Binding(Expr::Path(path, ..)) => {
						update_data_type(&mut data_types, path, &Type::Iter(Box::new(item_type)));
					},
					_ => {
						unimplemented!();
					},
				}
			}
		}

		if let Some(condition) = condition.as_ref() {
			match condition {
				Value::Binding(Expr::Path(path, ..)) => {
					update_data_type(&mut data_types, path, &Type::Boolean);
				},
				_ => {
					unimplemented!();
				},
			}
		}

		if let Some(data) = data {
			match data {
				Value::Binding(Expr::Path(path, ..)) => {
					let mut new_data_types = HashMap::new();
					update_data_type(&mut new_data_types, &path, &Type::Object(data_types));
					data_types = new_data_types;
				},
				_ => {
					unimplemented!();
				},
			}
		}

		Ok(Element {
			tag: parse_tree.path.join("."),
			condition,
			repeater,
			data_types,
			temporary_hacky_click_handler: parse_tree.event_handlers.get("click").map(|e| e.clone()),
			children,
			element_impl,
			added_properties,
		})
	}

	pub fn render_web(&self, ctx: &mut WebRenderer) -> Option<HtmlContent> {
		RenderWeb::render(self.element_impl.as_ref(), self.data(), ctx)
	}

	pub fn data(&self) -> ElementData {
		ElementData {
			tag: &self.tag,
			condition: &self.condition,
			repeater: &self.repeater,
			temporary_hacky_click_handler: &self.temporary_hacky_click_handler,
			children: &self.children,
			added_properties: &self.added_properties,
		}
	}
}

fn build_elements(scope: &Module, parse_tree: &[ParserContent]) -> Vec<Content> {
	build_elements_with_added_properties(scope, parse_tree, AddedProperties::None)
}

fn build_elements_with_added_properties(
	scope: &Module,
	parse_tree: &[ParserContent],
	added_properties: AddedProperties
) -> Vec<Content> {
	let mut elements = Vec::new();
	for item in parse_tree {
		match item {
			ParserContent::Element(e) => {
				match Element::construct_element(scope, e, added_properties.clone()) {
					Ok(element) => elements.push(Content::Element(element)),
					Err(message) => eprintln!("Error: {}", message)
				}
			},
			ParserContent::Children(c) => {
				// if let Some((scope, instance)) = scope.instance {
				// 	let children_elements = build_elements(scope, &instance.children);
				// 	// println!("{:#?}", children_elements);
				// 	let mut count = 0;
				// 	let limit = if c.single { 1 } else { i32::MAX };
				// 	for e in children_elements {
				// 		if count >= limit {
				// 			break;
				// 		}
				// 		elements.push(e);
				// 		count += 1;
				// 	}
				// }
				elements.push(Content::Children(c.clone()))
			},
		}
	}
	elements
}

pub fn build_component(scope: &Module, parse_tree: &ParserComponent) -> Component {
	match Element::construct_element(scope, &parse_tree.parse_tree, AddedProperties::None) {
		Ok(element) => Component { element, name: parse_tree.name.clone() },
		Err(message) => {
			eprintln!("Error: {}", message);
			Component { element: Element::default(), name: parse_tree.name.clone() }
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
	pub clip: Value,
	pub width: Value,
	pub height: Value,
	pub x: Value,
	pub y: Value,
	pub background: Value,
	pub border_width: Value,
	pub border_color: Value,
}

impl ElementImpl for Rect {
	fn property_types(&self) -> HashMap<String, Type> {
		hashmap![
			"clip".into() => Type::Boolean,
			"width".into() => Type::Length,
			"height".into() => Type::Length,
			"x".into() => Type::Length,
			"y".into() => Type::Length,
			"background".into() => Type::Brush,
			"border".into() => Type::Object(hashmap![
				"width".into() => Type::Length,
				"color".into() => Type::Brush,
			]),
		]
	}

	fn set_property(&mut self, k: &String, v: &Value) -> SetPropertyResult {
		match k.as_str() {
			"clip" => { self.clip = v.clone() },
			"width" => { self.width = v.clone() },
			"height" => { self.height = v.clone() },
			"x" => { self.x = v.clone() },
			"y" => { self.y = v.clone() },
			"border" => {
				match v {
					Value::Object(map) => {
						if let Some(width) = map.get(&"width".to_owned()) {
							self.border_width = width.clone();
						}
						if let Some(color) = map.get(&"color".to_owned()) {
							self.border_color = color.clone();
						}
					},
					_ => { return SetPropertyResult::TypeError }
				}
			},
			"background" => { self.background = v.clone() },
			_ => { return SetPropertyResult::Ignore },
		}
		SetPropertyResult::Set
	}
}

impl Rect {
	pub fn construct(scope: &Module, parse_tree: &ParserElement) -> ConstructedElementImpl {
		let data = Rect {
			clip: Value::Boolean(true),
			width: Value::Px(0.0),
			height: Value::Px(0.0),
			x: Value::Px(0.0),
			y: Value::Px(0.0),
			border_width: Value::Px(0.0),
			border_color: Value::Color(0,0,0,0.0),
			background: Value::Color(0,0,0,0.0),
		};
		ConstructedElementImpl::new(Box::new(data), build_elements(scope, &parse_tree.children))
	}
}

#[derive(Debug)]
pub struct Scroll {
	pub width: Value,
	pub height: Value,
	pub x: Value,
	pub y: Value,
}

impl Scroll {
	pub fn construct(scope: &Module, parse_tree: &ParserElement) -> ConstructedElementImpl {
		ConstructedElementImpl::new(
			Box::new(
				Scroll {
					width: Value::Px(0.0),
					height: Value::Px(0.0),
					x: Value::Px(0.0),
					y: Value::Px(0.0),
				}
			),
			build_elements(scope, &parse_tree.children),
		)
	}
}

impl ElementImpl for Scroll {
	fn property_types(&self) -> HashMap<String, Type> {
		hashmap![
			"width".into() => Type::Length,
			"height".into() => Type::Length,
			"x".into() => Type::Length,
			"y".into() => Type::Length,
		]
	}

	fn set_property(&mut self, k: &String, v: &Value) -> SetPropertyResult {
		match k.as_str() {
			"width" => { self.width = v.clone() },
			"height" => { self.height = v.clone() },
			"x" => { self.x = v.clone() },
			"y" => { self.y = v.clone() },
			_ => { return SetPropertyResult::Ignore },
		}
		SetPropertyResult::Set
	}
}

#[derive(Debug)]
pub struct Span {
	pub color: Value,
	pub max_width: Value,
	pub padding: Value,
}

impl Span {
	pub fn construct(scope: &Module, parse_tree: &ParserElement) -> ConstructedElementImpl {
		ConstructedElementImpl::new(
			Box::new(
				Span {
					color: Value::Color(0,0,0,1.0),
					max_width: Value::Unset,
					padding: Value::Px(0.0),
				}
			),
			build_elements(scope, &parse_tree.children),
		)
	}
}

impl ElementImpl for Span {
	fn property_types(&self) -> HashMap<String, Type> {
		hashmap![
			"color".into() => Type::Brush,
			"max_width".into() => Type::Length,
			"padding".into() => Type::Length,
		]
	}

	fn set_property(&mut self, k: &String, v: &Value) -> SetPropertyResult {
		match k.as_str() {
			"color" => { self.color = v.clone() }
			"max_width" => { self.max_width = v.clone() }
			"padding" => { self.padding = v.clone() }
			_ => { return SetPropertyResult::Ignore }
		}
		SetPropertyResult::Set
	}
}

#[derive(Debug)]
pub struct Text {
	pub content: Value,
}

impl Text {
	pub fn construct(scope: &Module, parse_tree: &ParserElement) -> ConstructedElementImpl {
		ConstructedElementImpl::new(
			Box::new(Text { content: Value::String("".to_owned()) }),
			build_elements(scope, &parse_tree.children)
		)
	}
}

impl ElementImpl for Text {
	fn property_types(&self) -> HashMap<String, Type> {
		hashmap!["content".into() => Type::String]
	}

	fn set_property(&mut self, k: &String, v: &Value) -> SetPropertyResult {
		match k.as_str() {
			"content" => { self.content = v.clone() }
			_ => { return SetPropertyResult::Ignore }
		}
		SetPropertyResult::Set
	}
}

#[derive(Debug)]
pub struct ComponentInstance {
	pub name: String,
	pub data_types: HashMap<String, Type>,
	pub properties: HashMap<String, Value>,
}

impl ComponentInstance {
	pub fn construct(
		scope: &Module,
		component: &Component,
		parse_tree: &ParserElement,
	) -> ConstructedElementImpl {
		let data = ComponentInstance {
			name: component.name.clone(),
			data_types: component.element.data_types.clone(),
			properties: HashMap::new(),
		};
		ConstructedElementImpl::new(Box::new(data), build_elements(scope, &parse_tree.children))
	}
}

impl ElementImpl for ComponentInstance {
	fn set_property(&mut self, k: &String, v: &Value) -> SetPropertyResult {
		if self.data_types.contains_key(k) {
			self.properties.insert(k.clone(), v.clone());
			return SetPropertyResult::Set
		}
		SetPropertyResult::Ignore
	}

	fn property_types(&self) -> HashMap<String, Type> {
		self.data_types.clone()
	}
}

#[derive(Debug, Clone)]
pub struct LayoutItem {
	pub align: Value,
	pub stretch: Value,
	pub column: bool,
	pub grow: bool,
}

impl LayoutItem {
	fn set_property(&mut self, k: &String, v: &Value) -> SetPropertyResult {
		match k.as_str() {
			"align" => { self.align = v.clone(); SetPropertyResult::Set },
			"stretch" if !self.grow => { self.stretch = v.clone(); SetPropertyResult::Set },
			_ => { SetPropertyResult::Ignore }
		}
	}
}

impl LayoutItem {
	fn new(column: bool, grow: bool) -> LayoutItem {
		LayoutItem {
			align: Value::Alignment(Alignment::Stretch),
			stretch: if grow { Value::Unset } else { Value::Float(1.0) },
			column,
			grow,

		}
	}
}

#[derive(Debug)]
pub struct Layout {
	pub width: Value,
	pub height: Value,
	pub x: Value,
	pub y: Value,
	pub padding: Value,
	pub column: bool,
	pub grow: bool,
}

impl Layout {
	pub fn row_stretch(scope: &Module, parse_tree: &ParserElement) -> ConstructedElementImpl {
		Self::construct(scope, parse_tree, false, false)
	}
	pub fn row_grow(scope: &Module, parse_tree: &ParserElement) -> ConstructedElementImpl {
		Self::construct(scope, parse_tree, false, true)
	}
	pub fn column_stretch(scope: &Module, parse_tree: &ParserElement) -> ConstructedElementImpl {
		Self::construct(scope, parse_tree, true, false)
	}
	pub fn column_grow(scope: &Module, parse_tree: &ParserElement) -> ConstructedElementImpl {
		Self::construct(scope, parse_tree, true, true)
	}
	pub fn construct(scope: &Module, parse_tree: &ParserElement, column: bool, grow: bool) -> ConstructedElementImpl {
		let (width, height) = match (column, grow) {
			(false, false) => {
				(Value::Px(0.0), Value::Unset)
			},
			(true, false) => {
				(Value::Unset, Value::Px(0.0))
			},
			_ => {
				(Value::Unset, Value::Unset)
			}
		};
		let data = Layout {
			x: Value::Px(0.0),
			y: Value::Px(0.0),
			padding: Value::Px(0.0),
			width,
			height,
			column,
			grow,
		};
		ConstructedElementImpl::new(
			Box::new(data),
			build_elements_with_added_properties(
				scope,
				&parse_tree.children,
				AddedProperties::Layout(LayoutItem::new(column, grow)),
			)
		)
	}
}

impl ElementImpl for Layout {
	fn property_types(&self) -> HashMap<String, Type> {
		match (self.column, self.grow) {
			(_, false) => {
				hashmap![
					"x".into() => Type::Length,
					"y".into() => Type::Length,
					"width".into() => Type::Length,
					"height".into() => Type::Length,
					"padding".into() => Type::Length,
				]
			},
			(false, true) => {
				hashmap![
					"x".into() => Type::Length,
					"y".into() => Type::Length,
					"height".into() => Type::Length,
					"padding".into() => Type::Length,
				]
			},
			(true, true) => {
				hashmap![
					"x".into() => Type::Length,
					"y".into() => Type::Length,
					"width".into() => Type::Length,
					"padding".into() => Type::Length,
				]
			}
		}
	}

	fn set_property(&mut self, k: &String, v: &Value) -> SetPropertyResult {
		match (k.as_str(), self.column, self.grow) {
			("width", _, false) => { self.width = v.clone() }
			("height", _, false) => { self.height = v.clone() }
			("width", true, true) => { self.width = v.clone() }
			("height", false, true) => { self.height = v.clone() }
			("x", _, _) => { self.x = v.clone() }
			("y", _, _) => { self.y = v.clone() }
			("padding", _, _) => { self.padding = v.clone() }
			_ => { return SetPropertyResult::Ignore }
		}
		SetPropertyResult::Set
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



