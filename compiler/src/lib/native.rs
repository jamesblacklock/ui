use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use quote::{quote, format_ident};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;

use super::{
	Value,
	Type,
	Ctx,
	Expr,
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

fn render_element(e: &Element, ctx: &mut NativeRenderer) -> TokenStream {
	let parent = RenderNative::render(e.element_impl.as_ref(), e.data(), ctx);

	let index = ctx.index;

	let mut children = Vec::new();
	for (i, child)in e.children.iter().enumerate() {
		ctx.index = i;
		match child {
			Content::Element(child) => children.push(render_element(child, ctx)),
			_ => unimplemented!()
		}
	}

	if let Some(repeater) = &e.repeater {
		let collection = repeater.collection.to_tokens_iter();
		let group = quote!(
			let parent = ui::begin_group(parent, #index);
			let mut i = 0;
			for item in #collection {
				#parent
				let e = ui::element_in(parent, e_impl, i);
				#(
					let e = {
						let parent = e;
						#children
						parent
					};
				)*
				i += 1;
			}
			ui::end_group(parent, i);
		);
		if let Some(cond) = &e.condition {
			let cond = cond.to_tokens();
			quote!(
				if #cond {
					#group
				} else {
					ui::element_out(parent, Box::new(ui::Group), #index);
				}
			)
		} else {
			group
		}
	} else {
		let body = quote!(
			let e = ui::element_in(parent, e_impl, #index);
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
					ui::element_out(parent, e_impl, #index);
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

pub fn render<S1: Into<String>, S2: Into<String>, P: Into<PathBuf>>(
	component: &Component,
	_script: Option<S1>,
	name: S2,
	path: P,
	web: bool,
) {
	let name = name.into();
	let mod_name = format_ident!("{}", name.clone().to_case(Case::Snake));
	let struct_name = format_ident!("{}", name.clone().to_case(Case::UpperCamel));

	let mut ctx = NativeRenderer::new(name, path);
	let code = render_element(&component.root, &mut ctx);

	let mut pub_fields = Vec::new();
	let mut pub_field_inits = Vec::new();
	let mut priv_fields = Vec::new();
	let mut priv_field_inits = Vec::new();
	for (name, decl) in component.props.iter() {
		let name = format_ident!("{}", name);
		let prop_type = decl.prop_type.to_tokens();
		if decl.is_pub {
			pub_fields.push(quote!(pub #name: #prop_type));
			pub_field_inits.push(quote!(#name: props.#name));
		} else {
			priv_fields.push(quote!(#name: #prop_type));
			priv_field_inits.push(quote!(#name: Default::default()));
		}
	}

	let web_code = if web {
		let target_struct_name = format_ident!("{}Target", struct_name);
		let interface_struct_name = format_ident!("{}Interface", struct_name);
		let props = component.props.iter().filter(|(_, decl)| decl.is_pub).map(|(name, _decl)| {
			let name = format_ident!("{}", name);
			let setter_name = format_ident!("set_{}", name);
			quote!(
				#[wasm_bindgen::prelude::wasm_bindgen(getter)]
				pub fn #name(&self) -> wasm_bindgen::JsValue {
					ui::web::ConvertJsValue::js_value(&self.object.component.#name)
				}
				#[wasm_bindgen::prelude::wasm_bindgen(setter)]
				pub fn #setter_name(&mut self, #name: wasm_bindgen::JsValue) {
					self.object.component.#name = ui::web::ConvertJsValue::from_js_value(#name);
					self.trigger_update();
				}
			)
		});
		quote!(
			struct #target_struct_name {
				component: #struct_name,
				web_element: ui::web::WebElement,
				root: ui::Element,
			}
			
			impl #target_struct_name {
				fn new(web_element: ui::web::WebElement, component: #struct_name) -> Self {
					Self {
						web_element,
						component,
						root: ui::Element::root(),
					}
				}
				fn render(&mut self) {
					ui::Component::update(&mut self.component, &mut self.root);
					self.web_element.last_in = None;
					ui::web::RenderWeb::render(&mut self.root, &mut self.web_element, 0, true);
				}
			}
			#[wasm_bindgen::prelude::wasm_bindgen]
			#[repr(C)]
			pub struct #interface_struct_name {
				object: Box<#target_struct_name>,
				animation_frame: i32,
			}
			use wasm_bindgen::JsCast;
			#[wasm_bindgen::prelude::wasm_bindgen]
			impl #interface_struct_name {
				pub fn render(&mut self) {
					self.object.render();
				}
				fn trigger_update(&mut self) {
					let window = web_sys::window().unwrap();
					window.cancel_animation_frame(self.animation_frame).unwrap();
					let ptr: *mut SimpleTarget = self.object.as_mut();
					self.animation_frame = window
						.request_animation_frame(
							wasm_bindgen::prelude::Closure::once_into_js(move || unsafe {
								(*ptr).render()
							})
							.as_ref()
							.unchecked_ref(),
						)
						.unwrap();
				}
				#(#props)*
			}
			impl #struct_name {
				pub fn attach_to_element(self, e: &web_sys::Node) -> #interface_struct_name {
					let mut target = #target_struct_name::new(
						ui::web::WebElement::new(Some(e.clone())),
						self
					);
					target.render();
					#interface_struct_name { object: Box::new(target), animation_frame: 0 }
				}
			}
		)
	} else {
		quote!()
	};

	let code = quote!(
		#[allow(unused_variables, dead_code)]
		mod #mod_name {
			use super::ui;
			pub struct #struct_name {
				#(#pub_fields,)*
				#(#priv_fields),*
			}
			pub struct Props {
				#(#pub_fields),*
			}
			impl #struct_name {
				pub fn new(props: Props) -> Self {
					Self {
						#(#pub_field_inits,)*
						#(#priv_field_inits),*
					}
				}
			}
			impl ui::Component for #struct_name {
				fn update(&mut self, parent: &mut ui::Element) {
					#code
				}
			}
			#web_code
		}
	);

	writeln!(ctx.file, "{code}").unwrap();

	ctx.finalize();
}

pub struct NativeRenderer {
	file: File,
	name: String,
	// indent: u32,
	// stack: Vec<HtmlContent>,
	dir: PathBuf,
	tempname: PathBuf,
	index: usize,
}

impl NativeRenderer {
	pub fn new<S: Into<String>, P: Into<PathBuf>>(name: S, dir: P) -> NativeRenderer {
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
		NativeRenderer {
			file,
			name,
			index: 0,
			// indent: 0,
			// stack: Vec::new(),
			dir,
			tempname,
		}
	}

	// fn indent(&self) -> String {
	// 	(0..self.indent).map(|_| '\t').collect()
	// }

	// fn begin<T: Into<HtmlContent>>(&mut self, content: T) {
	// 	self.stack.push(content.into());
	// }

	// fn end(&mut self) -> Option<HtmlContent> {
	// 	let item = self.stack.pop().unwrap();
	// 	if let Some(parent) = self.stack.last_mut() {
	// 		match parent {
	// 			HtmlContent::Element(e) => e.content.push(item),
	// 			HtmlContent::Component(c) => c.content.push(item),
	// 			_ => unreachable!(),
	// 		}
	// 		None
	// 	} else {
	// 		Some(item)
	// 	}
	// }

	// fn append_content(&mut self, content: HtmlContent) {
	// 	let parent = self.stack.last_mut().unwrap();
	// 	match parent {
	// 		HtmlContent::Element(e) => e.content.push(content),
	// 		HtmlContent::Component(c) => c.content.push(content),
	// 		_ => unreachable!(),
	// 	}
	// }

	// fn render_children(&mut self, children: &Vec<Content>) {
	// 	for child in children.iter() {
	// 		self.render_child(child);
	// 	}
	// }
	// fn render_child(&mut self, child: &Content) {
	// 	match child {
	// 		Content::Element(element) => {
	// 			element.render_web(self);
	// 		}
	// 		Content::Children(children) => {
	// 			self.append_content(HtmlContent::Children(children.clone()))
	// 		}
	// 	}
	// }

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

// trait ToTokens {
// 	fn to_tokens(&self) -> TokenStream;
// }

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
				quote!(fn() -> ())
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
				quote!(self.#ident)
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
				quote!(#tokens.clone())
			},
			_ => {
				self.to_tokens_move()
			}
		}
	}
}

