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
	pub fn set_index(&mut self, i: usize, value: T) {
		match self {
			Iterable::Int(_) => {},
			Iterable::Array(a) => {
				if i < a.len() {
					a[i] = value;
				}
			}
		}
	}
	pub fn len(&self) -> usize {
		match self {
			Iterable::Int(n) => std::cmp::max(*n, 0) as usize,
			Iterable::Array(a) => a.len()
		}
	}
}

impl Iterable<String> {
	pub fn iter<'a>(&'a self) -> Box<dyn std::iter::Iterator<Item = String> + 'a> {
		match self {
			Iterable::Int(n) => Box::new((0..*n).map(|e| e.to_string())),
			Iterable::Array(a) => Box::new(a.iter().cloned())
		}
	}
	pub fn get_index(&self, i: usize) -> String {
		match self {
			Iterable::Int(n) => if (i as i32) < *n { (i as i32).to_string() } else { 0.to_string() },
			Iterable::Array(a) => a.get(i).map(|e| e.clone()).unwrap_or_default(),
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
	pub fn get_index(&self, i: usize) -> i32 {
		match self {
			Iterable::Int(n) => if (i as i32) < *n { i as i32 } else { 0 },
			Iterable::Array(a) => a.get(i).map(|e| e.clone()).unwrap_or_default(),
		}
	}
}

pub trait Convert<T> {
	fn convert(&self) -> T;
}

impl <T: Clone> Convert<T> for T {
	fn convert(&self) -> T {
		self.clone()
	}
}

impl Convert<String> for i32 {
	fn convert(&self) -> String {
		self.to_string()
	}
}

pub trait ComponentBase: std::fmt::Debug + Component {
	type Abi: HostAbi;
	fn update<D: ElementData>(this: Rc<RefCell<Self>>, parent: &mut GenericElement<D>);
}

pub trait Component: Default {
	fn on_init(&mut self) {}
	fn on_update(&mut self) {}
}

pub trait DefaultProps: Default {
	fn default() -> Self {
		Default::default()
	}
}

#[derive(Debug)]
pub enum ElementImpl {
	Root(f32, f32),
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
			&ElementImpl::Root(w, h) => {
				Some(PxBounds { x: 0.0, y: 0.0, width: w, height: h })
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
	pub color: Color,
}

#[derive(Debug)]
pub struct Text {
	pub content: String,
}

pub trait ElementData: Default {}

#[derive(Debug)]
pub struct GenericElement<D: ElementData> {
	id: usize,
	pub data: D,
	pub element_impl: ElementImpl,
	pub children: Vec<GenericElement<D>>,
	pub show: bool,
	pub group: bool,
	pub events: Events,
}

fn next_id() -> usize {
	std::thread_local!(static COUNTER: RefCell<usize> = RefCell::new(0));
	COUNTER.with(|c| {
		let id = *c.borrow();
		*c.borrow_mut() = id + 1;
		id
	})
}

impl <D: ElementData> GenericElement<D> {
	pub fn root() -> Self {
		GenericElement {
			id: next_id(),
			element_impl: ElementImpl::Root(0.0, 0.0),
			children: Vec::new(),
			show: true,
			group: false,
			events: Default::default(),
			data: Default::default(),
		}
	}

	pub fn new(e: ElementImpl) -> Self {
		GenericElement {
			id: next_id(),
			element_impl: e,
			children: Vec::new(),
			show: true,
			group: false,
			events: Default::default(),
			data: Default::default(),
		}
	}

	pub fn id(&self) -> usize {
		self.id
	}

	pub fn element_in(&mut self, e: ElementImpl, i: usize) -> &mut Self {
		if i < self.children.len() {
			let element = &mut self.children[i];
			element.show = true;
			element.element_impl = e;
			element
		} else if i == self.children.len() {
			self.children.push(GenericElement::new(e));
			self.children.last_mut().unwrap()
		} else {
			panic!("this should never happen")
		}
	}

	pub fn element_out(&mut self, e: ElementImpl, i: usize) {
		self.element_in(e, i).show = false;
	}

	pub fn begin_group(&mut self, i: usize) -> &mut Self {
		let mut e = self.element_in(ElementImpl::Group, i);
		e.group = true;
		e
	}

	pub fn end_group(&mut self, i: usize) {
		for e in self.children.iter_mut().skip(i) {
			e.show = false;
		}
	}

	pub fn handle_event<C: ComponentBase + 'static>(&mut self, component: Rc<RefCell<C>>, event_type: EventType, callback: Option<Callback<C>>) {
		match event_type {
			EventType::PointerClick   => self.events.pointer_click   = callback.map(|c| c.bind(&component)),
			EventType::PointerPress   => self.events.pointer_press   = callback.map(|c| c.bind(&component)),
			EventType::PointerRelease => self.events.pointer_release = callback.map(|c| c.bind(&component)),
			EventType::PointerMove    => self.events.pointer_move    = callback.map(|c| c.bind(&component)),
			EventType::PointerIn      => self.events.pointer_in      = callback.map(|c| c.bind(&component)),
			EventType::PointerOut     => self.events.pointer_out     = callback.map(|c| c.bind(&component)),
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
	fn call(&self) { unreachable!() }
	fn id(&self) -> usize { unreachable!() }
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

