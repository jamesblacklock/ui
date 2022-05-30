use std::path::PathBuf;
use std::fs::File;
use std::io::Write as IoWrite;
use std::fmt::Write as FmtWrite;
use std::collections::HashMap;
use quote::{quote, format_ident};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;

use super::{
	Value,
	Type,
	Ctx,
	Expr,
	PropDecl,
	elements::{
		Empty,
		Rect,
		Scroll,
		Span,
		Text,
		ComponentInstance,
		Layout,
		ElementData,
		Component,
		Element,
		Content,
	}
};

fn codegen_element(e: &Element, ctx: &mut CodeGenCtx) -> TokenStream {
	let parent = CodeGen::generate(e.element_impl.as_ref(), e.data(), ctx);

	let index = ctx.index;

	let mut children = Vec::new();
	for (i, child)in e.children.iter().enumerate() {
		ctx.index = i;
		match child {
			Content::Element(child) => children.push(codegen_element(child, ctx)),
			_ => unimplemented!()
		}
	}

	let mut events = Vec::new();

	macro_rules! gen_event {
		($event:ident) => {
			if e.events.$event.is_set() {
				let callback = e.events.$event.to_tokens();
				let event_type = format_ident!("{}", stringify!($event).to_case(Case::UpperCamel));
				events.push(quote!(
					e.handle_event(this.clone(), ui::EventType::#event_type, Some(#callback));
				));
			}
		};
	}

	gen_event!(pointer_click);
	gen_event!(pointer_press);
	gen_event!(pointer_release);
	gen_event!(pointer_move);
	gen_event!(pointer_in);
	gen_event!(pointer_out);

	if let Some(repeater) = &e.repeater {
		let collection = repeater.collection.to_tokens_iter();
		let group = quote!(
			for item in #collection {
				#parent
				let e = parent.element_in(e_impl, i);
				#(#events)*
				#(
					let e = {
						let parent = e;
						#children
						parent
					};
				)*
				i += 1;
			}
		);
		if let Some(cond) = &e.condition {
			let cond = cond.to_tokens();
			quote!(
				{
					let parent = parent.begin_group(#index);
					let mut i = 0;
					if #cond {
						#group
					}
					parent.end_group(i);
				}
			)
		} else {
			quote!(
				{
					let parent = parent.begin_group(#index);
					let mut i = 0;
					#group
					parent.end_group(i);
				}
			)
		}
	} else {
		let body = quote!(
			let e = parent.element_in(e_impl, #index);
			#(#events)*
			#(
				let e = {
					let parent = e;
					#children
					parent
				};
			)*
		);
		if let Some(cond) = &e.condition {
			let cond = cond.to_tokens();
			quote!(
				#parent
				if #cond {
					#body
				} else {
					parent.element_out(e_impl, #index);
				}
			)
		} else {
			quote!(
				#parent
				#body
			)
		}
	}
}