pub trait RenderNative {
	fn render(&self, _element_data: ElementData, _ctx: &mut NativeRenderer) -> TokenStream {
		quote!()
	}
}

impl RenderNative for Empty {}

impl RenderNative for Rect {
	fn render(&self, _element_data: ElementData, _ctx: &mut NativeRenderer) -> TokenStream {
		let x = self.x.to_tokens();
		let y = self.y.to_tokens();
		let width = self.width.to_tokens();
		let height = self.height.to_tokens();
		let background = self.background.to_tokens();
		quote!(
			let e_impl = Box::new(
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

impl RenderNative for Scroll {}

impl RenderNative for Span {
	fn render(&self, _element_data: ElementData, _ctx: &mut NativeRenderer) -> TokenStream {
		let x = self.x.to_tokens();
		let y = self.y.to_tokens();
		let max_width = self.max_width.to_tokens_optional();
		quote!(
			let e_impl = Box::new(
				ui::Span {
					x: #x,
					y: #y,
					max_width: #max_width,
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

impl RenderNative for Text {
	fn render(&self, _element_data: ElementData, _ctx: &mut NativeRenderer) -> TokenStream {
		let content = self.content.to_tokens();
		quote!(
			let e_impl = Box::new(
				ui::Text {
					content: #content,
				}
			);
		)
	}
}

impl RenderNative for ComponentInstance {}

impl RenderNative for Layout {}
