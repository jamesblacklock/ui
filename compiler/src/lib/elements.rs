use std::fmt::Debug;
use std::collections::HashMap;
use maplit::hashmap;

use super::{
	codegen::{CodeGen},
	parser::Element as ParserElement,
	parser::Content as ParserContent,
	parser::Component as ParserComponent,
	Module,
	Value,
	Alignment,
	Expr,
	Type,
	Ctx,
	PropDecl,
};

pub use super::parser::Children;


#[derive(Debug, Default, Clone)]
pub struct EventsSpec {
	pub pointer_click: bool,
	pub pointer_press: bool,
	pub pointer_release: bool,
	pub pointer_move: bool,
	pub pointer_in: bool,
	pub pointer_out: bool,
}

#[derive(Debug, Default, Clone)]
pub struct Events {
	pub pointer_click: Value,
	pub pointer_press: Value,
	pub pointer_release: Value,
	pub pointer_move: Value,
	pub pointer_in: Value,
	pub pointer_out: Value,
}

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
	events_spec: EventsSpec,
}

impl ConstructedElementImpl {
	fn new(
		element_impl: Box<dyn ElementImpl>,
		children: Vec<Content>,
		events_spec: EventsSpec,
	) -> Self {
		ConstructedElementImpl {
			element_impl,
			children,
			events_spec,
		}
	}
}

pub type Constructor = fn(&mut Module, &ParserElement) -> ConstructedElementImpl;

pub trait ElementImpl: Debug + CodeGen {
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
	pub children: Vec<Content>,
	pub element_impl: Box<dyn ElementImpl>,
	pub added_properties: AddedProperties,
	pub events: Events,
}

#[derive(Debug)]
pub struct Component {
	pub root: Element,
	pub props: HashMap<String, PropDecl>,
	pub name: String,
}

pub struct ElementData<'a> {
	pub tag: &'a String,
	pub condition: &'a Option<Value>,
	pub repeater: &'a Option<Repeater>,
	pub children: &'a Vec<Content>,
	pub added_properties: &'a AddedProperties,
	pub events: &'a Events,
}

impl Default for Element {
	fn default() -> Self {
		Element {
			tag: String::from("<empty>"),
			condition: None,
			repeater: None,
			// data_types: HashMap::new(),
			children: Vec::new(),
			element_impl: Box::new(Empty),
			added_properties: AddedProperties::None,
			events: Default::default(),
		}
	}
}

pub enum SetPropertyResult {
	Set,
	Ignore,
	TypeError,
}

fn set_pointer_events_properties(
	properties: &HashMap<String, Value>,
	events_spec: &EventsSpec,
	events: &mut Events
) {
	for (k, v) in properties {
		match k.as_str() {
			"click"   if events_spec.pointer_click => {
				events.pointer_click = v.clone();
			},
			"press"   if events_spec.pointer_press => {
				events.pointer_press = v.clone();
			},
			"release" if events_spec.pointer_release => {
				events.pointer_release = v.clone();
			},
			"move"    if events_spec.pointer_move => {
				events.pointer_move = v.clone();
			},
			"in"      if events_spec.pointer_in => {
				events.pointer_in = v.clone();
			},
			"out"     if events_spec.pointer_out => {
				events.pointer_out = v.clone();
			},
			_ => {
				eprintln!("tried to set nonexistent property `{k}`");
			}
		}
	}
}

fn set_events_properties(
	properties: &HashMap<String, Value>,
	events_spec: &EventsSpec,
	events: &mut Events
) {
	for (k, v) in properties {
		match k.as_str() {
			"pointer" => {
				if let Value::Object(map) = v {
					set_pointer_events_properties(map, events_spec, events);
				}
			},
			_ => {
				eprintln!("tried to set nonexistent property `{k}`");
			}
		}
	}
}