pub fn generate<S1: Into<String>, S2: Into<String>, P: Into<PathBuf>>(
	component: &Component,
	_script: Option<S1>,
	name: S2,
	path: P,
	web: bool,
) {
	let name = name.into();
	let struct_name = format_ident!("{}", name.clone().to_case(Case::UpperCamel));

	let mut ctx = CodeGenCtx::new(name, path);
	let code = codegen_element(&component.root, &mut ctx);

	let mut pub_fields = Vec::new();
	let mut pub_field_inits = Vec::new();
	let mut priv_fields = Vec::new();
	let mut priv_field_inits = Vec::new();
	for (name, decl) in component.props.iter() {
		let name = format_ident!("{}", name);
		let prop_type = decl.prop_type.to_tokens();
		if decl.is_pub {
			pub_fields.push(quote!(pub #name: #prop_type,));
			pub_field_inits.push(quote!(#name: props.#name,));
		} else {
			priv_fields.push(quote!(#name: #prop_type,));
			priv_field_inits.push(quote!(#name: Default::default(),));
		}
	}

	let web_code = if web {
		let interface_struct_name = format_ident!("{}Interface", struct_name);
		let abi_struct_name = format_ident!("{}Abi", struct_name);
		let attach_to_element = format_ident!("{}__attach_to_element", struct_name);
		let render_component = format_ident!("{}__render_component", struct_name);
		let update_component = format_ident!("{}__update_component", struct_name);
		let new_component = format_ident!("{}__new_component", struct_name);
		let drop_component = format_ident!("{}__drop_component", struct_name);
		let get_props_json = format_ident!("{}__get_props_json", struct_name);
		let props_json = gen_props_json(&component.props);
		let mut js_field_inits = Vec::new();
		let mut props = Vec::new();
		for (name, decl) in component.props.iter().filter(|(_, decl)| decl.is_pub) {
			let name_ident = format_ident!("{}", name);
			let setter_name = format_ident!("{}__set__{}", struct_name, name);

			let getter = match decl.prop_type {
				Type::Callback => {
					let call = format_ident!("{}__call__{}", struct_name, name);
					quote!(
						#[no_mangle]
						#[allow(non_snake_case)]
						pub fn #call(this: #abi_struct_name) {
							let interface = #interface_struct_name::from_abi(this);
							let callback = interface.component.borrow()
								.#name_ident.bind(&interface.component);
							interface.release_into_js();
							callback.call();
						}
					)
				},
				_ => {
					let getter_name = format_ident!("{}__get__{}", struct_name, name);
					quote!(
						#[no_mangle]
						#[allow(non_snake_case)]
						pub fn #getter_name(this: #abi_struct_name) -> ui::JsValue {
							let interface = #interface_struct_name::from_abi(this);
							let result = ui::AsJsValue::as_js_value(&interface.component.borrow().#name_ident);
							interface.release_into_js();
							return result;
						}
					)
				},
			};

			props.push(quote!(
				#getter
				#[no_mangle]
				#[allow(non_snake_case)]
				pub fn #setter_name(this: #abi_struct_name, value: ui::JsValue) {
					let interface = #interface_struct_name::from_abi(this);
					interface.component.borrow_mut().#name_ident = ui::FromJsValue::from_js_value(value);
					interface.release_into_js();
				}
			));

			match decl.prop_type {
				Type::Iter(_) => {
					let get_index = format_ident!("{}__get_index__{}", struct_name, name);
					let set_index = format_ident!("{}__set_index__{}", struct_name, name);
					let len = format_ident!("{}__len__{}", struct_name, name);
					props.push(quote!(
						#[no_mangle]
						#[allow(non_snake_case)]
						pub fn #get_index(this: #abi_struct_name, index: usize) -> ui::JsValue {
							let interface = #interface_struct_name::from_abi(this);
							let result = ui::AsJsValue::as_js_value(&interface.component.borrow().#name_ident.get_index(index));
							interface.release_into_js();
							result
						}
						#[no_mangle]
						#[allow(non_snake_case)]
						pub fn #set_index(this: #abi_struct_name, index: usize, value: ui::JsValue) {
							let interface = #interface_struct_name::from_abi(this);
							interface.component.borrow_mut().#name_ident.set_index(index, ui::FromJsValue::from_js_value(value));
							interface.release_into_js();
						}
						#[no_mangle]
						#[allow(non_snake_case)]
						pub fn #len(this: #abi_struct_name) -> usize {
							let interface = #interface_struct_name::from_abi(this);
							let result = interface.component.borrow().#name_ident.len();
							interface.release_into_js();
							result
						}
					));
				},
				_ => {},
			}

			js_field_inits.push(quote!(
				#name_ident: value.get_property(#name)
					.map(|e| ui::FromJsValue::from_js_value(e))
					.unwrap_or(defaults.#name_ident),));
		}

		quote!(
			#[cfg(target_arch = "wasm32")]
			mod web {
				use super::*;
				#[repr(transparent)]
				pub struct #abi_struct_name(usize);
				struct #interface_struct_name {
					component: std::rc::Rc<std::cell::RefCell<#struct_name>>,
					web_element: Option<ui::WebElement>,
					root: ui::Element,
				}
				impl #interface_struct_name {
					fn new(props: ui::JsValue) -> #interface_struct_name {
						ui::panic_hook::set_once();

						#interface_struct_name {
							component: std::rc::Rc::new(std::cell::RefCell::new(#struct_name::new(Props::from(props)))),
							web_element: None,
							root: ui::Element::root(),
						}
					}
					fn from_abi(abi: #abi_struct_name) -> Box<#interface_struct_name> {
						unsafe { Box::from_raw(std::mem::transmute(abi.0)) }
					}
					fn release_into_js(self: Box<#interface_struct_name>) -> #abi_struct_name {
						unsafe { #abi_struct_name(std::mem::transmute(Box::leak(self))) }
					}
				}
				impl Props {
					pub fn from(value: ui::JsValue) -> Self {
						let defaults = <Props as ui::DefaultProps>::default();
						Self {
							#(#js_field_inits)*
						}
					}
				}
				#[no_mangle]
				#[allow(non_snake_case)]
				pub fn #new_component(props: ui::JsValue) -> #abi_struct_name {
					Box::new(#interface_struct_name::new(props)).release_into_js()
				}
				#[no_mangle]
				#[allow(non_snake_case)]
				pub fn #drop_component(this: #abi_struct_name) {
					std::mem::drop(#interface_struct_name::from_abi(this));
				}
				#[no_mangle]
				#[allow(non_snake_case)]
				pub fn #attach_to_element(this: #abi_struct_name, element: ui::HtmlNode) {
					let mut interface = #interface_struct_name::from_abi(this);
					interface.web_element = Some(ui::WebElement::new(Some(std::rc::Rc::new(element))));
					interface.release_into_js();
				}
				#[no_mangle]
				#[allow(non_snake_case)]
				pub fn #render_component(this: #abi_struct_name, heap_ref: ui::JsValue) {
					let mut interface = #interface_struct_name::from_abi(this);
					if let Some(e) = interface.web_element.as_mut() {
						ui::render_html(&heap_ref, &mut interface.root, e);
					}
					interface.release_into_js();
				}
				#[no_mangle]
				#[allow(non_snake_case)]
				pub fn #update_component(this: #abi_struct_name) {
					use ui::ComponentBase;
					let mut interface = #interface_struct_name::from_abi(this);
					ComponentBase::update(interface.component.clone(), &mut interface.root);
					interface.release_into_js();
				}
				#[no_mangle]
				#[allow(non_snake_case)]
				pub fn #get_props_json() {
					let json = #props_json;
					ui::string_into_js(&json, |ptr, len| unsafe { ui::__send_string(ptr, len) });
				}
				#(#props)*
			}
			#[cfg(target_arch = "wasm32")]
			pub use web::*;
		)
	} else {
		quote!()
	};

	let code = quote!(
		pub type Callback = ui::Callback<#struct_name>;
		
		#[derive(Default, Debug)]
		#[allow(dead_code)]
		pub struct #struct_name {
			#(#pub_fields)*
			#(#priv_fields)*
		}
		#[derive(Default, Debug)]
		pub struct Props {
			#(#pub_fields)*
		}
		impl #struct_name {
			pub fn new(props: Props) -> Self {
				let mut instance = Self {
					#(#pub_field_inits)*
					#(#priv_field_inits)*
				};
				ui::Component::on_init(&mut instance);
				instance
			}
		}
		impl ui::ComponentBase for #struct_name {
			type Abi = ui::Abi;

			#[allow(unused_variables)]
			fn update<D: ui::ElementData>(this: std::rc::Rc<std::cell::RefCell<#struct_name>>, parent: &mut ui::GenericElement<D>) {
				ui::Component::on_update(&mut *this.borrow_mut());
				#code
			}
		}
		#web_code
	);

	writeln!(ctx.file, "{code}").unwrap();

	ctx.finalize();
}

