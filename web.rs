use std::fs::File;
use std::io::Write;
use std::collections::HashMap;

use super::{
	Value,
	Element,
	Expr,
	elements::{
		// Window,
		Rect,
		// PanesH,
		// PanesV,
		Text,
		Span,
		// ChildPropertySetter,
		// Img,
		ElementData,
		Empty,
		Repeater,
		Component,
	}
};

pub fn render<S: Into<String>>(root: Element, name: S) {
	let mut ctx = WebRenderer::new(name);
	let root = root.render_web(&mut ctx).unwrap();

	writeln!(ctx.file, "(w => {{\nw.Thing.{} = (p, d={{}}, i=0) => {{\n\tThing.__begin(p);", ctx.name).unwrap();
	
	ctx.indent = 1;
	root.render_js(&mut ctx);

	writeln!(ctx.file, "}};\n}})(window);").unwrap();
}

#[derive(Debug)]
enum HtmlContent {
	Element(HtmlElement),
	Text(Value),
}

#[derive(Default, Debug)]
struct HtmlStyle {
	position: Value,
	display: Value,
	background: Value,
	left: Value,
	top: Value,
	width: Value,
	height: Value,
	font_weight: Value,
	font_style: Value,
}

#[derive(Debug)]
pub struct HtmlElement {
	tag: &'static str,
	style: HtmlStyle,
	attrs: HashMap<String, Value>,
	content: Vec<HtmlContent>,
	repeater: Option<Repeater>,
	condition: Option<Value>,
	temporary_hacky_click_handler: Option<Value>,
}

impl HtmlElement {
	fn new(tag: &'static str, e: &ElementData) -> Self {
		let mut style = HtmlStyle::default();
		style.width = e.standard_props.width.clone();
		style.height = e.standard_props.height.clone();
		style.left = e.standard_props.x.clone();
		style.top = e.standard_props.y.clone();
		style.background = e.standard_props.background.clone();
		HtmlElement {
			tag,
			style,
			attrs: HashMap::new(),
			content: Vec::new(),
			repeater: e.repeater.clone(),
			condition: e.condition.clone(),
			temporary_hacky_click_handler: e.temporary_hacky_click_handler.clone(),
		}
	}

	fn render_js(&self, ctx: &mut WebRenderer) {
		let mut ind = ctx.indent();

		if let Some(cond) = self.condition.as_ref() {
			render_value_js(ctx,
				format!("{ind}if("),
				cond,
				format!(") {{\n"));
			ctx.indent += 1;
			ind = ctx.indent();
		}

		if let Some(Repeater { collection, .. }) = self.repeater.as_ref() {
			render_value_js(ctx,
				format!("{ind}Thing.__beginGroup(p, i);\n{ind}for(i=0; i<"),
				collection,
				format!("?.length; i++) {{\n{ind}\t(d => {{\n"));
			ctx.indent += 2;
			ind = ctx.indent();
		}
		
		writeln!(ctx.file, "{ind}let e = Thing.__in(p, \"{}\", i);", self.tag).unwrap();

		render_value_js_coerce(ctx, format!("{ind}e.style.position = "),   &self.style.position, ";\n",    Coerce::AsCss);
		render_value_js_coerce(ctx, format!("{ind}e.style.display = "),    &self.style.display, ";\n",     Coerce::AsCss);
		render_value_js_coerce(ctx, format!("{ind}e.style.background = "), &self.style.background, ";\n",  Coerce::AsCss);
		render_value_js_coerce(ctx, format!("{ind}e.style.left = "),       &self.style.left, ";\n",        Coerce::AsCss);
		render_value_js_coerce(ctx, format!("{ind}e.style.top = "),        &self.style.top, ";\n",         Coerce::AsCss);
		render_value_js_coerce(ctx, format!("{ind}e.style.width = "),      &self.style.width, ";\n",       Coerce::AsCss);
		render_value_js_coerce(ctx, format!("{ind}e.style.height = "),     &self.style.height, ";\n",      Coerce::AsCss);
		render_value_js_coerce(ctx, format!("{ind}e.style.fontWeight = "), &self.style.font_weight, ";\n", Coerce::AsCss);
		render_value_js_coerce(ctx, format!("{ind}e.style.fontStyle = "),  &self.style.font_style, ";\n",  Coerce::AsCss);

		for (k, v) in self.attrs.iter() {
			render_value_js(ctx, format!("{ind}e.setAttribute(\"{k}\", "), v, ");\n");
		}

		if let Some(handler) = self.temporary_hacky_click_handler.as_ref() {
			render_value_js(ctx, format!("{ind}Thing.__event(e, \"click\", d, "), handler, ");\n");
		}

		for (i, item) in self.content.iter().enumerate() {
			match item {
				HtmlContent::Element(element) => {
					writeln!(ctx.file, "{ind}((p, d, i) => {{").unwrap();
					ctx.indent += 1;
					element.render_js(ctx);
					ctx.indent -= 1;
					writeln!(ctx.file, "{ind}}})(e, d, {i});").unwrap();
				},
				HtmlContent::Text(value) => {
					render_value_js(ctx, format!("{ind}Thing.__in(e, null, {i}, "), value, ");\n");
				},
			}
		}

		if let Some(Repeater { index, item, collection }) = self.repeater.as_ref() {
			ctx.indent -= 2;
			ind = ctx.indent();
			let index = index.clone().map(|i| format!("{i}: i, ")).unwrap_or_default();
			render_value_js(ctx,
				format!("{ind}\t}})({{ ...d, {index}{item}: "),
				collection,
				format!("[i] ?? {{}} }});\n"));
			writeln!(ctx.file, "\
				{ind}}}\n\
				{ind}Thing.__endGroup(p, i);").unwrap();
		}

		if self.condition.is_some() {
			ctx.indent -= 1;
			ind = ctx.indent();
			writeln!(ctx.file, "{ind}}} else {{\n{ind}\tThing.__out(p, \"{}\", i);\n{ind}}}", self.tag).unwrap();
		}
	}
}

