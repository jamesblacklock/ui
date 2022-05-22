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
	let cond = if let Some(cond) = &e.condition {
		let cond = cond.to_tokens();
		quote!(
			if #cond {
                let mut e = ui::element_in(parent, e_impl, #index);
            } else {
                ui::element_out(parent, e_impl, #index);
            }
		)
	} else {
		quote!(
			let mut e = ui::element_in(parent, e_impl, #index);
		)
	};

	let mut children = Vec::new();
	for (i, child)in e.children.iter().enumerate() {
		ctx.index = i;
		match child {
			Content::Element(child) => children.push(render_element(child, ctx)),
			_ => unimplemented!()
		}
	}

	quote!(
		#parent
		#cond
		#(
			let mut e = {
				let parent = e;
				#children
				parent
			};
		)*
	)
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

	let fields = component.props.iter().map(|(name, decl)| {
		let name = format_ident!("{}", name);
		let prop_type = decl.prop_type.to_tokens();
		quote!(pub #name: #prop_type)
	});

	let web_code = if web {
		let target_struct_name = format_ident!("{}Target", struct_name);
		let interface_struct_name = format_ident!("{}Interface", struct_name);
		let props = component.props.iter().map(|(name, decl)| {
			let name = format_ident!("{}", name);
			let setter_name = format_ident!("set_{}", name);
			let prop_type = decl.prop_type.to_tokens();
			quote!(
				#[wasm_bindgen::prelude::wasm_bindgen(getter)]
				pub fn #name(&self) -> #prop_type {
					self.object.component.#name.clone()
				}
				#[wasm_bindgen::prelude::wasm_bindgen(setter)]
				pub fn #setter_name(&mut self, #name: #prop_type) {
					self.object.component.#name = #name;
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
					ui::web::RenderWeb::render(&mut self.root, &mut self.web_element, 0, true);
				}
			}
			#[wasm_bindgen::prelude::wasm_bindgen]
			#[repr(C)]
			pub struct #interface_struct_name {
				object: Box<#target_struct_name>,
			}
			#[wasm_bindgen::prelude::wasm_bindgen]
			impl #interface_struct_name {
				pub fn render(&mut self) {
					self.object.render();
				}
				#(#props)*
			}
			impl #struct_name {
				pub fn attach_to_element(self, e: &web_sys::Node) -> #interface_struct_name {
					let mut target = #target_struct_name::new(
						ui::web::WebElement::new(e.clone()),
						self
					);
					target.render();
					#interface_struct_name { object: Box::new(target) }
				}
			}
		)
	} else {
		quote!()
	};

	let code = quote!(
		#[allow(unused_mut, unused_variables)]
		mod #mod_name {
			use super::ui;
			pub struct #struct_name {
				#(#fields),*
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
				quote!(f32)
			},
			Type::Brush => {
				quote!(Brush)
			},
			Type::String => {
				quote!(String)
			},
			Type::Boolean => {
				quote!(bool)
			},
			Type::Alignment => {
				quote!(Alignment)
			},
			Type::Callback => {
				quote!(fn() -> ())
			},
			_ => unimplemented!("render values as tokens unimplemented for {:?}", self)
		}
	}
}

impl Value {
	fn optional_to_tokens(&self) -> TokenStream {
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
	fn to_tokens(&self) -> TokenStream {
		match self {
			Value::Px(n) => {
				quote!(#n)
			},
			Value::Color(r, g, b, a) => {
				let r = *r as f32 / 255.0;
				let g = *g as f32 / 255.0;
				let b = *b as f32 / 255.0;
				let a = *a;
				quote!(ui::Color { r: #r, g: #g, b: #b, a: #a })
			},
			Value::String(s) => {
				quote!(#s.to_owned())
			},
			Value::Binding(Expr::Path(path, Ctx::Component)) => {
				let ident = format_ident!("{}", path.join("."));
				quote!(self.#ident.clone())
			},
			_ => unimplemented!("render values as tokens unimplemented for {:?}", self)
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
		let max_width = self.max_width.optional_to_tokens();
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