fn gen_props_json(props: &HashMap<String, PropDecl>) -> String {
	let mut buf = String::new();
	write!(buf, "{{").unwrap();
	let mut it = props.iter().filter(|(_, e)| e.is_pub);
	if let Some((_, decl)) = it.next() {
		write!(buf, "\"{}\":", decl.name).unwrap();
		gen_type_json(&mut buf, &decl.prop_type);
	}
	for (_, decl) in it {
		write!(buf, ",\"{}\":", decl.name).unwrap();
		gen_type_json(&mut buf, &decl.prop_type);
	}
	write!(buf, "}}").unwrap();
	buf
}

fn gen_type_json(buf: &mut String, prop_type: &Type) {
	match prop_type {
		Type::Object(map) => {
			write!(buf, "{{").unwrap();
			let mut it = map.iter();
			if let Some((name, prop_type)) = it.next() {
				write!(buf, "\"{}\":", name).unwrap();
				gen_type_json(buf, prop_type);
			}
			for (name, prop_type) in it {
				write!(buf, ",\"{}\":", name).unwrap();
				gen_type_json(buf, prop_type);
			}
			write!(buf, "}}").unwrap();
		}
		Type::Length => {
			write!(buf, "\"Length\"").unwrap();
		}
		Type::Brush => {
			write!(buf, "\"Brush\"").unwrap();
		}
		Type::Alignment => {
			write!(buf, "\"Alignment\"").unwrap();
		}
		Type::String => {
			write!(buf, "\"String\"").unwrap();
		}
		Type::Boolean => {
			write!(buf, "\"Boolean\"").unwrap();
		}
		Type::Iter(t) => {
			write!(buf, "[").unwrap();
			gen_type_json(buf, t);
			write!(buf, "]").unwrap();
		}
		Type::Callback => {
			write!(buf, "\"Callback\"").unwrap();
		}
		t => {
			unimplemented!("rendering data type: {:?}", t);
		}
	}
}