fn set_properties(
	properties: &HashMap<String, Value>,
	events_spec: &EventsSpec,
	events: &mut Events,
	element_impl: &mut Box<dyn ElementImpl>,
	added_properties: &mut AddedProperties,
) {
	for (k, v) in properties {
		if k == "self" {
			match v {
				Value::Object(map) => {
					set_properties(
						map,
						events_spec,
						events,
						element_impl,
						&mut AddedProperties::None
					);
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
			SetPropertyResult::Set       => { continue; },
			SetPropertyResult::Ignore    => {},
			SetPropertyResult::TypeError => {
				eprintln!("type error when setting property `{k}`");
				continue;
			},
		}
		if let ("events", Value::Object(map)) = (k.as_str(), v) {
			set_events_properties(map, &events_spec, events);
			continue;
		}
		eprintln!("tried to set nonexistent property `{k}`");
	}
}

fn can_coerce(from: &Type, to: &Type) -> bool {
	if *to == Type::Any || *from == *to {
		return true;
	}

	let (from, to) = match (from, to) {
		(_, Type::String) => { return true },
		(Type::Iter(from), Type::Iter(to)) => { return can_coerce(from, to) },
		(Type::Object(from), Type::Object(to)) => (from, to),
		_ => { return false }
	};

	for (k, to) in to {
		if let Some(from) = from.get(k) {
			if !can_coerce(from, to) {
				return false;
			}
		} else {
			return false
		}
	}

	true
}

fn check_binding(
	scope: &mut Module,
	expected_type: Option<&Type>,
	path: &[String],
) -> Option<(Type, Ctx)> {
	let mut binding_type = None;
	let mut ctx = Ctx::Component;
	for map in scope.stack.iter().rev() {
		if let Some(t) = map.get(&path[0]) {
			binding_type = Some(t);
			ctx = Ctx::Repeater;
			break;
		}
	}
	if binding_type.is_none() {
		binding_type = scope.props.get(&path[0]).map(|e| &e.prop_type);
	}
	for segment in path.iter().skip(1) {
		if let Some(Type::Object(map)) = binding_type {
			binding_type = map.get(segment);
		} else {
			binding_type = None;
			break;
		}
	}
	match (binding_type, expected_type) {
		(Some(binding_type), Some(expected_type)) => {
			if !can_coerce(binding_type, expected_type) {
				eprintln!("expected type {:?}, found {:?}", expected_type, binding_type);
			}
		},
		(None, _) => {
			eprintln!("binding to undeclared property: {}", path.join("."));
		},
		_ => {},
	}
	binding_type.map(|t| (t.clone(), ctx))
}

fn check_and_push_repeater_bindings(
	scope: &mut Module,
	repeater: &mut Option<Repeater>,
) {
	let map = if let Some(repeater) = repeater.as_mut() {
		match &mut repeater.collection {
			Value::Binding(Expr::Path(path, ref mut ctx)) => {
				let t = check_binding(scope, Some(&Type::Iter(Box::new(Type::Any))), &path);
				let item_type = if let Some((Type::Iter(t), new_ctx)) = t {
					*ctx = new_ctx;
					*t.clone()
				} else {
					Type::Any
				};
				let mut map = hashmap![repeater.item.clone() => item_type];
				if let Some(index) = &repeater.index {
					map.insert(index.clone(), Type::Int);
				}
				map
			},
			Value::Int(_) => {
				let mut map = hashmap![repeater.item.clone() => Type::Int];
				if let Some(index) = &repeater.index {
					map.insert(index.clone(), Type::Int);
				}
				map
			}
			x => {
				unimplemented!("{:?}", x);
			},
		}
	} else {
		HashMap::new()
	};

	scope.stack.push(map);
}

fn check_bindings(
	scope: &mut Module,
	expected_prop_types: &HashMap<String, Type>,
	received_props: &mut HashMap<String, Value>,
	condition: &mut Option<Value>,
) {
	fn check_prop_bindings(
		scope: &mut Module,
		received_props: &mut HashMap<String, Value>,
		expected_types: Option<&HashMap<String, Type>>
	) {
		for (k, v) in received_props {
			let expected_type = expected_types.and_then(|e| e.get(k));
			match v {
				Value::Object(map) => {
					let expected_types = match expected_type {
						Some(Type::Object(map)) => Some(map),
						_ => None,
					};
					check_prop_bindings(scope, map, expected_types);
				},
				Value::Binding(Expr::Path(path, ref mut ctx)) => {
					if let Some((_, new_ctx)) = check_binding(scope, expected_type, path) {
						*ctx = new_ctx;
					}
				},
				_ => {},
			}
		}
	}
	check_prop_bindings(scope, received_props, Some(expected_prop_types));

	// macro_rules! check_event_binding {
	// 	($id:ident) => {
	// 		match &events.$id {
	// 			Value::Binding(Expr::Path(path, Ctx::Component)) => {
	// 				check_binding(scope, Some(&Type::Callback), &path);
	// 			},
	// 			Value::Unset => {},
	// 			_ => {
	// 				unimplemented!();
	// 			},
	// 		}
	// 	};
	// }
	// check_event_binding!(pointer_click);
	// check_event_binding!(pointer_press);
	// check_event_binding!(pointer_release);
	// check_event_binding!(pointer_move);
	// check_event_binding!(pointer_in);
	// check_event_binding!(pointer_out);

	if let Some(condition) = condition.as_mut() {
		match condition {
			Value::Binding(Expr::Path(path, ref mut ctx)) => {
				if let Some((_, new_ctx)) = check_binding(scope, Some(&Type::Boolean), &path) {
					*ctx = new_ctx;
				}
			},
			_ => {
				unimplemented!();
			},
		}
	}
}

impl Element {
	fn construct_element(
		scope: &mut Module,
		parse_tree: &ParserElement,
		mut added_properties: AddedProperties,
	) -> Result<Self, String> {
		let mut repeater = parse_tree.repeater.as_ref().map(|e| Repeater {
			index: e.index.as_ref().map(|e| e.into()),
			item: e.item.clone(),
			collection: e.collection.clone(),
		});

		check_and_push_repeater_bindings(scope, &mut repeater);

		let ConstructedElementImpl {
			mut element_impl,
			children,
			events_spec,
		} = scope.construct(&parse_tree)?;

		let mut condition = parse_tree.condition.clone();
		let mut properties = parse_tree.properties.clone();

		check_bindings(
			scope,
			&element_impl.property_types(),
			&mut properties,
			&mut condition,
		);

		let mut events = Events::default();
		set_properties(
			&properties,
			&events_spec,
			&mut events,
			&mut element_impl,
			&mut added_properties,
		);

		if repeater.is_some() {
			scope.stack.pop();
		}

		Ok(Element {
			tag: parse_tree.path.join("."),
			condition,
			repeater,
			children,
			element_impl,
			added_properties,
			events,
		})
	}

	pub fn data(&self) -> ElementData {
		ElementData {
			tag: &self.tag,
			condition: &self.condition,
			repeater: &self.repeater,
			children: &self.children,
			added_properties: &self.added_properties,
			events: &self.events,
		}
	}
}

fn build_elements(scope: &mut Module, parse_tree: &[ParserContent]) -> Vec<Content> {
	build_elements_with_added_properties(scope, parse_tree, AddedProperties::None)
}

fn build_elements_with_added_properties(
	scope: &mut Module,
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

pub fn build_component(scope: &mut Module, parse_tree: &ParserComponent) -> Component {
	match Element::construct_element(scope, &parse_tree.root, AddedProperties::None) {
		Ok(root) => {
			Component { root, props: parse_tree.props.clone(), name: parse_tree.name.clone() }
		},
		Err(message) => {
			eprintln!("Error: {}", message);
			Component { root: Element::default(), props: parse_tree.props.clone(), name: parse_tree.name.clone() }
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
	pub fn construct(scope: &mut Module, parse_tree: &ParserElement) -> ConstructedElementImpl {
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
		
		ConstructedElementImpl::new(
			Box::new(data),
			build_elements(scope, &parse_tree.children),
			Default::default(),
		)
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
	pub fn construct(scope: &mut Module, parse_tree: &ParserElement) -> ConstructedElementImpl {
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
			Default::default(),
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
	pub x: Value,
	pub y: Value,
	pub color: Value,
	pub max_width: Value,
	pub padding: Value,
}

impl Span {
	pub fn construct(scope: &mut Module, parse_tree: &ParserElement) -> ConstructedElementImpl {
		ConstructedElementImpl::new(
			Box::new(
				Span {
					x: Value::Px(0.0),
					y: Value::Px(0.0),
					color: Value::Color(0,0,0,1.0),
					max_width: Value::Unset,
					padding: Value::Px(0.0),
				}
			),
			build_elements(scope, &parse_tree.children),
			EventsSpec::default(),
		)
	}
}

impl ElementImpl for Span {
	fn property_types(&self) -> HashMap<String, Type> {
		hashmap![
			"color".into() => Type::Brush,
			"x".into() => Type::Length,
			"y".into() => Type::Length,
			"max_width".into() => Type::Length,
			"padding".into() => Type::Length,
		]
	}

	fn set_property(&mut self, k: &String, v: &Value) -> SetPropertyResult {
		match k.as_str() {
			"color" => { self.color = v.clone() },
			"x" => { self.x = v.clone() },
			"y" => { self.y = v.clone() },
			"max_width" => { self.max_width = v.clone() },
			"padding" => { self.padding = v.clone() },
			_ => { return SetPropertyResult::Ignore },
		}
		SetPropertyResult::Set
	}
}

#[derive(Debug)]
pub struct Text {
	pub content: Value,
}

impl Text {
	pub fn construct(scope: &mut Module, parse_tree: &ParserElement) -> ConstructedElementImpl {
		ConstructedElementImpl::new(
			Box::new(Text { content: Value::String("".to_owned()) }),
			build_elements(scope, &parse_tree.children),
			Default::default(),
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
	pub prop_decls: HashMap<String, PropDecl>,
	pub properties: HashMap<String, Value>,
}

impl ComponentInstance {
	pub fn construct(
		scope: &mut Module,
		component: &Component,
		parse_tree: &ParserElement,
	) -> ConstructedElementImpl {
		let data = ComponentInstance {
			name: component.name.clone(),
			prop_decls: component.props.clone(),
			properties: HashMap::new(),
		};
		ConstructedElementImpl::new(
			Box::new(data),
			build_elements(scope, &parse_tree.children),
			Default::default(),
		)
	}
}

impl ElementImpl for ComponentInstance {
	fn set_property(&mut self, k: &String, v: &Value) -> SetPropertyResult {
		if self.prop_decls.contains_key(k) {
			self.properties.insert(k.clone(), v.clone());
			return SetPropertyResult::Set
		}
		SetPropertyResult::Ignore
	}

	fn property_types(&self) -> HashMap<String, Type> {
		self.prop_decls
			.iter()
			.fold(
				HashMap::new(),
				|mut map, (k, v)| { map.insert(k.clone(), v.prop_type.clone()); map })
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
	pub rect: Rect,
	// pub width: Value,
	// pub height: Value,
	// pub x: Value,
	// pub y: Value,
	pub padding: Value,
	pub spacing: Value,
	pub column: bool,
	pub grow: bool,
}

impl Layout {
	pub fn row_stretch(scope: &mut Module, parse_tree: &ParserElement) -> ConstructedElementImpl {
		Self::construct(scope, parse_tree, false, false)
	}
	pub fn row_grow(scope: &mut Module, parse_tree: &ParserElement) -> ConstructedElementImpl {
		Self::construct(scope, parse_tree, false, true)
	}
	pub fn column_stretch(scope: &mut Module, parse_tree: &ParserElement) -> ConstructedElementImpl {
		Self::construct(scope, parse_tree, true, false)
	}
	pub fn column_grow(scope: &mut Module, parse_tree: &ParserElement) -> ConstructedElementImpl {
		Self::construct(scope, parse_tree, true, true)
	}
	pub fn construct(scope: &mut Module, parse_tree: &ParserElement, column: bool, grow: bool) -> ConstructedElementImpl {
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
			rect: Rect {
				clip: Value::Boolean(true),
				x: Value::Px(0.0),
				y: Value::Px(0.0),
				width,
				height,
				border_width: Value::Px(0.0),
				border_color: Value::Color(0,0,0,0.0),
				background: Value::Color(0,0,0,0.0),
			},
			padding: Value::Px(0.0),
			spacing: Value::Px(0.0),
			column,
			grow,
		};
		ConstructedElementImpl::new(
			Box::new(data),
			build_elements_with_added_properties(
				scope,
				&parse_tree.children,
				AddedProperties::Layout(LayoutItem::new(column, grow)),
			),
			EventsSpec {
				pointer_click: true,
				pointer_out: true,
				pointer_in: true,
				pointer_press: true,
				pointer_release: true,
				..Default::default()
			},
		)
	}
}

impl ElementImpl for Layout {
	fn property_types(&self) -> HashMap<String, Type> {
		let mut props = self.rect.property_types();
		match (self.column, self.grow) {
			(false, true) => {
				props.remove("width".into());
			},
			(true, true) => {
				props.remove("height".into());
			},
			_ => {},
		}
		props
	}

	fn set_property(&mut self, k: &String, v: &Value) -> SetPropertyResult {
		match (k.as_str(), self.column, self.grow) {
			("width", _, false) => { self.rect.width = v.clone() },
			("height", _, false) => { self.rect.height = v.clone() },
			("width", true, true) => { self.rect.width = v.clone() },
			("height", false, true) => { self.rect.height = v.clone() },
			("clip", ..) => { self.rect.clip = v.clone() },
			("x", ..) => { self.rect.x = v.clone() },
			("y", ..) => { self.rect.y = v.clone() },
			("width", ..) => { self.rect.width = v.clone() },
			("height", ..) => { self.rect.height = v.clone() },
			("border_width", ..) => { self.rect.border_width = v.clone() },
			("border_color", ..) => { self.rect.border_color = v.clone() },
			("background", ..) => { self.rect.background = v.clone() },
			("padding", ..) => { self.padding = v.clone() }
			("spacing", ..) => { self.spacing = v.clone() }
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



