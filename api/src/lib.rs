#![feature(try_blocks)]

cfg_if::cfg_if! {
	if #[cfg(all(feature = "web", feature = "native"))] {
		pub mod web;
		pub mod native;
		pub trait RenderTrait: web::RenderWeb + native::RenderNative {}
	} else if #[cfg(feature = "web")] {
		pub mod web;
		pub trait RenderTrait: web::RenderWeb {}
	} else if #[cfg(feature = "native")] {
		pub mod native;
		pub trait RenderTrait: native::RenderNative {}
	} else {
		pub trait RenderTrait {}
	}
}

impl <T> RenderTrait for T where T: ElementImpl {}

#[derive(Debug, Clone)]
pub struct Color {
	pub r: f32,
	pub g: f32,
	pub b: f32,
	pub a: f32,
}

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
pub enum Length {
	Px(f32),
}

impl Length {
	pub fn to_px(&self) -> f32 {
		match *self {
			Length::Px(f) => f,
			// _ => unimplemented!()
		}
	}
}

#[derive(Debug)]
pub enum Iterable<T> {
	Int(i32),
	Array(Vec<T>),
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

pub trait Component {
	fn update(&mut self, parent: &mut Element);
}

pub trait ElementImpl: RenderTrait + std::fmt::Debug {
	fn bounds(&self) -> Option<PxBounds> {
		None
	}
}

#[derive(Debug)]
pub struct Rect {
	pub color: Color,
	pub bounds: Bounds,
}

impl ElementImpl for Rect {
	fn bounds(&self) -> Option<PxBounds> {
		Some(self.bounds.to_px_bounds())
	}
}

#[derive(Debug)]
pub struct Span {
	pub max_width: Option<f32>,
	pub x: Length,
	pub y: Length,
}

impl ElementImpl for Span {
	fn bounds(&self) -> Option<PxBounds> {
		None
	}
}

#[derive(Debug)]
pub struct Text {
	pub content: String,
}

impl ElementImpl for Text {
	fn bounds(&self) -> Option<PxBounds> {
		None
	}
}

#[derive(Debug)]
struct Root {}
impl ElementImpl for Root {}

#[derive(Debug)]
pub struct Group;
impl ElementImpl for Group {}

#[derive(Debug)]
pub struct Element {
	pub element_impl: Box<dyn ElementImpl>,
	pub children: Vec<Element>,
	pub show: bool,
	pub group: bool,
}

impl Element {
	pub fn root() -> Element {
		Element {
			element_impl: Box::new(Root {}),
			children: Vec::new(),
			show: true,
			group: false,
		}
	}
}

pub fn element_in(parent: &mut Element, e: Box<dyn ElementImpl>, i: usize) -> &mut Element {
	if i < parent.children.len() {
		let element = &mut parent.children[i];
		element.show = true;
		element.element_impl = e;
		element
	} else if i == parent.children.len() {
		parent.children.push(Element { group: false, show: true, element_impl: e, children: Vec::new() });
		parent.children.last_mut().unwrap()
	} else {
		panic!("this should never happen")
	}
}
pub fn element_out(parent: &mut Element, e: Box<dyn ElementImpl>, i: usize) {
	element_in(parent, e, i).show = false;
}

pub fn begin_group(parent: &mut Element, i: usize) -> &mut Element {
	let mut e = element_in(parent, Box::new(Group), i);
	e.group = true;
	e
}

pub fn end_group(parent: &mut Element, i: usize) {
	for e in parent.children.iter_mut().skip(i) {
		e.show = false;
	}
}