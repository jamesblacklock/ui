use std::rc::Rc;

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

pub trait Component<T: HostAbi> {
	fn update(&mut self, parent: &mut Element<T>);
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
pub struct Element<T: HostAbi> {
	pub element_impl: ElementImpl,
	pub children: Vec<Element<T>>,
	pub show: bool,
	pub group: bool,
	pub events: Events<T>,
}

impl <T: HostAbi> Element<T> {
	pub fn root() -> Self {
		Element {
			element_impl: ElementImpl::Root,
			children: Vec::new(),
			show: true,
			group: false,
			events: Events::default()
		}
	}

	pub fn new(e: ElementImpl) -> Element<T> {
		Element {
			element_impl: e,
			children: Vec::new(),
			show: true,
			group: false,
			events: Events::default()
		}
	}
}

use std::cell::Cell;

pub trait HostAbi: std::fmt::Debug {
	fn call(&self);
}

#[derive(Debug)]
pub struct NoAbi(());
impl HostAbi for NoAbi {
	fn call(&self) {
		unreachable!()
	}
}

pub enum CallbackInner<T: HostAbi> {
	Empty,
	HostAbi(T),
	Native(Box<dyn Fn()>),
}

impl <T: HostAbi> CallbackInner<T> {
	fn call(&self) {
		match self {
			CallbackInner::Empty => {},
			CallbackInner::HostAbi(abi) => abi.call(),
			CallbackInner::Native(f) => f(),
		}
	}
}

impl <T: HostAbi> Default for CallbackInner<T> {
	fn default() -> Self {
		CallbackInner::Empty
	}
}

impl <T: HostAbi> std::fmt::Debug for CallbackInner<T> {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			CallbackInner::Empty      => write!(fmt, "Empty"),
			CallbackInner::HostAbi(abi) => write!(fmt, "HostAbi({abi:?})"),
			CallbackInner::Native(_)  => write!(fmt, "Native(Box<dyn Fn()>)"),
		}
	}
}

pub struct Callback<T: HostAbi = NoAbi>(pub Rc<Cell<CallbackInner<T>>>);

impl <T: HostAbi> Clone for Callback<T> {
	fn clone(&self) -> Self {
		Callback(self.0.clone())
	}
}

impl <T: HostAbi> Default for Callback<T> {
	fn default() -> Self {
		Callback(Rc::new(Cell::new(CallbackInner::Empty)))
	}
}

impl <T: HostAbi, F: 'static + Fn()> From<&'static F> for Callback<T> {
	fn from(f: &'static F) -> Self {
		let f: &'static dyn Fn() = f;
		Callback(Rc::new(Cell::new(CallbackInner::Native(Box::new(f)))))
	}
}

impl <T: HostAbi> std::fmt::Debug for Callback<T> {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		let item = self.0.take();
		write!(fmt, "Callback({:?})", item)?;
		self.0.set(item);
		Ok(())
	}
}

impl <T: HostAbi> Callback<T> {
	pub fn call(&self) {
		let f = self.0.take();
		f.call();
		self.0.set(f);
	}
}

impl <T: HostAbi> std::cmp::PartialEq for Callback<T> {
	fn eq(&self, other: &Callback<T>) -> bool {
		Rc::ptr_eq(&self.0, &other.0)
	}
}

pub fn element_in<T: HostAbi>(parent: &mut Element<T>, e: ElementImpl, i: usize) -> &mut Element<T> {
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

pub fn element_out<T: HostAbi>(parent: &mut Element<T>, e: ElementImpl, i: usize) {
	element_in(parent, e, i).show = false;
}

pub fn begin_group<T: HostAbi>(parent: &mut Element<T>, i: usize) -> &mut Element<T> {
	let mut e = element_in(parent, ElementImpl::Group, i);
	e.group = true;
	e
}

pub fn end_group<T: HostAbi>(parent: &mut Element<T>, i: usize) {
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
pub struct Events<T: HostAbi>  {
	pub pointer_click: Option<Callback<T>>,
	pub pointer_press: Option<Callback<T>>,
	pub pointer_release: Option<Callback<T>>,
	pub pointer_move: Option<Callback<T>>,
	pub pointer_in: Option<Callback<T>>,
	pub pointer_out: Option<Callback<T>>,
}

impl <T: HostAbi> Default for Events<T> {
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

pub fn handle_event<T: HostAbi>(parent: &mut Element<T>, event_type: EventType, callback: Option<Callback<T>>) {
	match event_type {
		EventType::PointerClick   => parent.events.pointer_click   = callback,
		EventType::PointerPress   => parent.events.pointer_press   = callback,
		EventType::PointerRelease => parent.events.pointer_release = callback,
		EventType::PointerMove    => parent.events.pointer_move    = callback,
		EventType::PointerIn      => parent.events.pointer_in      = callback,
		EventType::PointerOut     => parent.events.pointer_out     = callback,
	}
}