pub struct CodeGenCtx {
	file: File,
	name: String,
	dir: PathBuf,
	tempname: PathBuf,
	index: usize,
}

impl CodeGenCtx {
	pub fn new<S: Into<String>, P: Into<PathBuf>>(name: S, dir: P) -> CodeGenCtx {
		let name = name.into();
		let dir = dir.into();
		std::fs::create_dir_all(&dir).unwrap();
		let mut tempname = dir.clone();
		let timestamp = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_millis();
		tempname.push(format!("{}.rs.{}", name, timestamp));
		let file = File::create(&tempname).unwrap();
		CodeGenCtx {
			file,
			name,
			index: 0,
			dir,
			tempname,
		}
	}

	fn finalize(self) {
		std::mem::drop(self.file);
		let mut path = self.dir;
		path.push(format!("{}.rs", self.name));
		if path.is_file() {
			if std::fs::remove_file(&path).is_err() {
				eprintln!("unable to replace file: {}", path.display());
				return;
			}
		}
		if std::fs::rename(&self.tempname, path).is_err() {
			eprintln!("unable to rename file: {}", self.tempname.display());
			return;
		}
	}
}

impl Type {
	fn to_tokens(&self) -> TokenStream {
		match self {
			Type::Int => {
				quote!(i32)
			},
			Type::Length => {
				quote!(ui::Length)
			},
			Type::Brush => {
				quote!(ui::Brush)
			},
			Type::String => {
				quote!(String)
			},
			Type::Boolean => {
				quote!(bool)
			},
			Type::Alignment => {
				quote!(ui::Alignment)
			},
			Type::Callback => {
				quote!(Callback)
			},
			Type::Iter(t) => {
				let t = t.to_tokens();
				quote!(ui::Iterable<#t>)
			},
			_ => unimplemented!("render values as tokens unimplemented for {:?}", self)
		}
	}
}

impl Value {
	fn to_tokens_optional(&self) -> TokenStream {
		match self {
			Value::Unset => {
				quote!(None)
			},
			_ => {
				let res = self.to_tokens();
				quote!(Some(#res))
			},
		}
	}
	fn to_tokens_move(&self) -> TokenStream {
		match self {
			Value::Px(n) => {
				quote!(ui::Length::Px(#n))
			},
			Value::Color(r, g, b, a) => {
				let r = *r as f32 / 255.0;
				let g = *g as f32 / 255.0;
				let b = *b as f32 / 255.0;
				let a = *a;
				quote!(ui::Color { r: #r, g: #g, b: #b, a: #a })
			},
			Value::Int(n) => {
				quote!(#n)
			},
			Value::String(s) => {
				quote!(#s.to_owned())
			},
			Value::Binding(Expr::Path(path, Ctx::Component)) => {
				let ident = format_ident!("{}", path.join("."));
				quote!(this.borrow().#ident)
			},
			Value::Binding(Expr::Path(path, Ctx::Repeater)) => {
				if path.len() > 1 {
					let ident = format_ident!("{}", path[1..].join("."));
					quote!(item.#ident)
				} else {
					quote!(item)
				}
			},
			_ => unimplemented!("render values as tokens unimplemented for {:?}", self)
		}
	}
	fn to_tokens_iter(&self) -> TokenStream {
		match self {
			Value::Binding(..) => {
				let tokens = self.to_tokens_move();
				quote!(#tokens.iter())
			},
			_ => {
				let tokens = self.to_tokens_move();
				quote!(ui::Iterable::iter(&#tokens))
			}
		}
	}
	fn to_tokens(&self) -> TokenStream {
		match self {
			Value::Binding(..) => {
				let tokens = self.to_tokens_move();
				quote!(ui::Convert::convert(&#tokens))
			},
			_ => {
				self.to_tokens_move()
			}
		}
	}
}

pub trait CodeGen {
	fn generate(&self, _element_data: ElementData, _ctx: &mut CodeGenCtx) -> TokenStream {
		quote!()
	}
}

impl CodeGen for Empty {}

impl CodeGen for Rect {
	fn generate(&self, _element_data: ElementData, _ctx: &mut CodeGenCtx) -> TokenStream {
		let x = self.x.to_tokens();
		let y = self.y.to_tokens();
		let width = self.width.to_tokens();
		let height = self.height.to_tokens();
		let background = self.background.to_tokens();
		quote!(
			let e_impl = ui::ElementImpl::Rect(
				ui::Rect {
					bounds: ui::Bounds {
						x: #x,
						y: #y,
						width: #width,
						height: #height,
					},
					color: #background,
				}
			);
		)
	}
}

impl CodeGen for Scroll {}

impl CodeGen for Span {
	fn generate(&self, _element_data: ElementData, _ctx: &mut CodeGenCtx) -> TokenStream {
		let x = self.x.to_tokens();
		let y = self.y.to_tokens();
		let color = self.color.to_tokens();
		let max_width = self.max_width.to_tokens_optional();
		quote!(
			let e_impl = ui::ElementImpl::Span(
				ui::Span {
					x: #x,
					y: #y,
					max_width: #max_width,
					color: #color,
				}
			);
		)
		// let mut span = HtmlElement::new("span", &e);
		// let fit_content = Value::String(String::from("fit-content"));
		// span.style.width = if let AddedProperties::Layout(layout) = e.added_properties {
		// 	if layout.column {
		// 		fit_content.clone()
		// 	} else {
		// 		Value::Unset
		// 	}
		// } else {
		// 	fit_content.clone()
		// };
		// span.style.height = if let AddedProperties::Layout(layout) = e.added_properties {
		// 	if !layout.column {
		// 		fit_content.clone()
		// 	} else {
		// 		Value::Unset
		// 	}
		// } else {
		// 	fit_content.clone()
		// };
		// span.style.max_width = self.max_width.clone();
		// span.style.color = self.color.clone();
		// span.style.padding = self.padding.clone();
		// span.style.white_space = Value::String(String::from("nowrap"));
		// span.position_children = "static";
		// span.display_children = "inline";
		// ctx.begin(span);
		// ctx.render_children(e.children);
		// ctx.end()
	}
}

impl CodeGen for Text {
	fn generate(&self, _element_data: ElementData, _ctx: &mut CodeGenCtx) -> TokenStream {
		let content = self.content.to_tokens();
		quote!(
			let e_impl = ui::ElementImpl::Text(
				ui::Text {
					content: #content,
				}
			);
		)
	}
}

impl CodeGen for ComponentInstance {}

impl CodeGen for Layout {}
