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

#[derive(Clone)]
pub struct Color {
	pub r: f32,
	pub g: f32,
	pub b: f32,
	pub a: f32,
}

#[derive(Clone)]
pub struct Bounds {
	pub x: f32,
	pub y: f32,
	pub width: f32,
	pub height: f32,
}

pub trait Component {
	fn update(&mut self, parent: &mut Element);
}

pub trait ElementImpl: RenderTrait {
	fn bounds(&self) -> Option<Bounds> {
		None
	}
}

pub struct Rect {
	pub color: Color,
	pub bounds: Bounds,
}

impl ElementImpl for Rect {
	fn bounds(&self) -> Option<Bounds> {
		Some(self.bounds.clone())
	}
}

pub struct Span {
	pub max_width: Option<f32>,
	pub x: f32,
	pub y: f32,
}

impl ElementImpl for Span {
	fn bounds(&self) -> Option<Bounds> {
		None
	}
}

pub struct Text {
	pub content: String,
}

impl ElementImpl for Text {
	fn bounds(&self) -> Option<Bounds> {
		None
	}
}

struct Root {}
impl ElementImpl for Root {}

pub struct Element {
	pub element_impl: Box<dyn ElementImpl>,
	pub children: Vec<Element>,
	pub show: bool,
}

impl Element {
	pub fn root() -> Element {
		Element {
			element_impl: Box::new(Root {}),
			children: Vec::new(),
			show: true,
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
		parent.children.push(Element { show: true, element_impl: e, children: Vec::new() });
		parent.children.last_mut().unwrap()
	} else {
		panic!("this should never happen")
	}
}
pub fn element_out(parent: &mut Element, e: Box<dyn ElementImpl>, i: usize) {
	element_in(parent, e, i).show = false;
}