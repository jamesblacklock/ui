#![allow(dead_code)]
// #![allow(unused_variables)]
#[macro_use] extern crate maplit;

use std::collections::HashMap;

mod parser;
mod elements;
mod web;

use elements as el;
use elements::Element;

pub struct Module {
	name: String,
	map: HashMap<String, Item>,
}

pub enum Item {
	Module(Module),
	Constructor(el::Constructor),
}

impl Module {
	pub fn new<S: Into<String>>(name: S) -> Self {
		Self {
			name: name.into(),
			map: hashmap![
				// String::from("window") => Item::Constructor(el::Window::construct),
				String::from("rect")   => Item::Constructor(el::Rect::construct),
				String::from("text")   => Item::Constructor(el::Text::construct),
				String::from("span")   => Item::Constructor(el::Span::construct),
				// String::from("i")      => Item::Constructor(el::ChildPropertySetter::construct_i),
				// String::from("b")      => Item::Constructor(el::ChildPropertySetter::construct_b),
				// String::from("img")    => Item::Constructor(el::Img::construct),
				// String::from("panes")  => Item::Module(Module {
				// 	name: String::from("panes"),
				// 	map: hashmap![
				// 		String::from("h")  => Item::Constructor(el::PanesH::construct),
				// 		String::from("v")  => Item::Constructor(el::PanesV::construct),
				// ]}),
			]
		}
	}

	pub fn lookup(&self, path: &Vec<&str>) -> Result<el::Constructor, String> {
		assert!(path.len() > 0);
		
		let mut map = &self.map;
		let mut it = path.iter().peekable();
		loop {
			let &segment = it.next().unwrap();

			if let Some(item) = map.get(segment) {
				match item {
					Item::Module(module) => {
						if it.peek().is_none() {
							return Err(format!("'{}' is a module, not an element", segment))
						}
						map = &module.map;
					},
					Item::Constructor(constructor) => {
						if it.peek().is_some() {
							return Err(format!("'{}' is an element, not a module", segment))
						}
						return Ok(*constructor);
					},
				}
			} else {
				return Err(format!("item '{}' not found", segment))
			}
		}
	}
}

// #[derive(Debug, Clone, Copy)]
// pub enum Length {
// 	Px(i32),
// 	Percent(f32),
// }

// impl Default for Length {
// 	fn default() -> Length {
// 		Length::Px(0)
// 	}
// }

// #[derive(Debug, Clone, Copy)]
// pub enum Brush {
// 	Color(u8, u8, u8),
// }

// impl Default for Brush {
// 	fn default() -> Brush {
// 		Brush::Color(255, 255, 255)
// 	}
// }

#[derive(Debug, Clone)]
pub enum Value {
	Px(i32),
	Float(f32),
	Color(u8, u8, u8),
	String(String),
	Boolean(bool),
	Binding(String),
	Null,
}

impl Default for Value {
	fn default() -> Self {
		Value::Null
	}
}

fn hex_to_int(hex: u8) -> u8 {
	if hex >= '0' as u8 && hex <= '9' as u8 {
		hex - '0' as u8
	} else if hex >= 'A' as u8 && hex <= 'F' as u8 {
		10 + (hex - 'A' as u8)
	} else if hex >= 'a' as u8 && hex <= 'f' as u8 {
		10 + (hex - 'a' as u8)
	} else {
		assert!(false, "invalid hex char: {}", hex as char);
		unreachable!();
	}
}

impl Value {
	pub fn color_from_hex(hex: &str) -> Value {
		let hex = hex.as_bytes();
		assert!(hex.len() == 3 || hex.len() == 6);
		match hex.len() {
			3 => {
				let mut r = hex_to_int(hex[0]);
				let mut g = hex_to_int(hex[1]);
				let mut b = hex_to_int(hex[2]);
				r = (r << 4) + r;
				g = (g << 4) + g;
				b = (b << 4) + b;
				Value::Color(r, g, b)
			},
			6 => {
				let r = (hex_to_int(hex[0]) << 4) + hex_to_int(hex[1]);
				let g = (hex_to_int(hex[2]) << 4) + hex_to_int(hex[3]);
				let b = (hex_to_int(hex[4]) << 4) + hex_to_int(hex[5]);
				Value::Color(r, g, b)
			},
			_ => unreachable!(),
		}
	}

	// pub fn to_length(self) -> Length {
	// 	match self {
	// 		Value::Px(px) => Length::Px(px),
	// 		_ => Length::default(),
	// 	}
	// }

	// pub fn to_brush(self) -> Brush {
	// 	match self {
	// 		Value::Color(r, g, b) => Brush::Color(r, g, b),
	// 		_ => Brush::default(),
	// 	}
	// }

	// pub fn to_string(self) -> String {
	// 	match self {
	// 		Value::String(s) => {
	// 			// s.split_ascii_whitespace().collect::<Vec<_>>().join(" ")
	// 			s
	// 		},
	// 		_ => String::default(),
	// 	}
	// }

	// pub fn to_boolean(self) -> bool {
	// 	match self {
	// 		Value::Boolean(b) => {
	// 			b
	// 		},
	// 		_ => false,
	// 	}
	// }
}

fn main() {
	let ml = include_str!("./hello.ui");
	let parse_tree = parser::parse(ml).expect("parse error:");
	// println!("{:#?}", parse_tree);

	let module = Module::new("builtins");
	let root = el::build_element(&module, parse_tree);//build_dom(&module, parse_tree);
	// println!("{:#?}", dom);

	web::render(root, "hello");
}