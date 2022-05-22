#![allow(dead_code)]
// #![allow(unused_variables)]
use maplit::hashmap;

use std::collections::HashMap;

mod parser;
mod elements;
mod web;
mod native;

use elements as el;
use elements::{Component, ComponentInstance};

pub struct Module<'a> {
	builtins: HashMap<String, Item>,
	imports: &'a HashMap<String, PathBuf>,
	props: &'a HashMap<String, PropDecl>,
	stack: Vec<HashMap<String, Type>>,
	components: &'a HashMap<PathBuf, Component>,
}

#[derive(Debug)]
pub struct Import {
	pub path: PathBuf,
	pub alias: Option<String>,
}

pub enum Item {
	Module(HashMap<String, Item>),
	Constructor(el::Constructor),
}

#[derive(Debug, Clone)]
pub struct PropDecl {
	pub is_pub: bool,
	pub name: String,
	pub prop_type: Type,
	pub default: Option<Value>,
}

impl <'a> Module<'a> {
	pub fn new(
		imports: &'a HashMap<String, PathBuf>,
		components: &'a HashMap<PathBuf, Component>,
		props: &'a HashMap<String, PropDecl>,
	) -> Self {
		Self {
			imports,
			props,
			components,
			stack: Vec::new(),
			builtins: hashmap![
				// String::from("window") => Item::Constructor(el::Window::construct),
				String::from("rect")   => Item::Constructor(el::Rect::construct),
				String::from("text")   => Item::Constructor(el::Text::construct),
				String::from("span")   => Item::Constructor(el::Span::construct),
				String::from("scroll") => Item::Constructor(el::Scroll::construct),
				String::from("row") => Item::Module(hashmap![
					String::from("grow") => Item::Constructor(el::Layout::row_grow),
					String::from("stretch") => Item::Constructor(el::Layout::row_stretch),
				]),
				String::from("column") => Item::Module(hashmap![
					String::from("grow") => Item::Constructor(el::Layout::column_grow),
					String::from("stretch") => Item::Constructor(el::Layout::column_stretch),
				]),
				// String::from("img")    => Item::Constructor(el::Img::construct),
			]
		}
	}

	pub fn construct(
		&mut self,
		parse_tree: &parser::Element,
	) -> Result<el::ConstructedElementImpl, String> {
		assert!(parse_tree.path.len() > 0);

		if parse_tree.path.len() == 1 {
			if let Some(file_path) = self.imports.get(&parse_tree.path[0]) {
				let component = self.components.get(file_path).unwrap();
				return Ok(ComponentInstance::construct(self, &component, parse_tree));
			}
		}

		Ok(self.lookup(&parse_tree.path)?(self, parse_tree))
	}
	