fn render_value_js<S: AsRef<str>, T: AsRef<str>>(ctx: &mut WebRenderer, before: S, value: &Value, after: T) {
	ctx.file.write(before.as_ref().as_bytes()).unwrap();
	value.render_js(ctx);
	ctx.file.write(after.as_ref().as_bytes()).unwrap();
}

fn render_value_js_coerce<S: AsRef<str>, T: AsRef<str>>(ctx: &mut WebRenderer, before: S, value: &Value, after: T, coercion: Coerce) {
	ctx.file.write(before.as_ref().as_bytes()).unwrap();
	value.render_js_coerce(ctx, coercion);
	ctx.file.write(after.as_ref().as_bytes()).unwrap();
}

#[derive(Debug, Clone, Copy)]
enum Coerce {
	NoCoercion,
	AsCss,
}

trait RenderJs {
	fn render_js(&self, ctx: &mut WebRenderer);
	fn render_js_coerce(&self, ctx: &mut WebRenderer, coercion: Coerce);
}

impl RenderJs for Value {
	fn render_js(&self, ctx: &mut WebRenderer) {
		match self {
			Value::String(s) => {
				let s = s.replace("\n", "\\n");
				write!(ctx.file, "\"{s}\"").unwrap();
			},
			Value::Binding(expr) => {
				match expr {
					Expr::Path(path) => {
						write!(ctx.file, "d?.{}", path.join(".")).unwrap();
					}
				}
			},
			Value::Color(r, g, b) => {
				write!(ctx.file, "{{ r: {r}, g: {g}, b: {b} }}").unwrap();
			},
			Value::Px(px) => {
				write!(ctx.file, "{{ length: {px}, unit: \"px\" }}").unwrap();
			},
			Value::Null => {
				write!(ctx.file, "null").unwrap();
			},
			_ => unimplemented!("RenderJs unimplemented for {:?}", self),
		}
	}

	fn render_js_coerce(&self, ctx: &mut WebRenderer, coercion: Coerce) {
		match coercion {
			Coerce::NoCoercion => {
				self.render_js(ctx);
			},
			Coerce::AsCss => {
				match self {
					Value::String(_) => {
						self.render_js(ctx);
					},
					Value::Binding(_) => {
						render_value_js(ctx, "String(", self, " ?? \"\")");
					},
					Value::Color(r, g, b) => {
						write!(ctx.file, "\"rgb({r},{g},{b})\"").unwrap();
					},
					Value::Px(px) => {
						write!(ctx.file, "\"{px}px\"").unwrap();
					},
					Value::Null => {
						write!(ctx.file, "\"\"").unwrap();
					},
					_ => unimplemented!("RenderJs Coercion AsCss unimplemented for {:?}", self),
				}
			}
		}
	}
}

pub struct WebRenderer {
	file: File,
	name: String,
	indent: u32,
	position: Vec<&'static str>,
	display: Vec<&'static str>,
	stack: Vec<HtmlElement>,
}

impl WebRenderer {
	pub fn new<S: Into<String>>(name: S) -> WebRenderer {
		let name = name.into();
		let path = format!("./{}.js", name);
		let file = File::create(path).unwrap();
		WebRenderer {
			file,
			name,
			indent: 0,
			position: vec!["absolute"],
			display: vec!["block"],
			stack: Vec::new(),
		}
	}

	fn indent(&self) -> String {
		(0..self.indent).map(|_| '\t').collect()
	}

