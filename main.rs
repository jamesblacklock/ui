#![allow(dead_code)]
// #![allow(unused_variables)]
#[macro_use] extern crate maplit;

use std::collections::HashMap;

mod parser;
mod elements;
mod web;

use elements as el;
use elements::{Element, Component};

#[derive(Clone)]
pub struct LookupScope<'a> {
	module: &'a Module,
	imports: Option<&'a HashMap<String, PathBuf>>,
}

impl <'a> LookupScope<'a> {
	fn construct(&self, parse_tree: &parser::Element) -> Result<(Box<dyn el::ElementImpl>, Vec<el::Element>), String> {
		if self.imports.is_some() && parse_tree.path.len() == 1 {
			if let Some(file_path) = self.imports.unwrap().get(&parse_tree.path[0]) {
				let component = self.module.global_imports.get(file_path).unwrap();
				let scope = LookupScope { module: self.module, imports: Some(&component.imports_map) };
				return Ok(Component::construct(&scope, &component.parse_tree, parse_tree));
			}
		}

		Ok(self.module.lookup(&parse_tree.path)?(self, parse_tree))
	}
}

pub struct Module {
	map: HashMap<String, Item>,
	global_imports: HashMap<PathBuf, parser::Component>,
}

#[derive(Debug)]
pub struct Import {
	pub path: String,
	pub alias: Option<String>,
}

pub enum Item {
	Module(Module),
	Constructor(el::Constructor),
}

impl Module {
	pub fn new(global_imports: HashMap<PathBuf, parser::Component>) -> Self {
		Self {
			global_imports,
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

	pub fn lookup(&self, path: &Vec<String>) -> Result<el::Constructor, String> {
		assert!(path.len() > 0);
		
		let mut map = &self.map;
		let mut it = path.iter().peekable();
		loop {
			let segment = it.next().unwrap();

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
pub enum Expr {
	Path(Vec<String>),
}

#[derive(Debug, Clone)]
pub enum Value {
	Px(i32),
	Float(f32),
	Color(u8, u8, u8),
	String(String),
	Boolean(bool),
	Binding(Expr),
	Null,
	Unset,
}

impl Value {
	fn is_set(&self) -> bool {
		if let Value::Unset = self {
			false
		} else {
			true
		}
	}
}

impl Default for Value {
	fn default() -> Self {
		Value::Unset
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

use std::{ env, fs, process, path::PathBuf, io::Read };

fn load_single_ui_component<'a>(exe: &str, path: PathBuf) -> parser::Component {
		let mut ui_string = String::new();
		fs::File::open(&path)
			.unwrap()
			.read_to_string(&mut ui_string)
			.unwrap();
		
		let component = if let Ok(mut component) = parser::parse(&ui_string) {
			component.name = path
				.file_stem()
				.unwrap()
				.to_string_lossy()
				.into();
			component
		} else {
			eprintln!("{}: parse error", exe);
			process::exit(1);
		};

		component
}

fn resolve_ui_import<'a>(exe: &str, import: Import, components: &'a mut HashMap<PathBuf, parser::Component>) -> (String, PathBuf) {
	let pathbuf = if let Ok(path) = fs::canonicalize(&import.path) {
		Some(path)
	} else if let Ok(path) = fs::canonicalize(import.path.clone() + ".ui") {
		Some(path)
	} else {
		None
	};
	let pathbuf = if pathbuf.is_none() || !pathbuf.as_ref().unwrap().is_file() {
		eprintln!("{}: invalid path specified: {}", exe, import.path);
		process::exit(1);
	} else {
		pathbuf.unwrap()
	};
	if let Some(component) = components.get(&pathbuf) {
		return (component.name.clone(), pathbuf);
	}

	let mut component = load_single_ui_component(exe, pathbuf.clone());
	
	while let Some(import) = component.import_decls.pop() {
		let alias = import.alias.clone();
		let (name, path) = resolve_ui_import(exe, import, components);
		component.imports_map.insert(alias.unwrap_or(name), path);
	}
	
	let name = component.name.clone();
	components.insert(pathbuf.clone(), component);
	(name, pathbuf)
}

fn load_ui_component<'a>(exe: &str, path: String, imports: &mut HashMap<PathBuf, parser::Component>) -> parser::Component {
	let ui_import = Import { path, alias: None };
	let (_, path) = resolve_ui_import(exe, ui_import, imports);
	imports.remove(&path).unwrap()
}

fn main() {

	let args: Vec<_> = env::args().collect();
	if args.len() != 2 {
		eprintln!("usage: {} <FILE>", args[0]);
		process::exit(1);
	}
	
	let mut imports = HashMap::new();
	let component = load_ui_component(&args[0], args[1].clone(), &mut imports);
	// println!("{:#?}", component);

	let module = Module::new(imports);
	let root = el::build_component(&module, &component);
	// println!("{:#?}", root);

	web::render(root, &component.name);
}