	fn lookup(&self, path: &Vec<String>) -> Result<el::Constructor, String> {
		let mut map = &self.builtins;
		let mut it = path.iter().peekable();
		loop {
			let segment = it.next().unwrap();

			if let Some(item) = map.get(segment) {
				match item {
					Item::Module(next_map) => {
						if it.peek().is_none() {
							return Err(format!("'{}' is a module, not an element", segment))
						}
						map = next_map;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ctx {
	Component,
	Element,
	Parent,
}

#[derive(Debug, Clone)]
pub enum Expr {
	Path(Vec<String>, Ctx),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
	Any,
	Int,
	Length,
	Brush,
	String,
	Boolean,
	Alignment,
	Callback,
	Iter(Box<Type>),
	Object(HashMap<String, Type>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
	Stretch,
	Start,
	Center,
	End,
}

#[derive(Debug, Clone)]
pub enum Value {
	Px(f32),
	Float(f32),
	Int(i32),
	Color(u8, u8, u8, f32),
	String(String),
	Boolean(bool),
	Binding(Expr),
	Alignment(Alignment),
	Object(HashMap<String, Value>),
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
				Value::Color(r, g, b, 1.0)
			},
			6 => {
				let r = (hex_to_int(hex[0]) << 4) + hex_to_int(hex[1]);
				let g = (hex_to_int(hex[2]) << 4) + hex_to_int(hex[3]);
				let b = (hex_to_int(hex[4]) << 4) + hex_to_int(hex[5]);
				Value::Color(r, g, b, 1.0)
			},
			_ => unreachable!(),
		}
	}
}

use std::{ fs, process, path::PathBuf, io::Read };

fn load_single_ui_component<'a>(exe: &str, path: PathBuf) -> Result<parser::Component, String> {
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
			return Err(format!("{}: parse error", exe));
		};

		Ok(component)
}

fn resolve_ui_import<'a>(
	exe: &str,
	import: Import,
	components: &'a mut HashMap<PathBuf, parser::Component>)
	-> Result<(String, PathBuf), String> {
	
	let pathbuf = if let Ok(path) = fs::canonicalize(&import.path) {
		Some(path)
	} else if let Ok(path) = fs::canonicalize(import.path.with_extension("ui")) {
		Some(path)
	} else {
		None
	};
	let pathbuf = if pathbuf.is_none() || !pathbuf.as_ref().unwrap().is_file() {
		return Err(format!("{}: invalid path specified: {}", exe, import.path.display()));
	} else {
		pathbuf.unwrap()
	};
	if let Some(component) = components.get(&pathbuf) {
		return Ok((component.name.clone(), pathbuf));
	}

	let mut component = load_single_ui_component(exe, pathbuf.clone())?;
	
	while let Some(mut import) = component.import_decls.pop() {
		let alias = import.alias.clone();
		if import.path.is_relative() {
			let mut pathbuf = pathbuf.parent().unwrap().to_path_buf();
			pathbuf.push(import.path);
			import.path = pathbuf;
		}
		let (name, path) = resolve_ui_import(exe, import, components)?;
		component.imports_map.insert(alias.unwrap_or(name), path);
	}
	
	let name = component.name.clone();
	components.insert(pathbuf.clone(), component);
	Ok((name, pathbuf))
}

fn load_ui_component<'a>(
	exe: &str,
	path: &str,
	parse_trees: &mut HashMap<PathBuf,
	parser::Component>) -> Result<PathBuf, String> {
	
	let ui_import = Import { path: path.into(), alias: None };
	let (_, path) = resolve_ui_import(exe, ui_import, parse_trees)?;
	Ok(path)
}

fn build_impl<'a>(
	path: &PathBuf,
	parse_trees: &HashMap<PathBuf, parser::Component>,
	components: &'a mut HashMap<PathBuf, Component>,
	web: bool,
) -> Result<&'a Component, String> {
	use parser::CompileStatus;
	
	let parse_tree = parse_trees.get(path).unwrap();
	match parse_tree.status.get() {
		CompileStatus::Ready => {},
		CompileStatus::Building => { return Err(String::from("encountered recursive import")); },
		CompileStatus::Done => { return Ok(components.get(path).unwrap()); },
	}
	parse_tree.status.set(CompileStatus::Building);
	
	for (_, path) in parse_tree.imports_map.iter() {
		build_impl(path, parse_trees, components, web)?;
	}

	let mut module = Module::new(&parse_tree.imports_map, components, &parse_tree.props);
	let component = el::build_component(&mut module, parse_tree);
	// println!("{:#?}", component);

	let mut dir = path.parent().unwrap().to_path_buf();
	dir.push("dist");

	let script_path = path.with_extension("js");
	let script = if script_path.is_file() {
		let mut script = String::new();
		fs::File::open(&script_path)
			.expect(&format!("failed to open file: {}", script_path.display()))
			.read_to_string(&mut script).unwrap();
		Some(script)
	} else {
		None
	};

	native::render(&component, script, &parse_tree.name, dir, web);

	parse_tree.status.set(CompileStatus::Done);
	components.insert(path.clone(), component);
	Ok(components.get(path).unwrap())
}

pub fn build(exe: &str, path: &str, web: bool) -> Result<Vec<PathBuf>, String> {
	let mut parse_trees = HashMap::new();
	let path = load_ui_component(&exe, &path, &mut parse_trees)?;
	// println!("{:#?}", component);

	let mut components = HashMap::new();
	build_impl(&path, &parse_trees, &mut components, web)?;

	Ok(components.into_iter().map(|(k,_)|k).collect())
}

pub fn watch(exe: &str, path: &str, web: bool) {
	use notify::{Watcher, RecursiveMode, DebouncedEvent, watcher};
	use std::sync::mpsc::channel;
	use std::time::Duration;

	let (tx, rx) = channel();
	let mut watcher = watcher(tx, Duration::from_millis(500)).unwrap();
	let mut prev_paths = Vec::new();

	let mut build_once = || {
		match build(exe, path, web) {
			Ok(paths) => {
				for path in prev_paths.iter() {
					watcher.unwatch(path).unwrap();
				}
				for path in paths.iter() {
					watcher.watch(path, RecursiveMode::NonRecursive).unwrap();
					println!("watching: {:?}", path);
				}
				prev_paths = paths;
				true
			},
			Err(message) => {
				eprintln!("{}", message);
				false
			}
		}
	};
	
	if build_once() == false {
		return;
	}

	loop {
		match rx.recv() {
		   Ok(DebouncedEvent::Write(_)) => {
			   println!("rebuilding...");
			   build_once();
		   },
		   Err(e) => {
			   eprintln!("error: {:?}", e);
			   process::exit(1);
		   },
		   _ => {},
		}
	}
}