	fn position(&self) -> String {
		self.position[self.position.len() - 1].to_owned()
	}

	fn display(&self) -> String {
		self.display[self.display.len() - 1].to_owned()
	}

	fn begin_element(&mut self, mut element: HtmlElement) {
		element.style.position = Value::String(self.position());
		element.style.display = Value::String(self.display());
		self.stack.push(element);
	}

	fn end_element(&mut self) -> Option<HtmlElement> {
		let element = self.stack.pop().unwrap();
		if let Some(parent) = self.stack.last_mut() {
			parent.content.push(HtmlContent::Element(element));
			None
		} else {
			Some(element)
		}
	}

	fn empty_element(&mut self, element: HtmlElement) -> Option<HtmlElement> {
		self.begin_element(element);
		self.end_element()
	}

	fn append_content(&mut self, content: HtmlContent) {
		let parent = self.stack.last_mut().unwrap();
		parent.content.push(content);
	}

	fn render_children(&mut self, children: &Vec<Element>) {
		self.indent += 1;
		for element in children.iter() {
			element.render_web(self);
		}
		self.indent -= 1;
	}
}

pub trait RenderWeb {
	fn render(&self, element_data: ElementData, ctx: &mut WebRenderer) -> Option<HtmlElement>;
}

// impl RenderWeb for Window {
// 	fn render(&self, ctx: &mut WebRenderer) -> Option<HtmlElement> {
// 		let mut body = HtmlElement::new("body");
// 		body.style.background = self.standard_props.background.clone();
// 		body.style.width = Value::String(String::from("100%"));
// 		body.style.height = Value::String(String::from("100%"));
// 		ctx.begin_element(body);
// 		ctx.render_children(&self.children);
// 		ctx.end_element()
// 	}
// }

impl RenderWeb for Rect {
	fn render(&self, e: ElementData, ctx: &mut WebRenderer) -> Option<HtmlElement> {
		ctx.begin_element(HtmlElement::new("div", &e));
		ctx.render_children(e.children);
		ctx.end_element()
	}
}
// impl RenderWeb for PanesH {
// 	fn render(&self, _ctx: &mut WebRenderer) -> Option<HtmlElement> {
// 		unimplemented!("RenderWeb unimplemented for `panes.h`");
// 	}
// }
// impl RenderWeb for PanesV {
// 	fn render(&self, _ctx: &mut WebRenderer) -> Option<HtmlElement> {
// 		unimplemented!("RenderWeb unimplemented for `panes.v`");
// 	}
// }

impl RenderWeb for Span {
	fn render(&self, e: ElementData, ctx: &mut WebRenderer) -> Option<HtmlElement> {
		let span = HtmlElement::new("span", &e);
		ctx.begin_element(span);
		ctx.position.push("static");
		ctx.display.push("inline");
		ctx.render_children(e.children);
		ctx.display.pop();
		ctx.position.pop();
		ctx.end_element()
	}
}

impl RenderWeb for Text {
	fn render(&self, e: ElementData, ctx: &mut WebRenderer) -> Option<HtmlElement> {
		let using_span = ctx.stack.len() == 0;
		if using_span { ctx.begin_element(HtmlElement::new("span", &e)); }
		ctx.append_content(HtmlContent::Text(self.content.clone()));
		if using_span { ctx.end_element() } else { None }
	}
}

impl RenderWeb for Component {
	fn render(&self, e: ElementData, ctx: &mut WebRenderer) -> Option<HtmlElement> {
		ctx.render_children(e.children);
		None
	}
}

impl RenderWeb for Empty {
	fn render(&self, _: ElementData, _: &mut WebRenderer) -> Option<HtmlElement> {
		None
	}
}

// impl RenderWeb for Each {
// 	fn render(&self, e: ElementData, ctx: &mut WebRenderer) -> Option<HtmlElement> {
// 		ctx.begin_element(HtmlElement::repeater("div", &e));
// 		ctx.render_children(e.children);
// 		ctx.end_element()
// 	}
// }

// impl RenderWeb for ChildPropertySetter {
// 	fn render(&self, ctx: &mut WebRenderer) -> Option<HtmlElement> {
// 		ctx.render_children(&self.children);
// 		None
// 	}
// }
// impl RenderWeb for Img {
// 	fn render(&self, ctx: &mut WebRenderer) -> Option<HtmlElement> {
// 		let mut img = HtmlElement::new("img");
// 		img.style.width = self.standard_props.width.clone();
// 		img.style.height = self.standard_props.height.clone();
// 		img.style.left = self.standard_props.x.clone();
// 		img.style.top = self.standard_props.y.clone();
// 		img.attrs.insert("src".to_owned(), self.src.clone());
// 		ctx.empty_element(img)
// 	}
// }