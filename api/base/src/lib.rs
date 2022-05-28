use std::cell::RefCell;
use std::rc::Rc;

mod callback;

pub use callback::{Callback, BoundCallback};

#[derive(Debug, Clone)]
pub struct PxBounds {
	pub x: f32,
	pub y: f32,
	pub width: f32,
	pub height: f32,
}

#[derive(Debug, Clone)]
pub struct Bounds {
	pub x: Length,
	pub y: Length,
	pub width: Length,
	pub height: Length,
}

impl Bounds {
	pub fn to_px_bounds(&self) -> PxBounds {
		PxBounds {
			x: self.x.to_px(),
			y: self.y.to_px(),
			width: self.width.to_px(),
			height: self.height.to_px(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Color {
	pub r: f32,
	pub g: f32,
	pub b: f32,
	pub a: f32,
}

#[derive(Debug, Clone)]
pub enum Length {
	Px(f32),
	In(f32),
	Cm(f32),
	Mm(f32),
}

impl Default for Length {
	fn default() -> Self {
		Length::Px(0.0)
	}
}

impl Length {
	pub fn to_px(&self) -> f32 {
		match *self {
			Length::Px(f) => f,
			_ => unimplemented!()
		}
	}
}

#[derive(Debug)]
pub enum Iterable<T> {
	Int(i32),
	Array(Vec<T>),
}

impl <T> Default for Iterable<T> {
	fn default() -> Self {
		Iterable::Int(0)
	}
}

impl <T> Iterable<T> {
	pub fn from<U: Clone + Into<T>, A: AsRef<[U]>>(array: A) -> Self {
		let vector = array.as_ref().iter().map(|e| e.clone().into()).collect();
		Iterable::Array(vector)
	}
}

impl Iterable<String> {
	pub fn iter<'a>(&'a self) -> Box<dyn std::iter::Iterator<Item = String> + 'a> {
		match self {
			Iterable::Int(n) => Box::new((0..*n).map(|e| e.to_string())),
			Iterable::Array(a) => Box::new(a.iter().cloned())
		}
	}
}

impl Iterable<i32> {
	pub fn iter<'a>(&'a self) -> Box<dyn std::iter::Iterator<Item = i32> + 'a> {
		match self {
			Iterable::Int(n) => Box::new(0..*n),
			Iterable::Array(a) => Box::new(a.iter().copied())
		}
	}
}

pub trait Component: std::fmt::Debug {
	type Abi: HostAbi;
	fn update(this: Rc<RefCell<Self>>, parent: &mut Element);
}

#[derive(Debug)]
pub enum ElementImpl {
	Root,
	Group,
	Rect(Rect),
	Span(Span),
	Text(Text),
}

impl ElementImpl {
	pub fn bounds(&self) -> Option<PxBounds> {
		match self {
			ElementImpl::Rect(rect) => {
				Some(rect.bounds.to_px_bounds())
			},
			_ => None,
		}
	}
}

#[derive(Debug)]
pub struct Rect {
	pub color: Color,
	pub bounds: Bounds,
}

#[derive(Debug)]
pub struct Span {
	pub max_width: Option<f32>,
	pub x: Length,
	pub y: Length,
}

#[derive(Debug)]
pub struct Text {
	pub content: String,
}

#[derive(Debug)]
pub struct Element {
	pub element_impl: ElementImpl,
	pub children: Vec<Element>,
	pub show: bool,
	pub group: bool,
	pub events: Events,
}

impl Element {
	pub fn root() -> Self {
		Element {
			element_impl: ElementImpl::Root,
			children: Vec::new(),
			show: true,
			group: false,
			events: Events::default()
		}
	}

	pub fn new(e: ElementImpl) -> Element {
		Element {
			element_impl: e,
			children: Vec::new(),
			show: true,
			group: false,
			events: Events::default()
		}
	}
}

pub trait HostAbi: std::fmt::Debug {
	fn call(&self);
	fn id(&self) -> usize;
}

#[derive(Debug)]
pub struct NoAbi(());
impl HostAbi for NoAbi {
	fn call(&self) {
		unreachable!()
	}
	fn id(&self) -> usize {
		unreachable!()
	}
}

pub fn element_in(parent: &mut Element, e: ElementImpl, i: usize) -> &mut Element {
	if i < parent.children.len() {
		let element = &mut parent.children[i];
		element.show = true;
		element.element_impl = e;
		element
	} else if i == parent.children.len() {
		parent.children.push(Element::new(e));
		parent.children.last_mut().unwrap()
	} else {
		panic!("this should never happen")
	}
}

pub fn element_out(parent: &mut Element, e: ElementImpl, i: usize) {
	element_in(parent, e, i).show = false;
}

pub fn begin_group(parent: &mut Element, i: usize) -> &mut Element {
	let mut e = element_in(parent, ElementImpl::Group, i);
	e.group = true;
	e
}

pub fn end_group(parent: &mut Element, i: usize) {
	for e in parent.children.iter_mut().skip(i) {
		e.show = false;
	}
}

pub enum EventType {
	PointerClick,
	PointerPress,
	PointerRelease,
	PointerMove,
	PointerIn,
	PointerOut,
}

#[derive(Debug)]
pub struct Events  {
	pub pointer_click: Option<BoundCallback>,
	pub pointer_press: Option<BoundCallback>,
	pub pointer_release: Option<BoundCallback>,
	pub pointer_move: Option<BoundCallback>,
	pub pointer_in: Option<BoundCallback>,
	pub pointer_out: Option<BoundCallback>,
}

impl Default for Events {
	fn default() -> Self {
		Events {
			pointer_click: None,
			pointer_press: None,
			pointer_release: None,
			pointer_move: None,
			pointer_in: None,
			pointer_out: None,
		}
	}
}

pub fn handle_event<C: Component + 'static>(component: Rc<RefCell<C>>, parent: &mut Element, event_type: EventType, callback: Option<Callback<C>>) {
	match event_type {
		EventType::PointerClick   => parent.events.pointer_click   = callback.map(|c| c.bind(&component)),
		EventType::PointerPress   => parent.events.pointer_press   = callback.map(|c| c.bind(&component)),
		EventType::PointerRelease => parent.events.pointer_release = callback.map(|c| c.bind(&component)),
		EventType::PointerMove    => parent.events.pointer_move    = callback.map(|c| c.bind(&component)),
		EventType::PointerIn      => parent.events.pointer_in      = callback.map(|c| c.bind(&component)),
		EventType::PointerOut     => parent.events.pointer_out     = callback.map(|c| c.bind(&component)),
	}
}

