use std::fs::File;
use std::io::Write;
use std::collections::HashMap;

use super::{
	Value,
	Element,
	Expr,
	Type,
	Direction,
	elements::{
		// Window,
		Rect,
		Text,
		Span,
		// Img,
		ElementData,
		Empty,
		Repeater,
		Layout,
		ComponentInstance,
		Content,
		Children,
	}
};

fn render_data_type(ctx: &mut WebRenderer, data_type: Type) {
	let ind = ctx.indent();
	match data_type {
		Type::Object(data_types) => {
			writeln!(ctx.file, "new w.UI.__types._Object({{").unwrap();
			ctx.indent += 1;
			for (s, t) in data_types {
				let ind = ctx.indent();
				write!(ctx.file, "{ind}{s}: ").unwrap();
				render_data_type(ctx, t);
				writeln!(ctx.file, ",").unwrap();
			}
			ctx.indent -= 1;
			write!(ctx.file, "{ind}}})").unwrap();
		},
		Type::Length => {
			write!(ctx.file, "w.UI.__types._Length").unwrap();
		},
		Type::Brush => {
			write!(ctx.file, "w.UI.__types._Brush").unwrap();
		},
		Type::Direction => {
			write!(ctx.file, "w.UI.__types._Direction").unwrap();
		},
		Type::String => {
			write!(ctx.file, "w.UI.__types._String").unwrap();
		},
		Type::Boolean => {
			write!(ctx.file, "w.UI.__types._Boolean").unwrap();
		},
		Type::Iter(t) => {
			write!(ctx.file, "new w.UI.__types._Iter(").unwrap();
			ctx.indent += 1;
			render_data_type(ctx, *t);
			ctx.indent -= 1;
			write!(ctx.file, ")").unwrap();
		},
		// _ => {
		// 	unimplemented!("rendering data type: {:?}", data_type);
		// }
	}
}

pub fn render<S: Into<String>>(root: &Element, name: S) {
	let mut ctx = WebRenderer::new(name);
	let root_html = root.render_web(&mut ctx).unwrap();

	writeln!(ctx.file, "(w => {{\n\
		w.UI.{} = (p, init, i=0, h=(() => null)) => {{\n\
			\tfunction update(d) {{\n\
				\t\tif(i == 0) {{\n\
				\t\t\tUI.__begin(p);\n\
				\t\t}}", ctx.name).unwrap();
	
	ctx.indent = 2;
	root_html.render_js(&mut ctx);

	write!(ctx.file, "\
			\t}}\n\
			\tlet d = new w.UI.__types._ObjectInstance(\n\t\t").unwrap();

	ctx.indent = 2;
	render_data_type(&mut ctx, Type::Object(root.data_types.clone()));
	
	writeln!(ctx.file, ",\n\
				\t\tinit,\n\
				\t\tupdate,\n\
			\t);\n\
			\tupdate(d);\n\
			\treturn d;\n\
		}};\n}})(window);").unwrap();
}

#[derive(Debug)]
pub enum HtmlContent {
	Children(Children),
	Component(HtmlComponent),
	Element(HtmlElement),
	Text(Value),
}

impl HtmlContent {
	fn render_js(&self, ctx: &mut WebRenderer) {
		match self {
			HtmlContent::Element(e) => e.render_js(ctx),
			HtmlContent::Component(c) => c.render_js(ctx),
			_ => unreachable!(),
		}
	}
}

#[derive(Debug)]
struct HtmlStyle {
	position: Value,
	display: Value,
	flex: Value,
	color: Value,
	background: Value,
	left: Value,
	top: Value,
	width: Value,
	height: Value,
	font_weight: Value,
	font_style: Value,
	flex_direction: Value,
}

impl Default for HtmlStyle {
	fn default() -> Self {
		HtmlStyle {
			position: Default::default(),
			display: Default::default(),
			flex: Value::String(String::from("1")),
			color: Default::default(),
			background: Default::default(),
			left: Default::default(),
			top: Default::default(),
			width: Default::default(),
			height: Default::default(),
			font_weight: Default::default(),
			font_style: Default::default(),
			flex_direction: Default::default(),
		}
	}
}

#[derive(Debug)]
pub struct HtmlComponent {
	name: String,
	properties: HashMap<String, Value>,
	content: Vec<HtmlContent>,
	repeater: Option<Repeater>,
	condition: Option<Value>,
}

impl Into<HtmlContent> for HtmlComponent {
	fn into(self) -> HtmlContent {
		HtmlContent::Component(self)
	}
}

impl HtmlComponent {
	fn new(name: &str, e: &ElementData, properties: &HashMap<String, Value>) -> Self {
		HtmlComponent {
			name: name.into(),
			properties: properties.clone(),
			content: Vec::new(),
			repeater: e.repeater.clone(),
			condition: e.condition.clone(),
		}
	}

	fn render_js(&self, ctx: &mut WebRenderer) {
		let ind = ctx.indent();
		if self.content.len() > 0 {
			writeln!(ctx.file, "{ind}let h = (e => {{").unwrap();
			ctx.indent += 1;
			render_content_js(&self.content, ctx);
			ctx.indent -= 1;
			writeln!(ctx.file, "{ind}}});").unwrap();
		} else {
			writeln!(ctx.file, "{ind}let h = (() => null);").unwrap();
		}
	
		let name = format!("UI.{}", self.name);
		let condition = self.condition.as_ref();
		let repeater = self.repeater.as_ref();
	
		let render = |ctx: &mut WebRenderer| {
			let ind = ctx.indent();
			for (k, v) in &self.properties {
				render_value_js(ctx, format!("{ind}e.__d.__changes.{k} = "), v, ";\n");
			}
			writeln!(ctx.file, "{ind}e.__d.commit();").unwrap();
		};
	
		render_outer_js(ctx, &name, condition, repeater, render);
	}
}

#[derive(Debug)]
pub struct HtmlElement {
	tag: &'static str,
	style: HtmlStyle,
	position_children: &'static str,
	display_children: &'static str,
	attrs: HashMap<String, Value>,
	content: Vec<HtmlContent>,
	repeater: Option<Repeater>,
	condition: Option<Value>,
	temporary_hacky_click_handler: Option<Value>,
}

impl Into<HtmlContent> for HtmlElement {
	fn into(self) -> HtmlContent {
		HtmlContent::Element(self)
	}
}

fn render_outer_js<F>(
	ctx: &mut WebRenderer,
	element_js: &str,
	condition: Option<&Value>,
	repeater: Option<&Repeater>,
	render: F)
	where F: FnOnce(&mut WebRenderer) {
	
	let mut ind = ctx.indent();

	if let Some(cond) = condition {
		render_value_js_coerce(ctx,
			format!("{ind}if("),
			cond,
			format!(") {{\n"),
			Coerce::AsPrimitive);
		ctx.indent += 1;
		ind = ctx.indent();
	}

	if let Some(Repeater { collection, .. }) = repeater {
		render_value_js_coerce(ctx,
			format!("{ind}UI.__beginGroup(p, i);\n{ind}for(let [i, item] of "),
			collection,
			format!(") {{\n{ind}\t(d => {{\n"),
			Coerce::AsIter);
		ctx.indent += 2;
		ind = ctx.indent();
	}

	writeln!(ctx.file, "{ind}let e = UI.__in(p, {}, i, null, h);", element_js).unwrap();

	render(ctx);

	if let Some(Repeater { index, item, .. }) = repeater {
		ctx.indent -= 2;
		ind = ctx.indent();
		let index = index.clone().map(|i| format!("{i}: i, ")).unwrap_or_default();
		writeln!(ctx.file, "\
			{ind}\t}})({{ __props: {{ ...d.__props, {index}{item}: item }} }});\n\
			{ind}}}\n\
			{ind}UI.__endGroup(p);").unwrap();
	}

	if condition.is_some() {
		ctx.indent -= 1;
		ind = ctx.indent();
		writeln!(ctx.file, "{ind}}} else {{\n{ind}\tUI.__out(p, {}, i, null, h);\n{ind}}}", element_js).unwrap();
	}
}

fn render_content_js(content: &[HtmlContent], ctx: &mut WebRenderer) {
	let ind = ctx.indent();
	for (i, item) in content.iter().enumerate() {
		match item {
			HtmlContent::Element(element) => {
				writeln!(ctx.file, "{ind}((p, d, i) => {{").unwrap();
				ctx.indent += 1;
				element.render_js(ctx);
				ctx.indent -= 1;
				writeln!(ctx.file, "{ind}}})(e, d, {i});").unwrap();
			},
			HtmlContent::Component(component) => {
				writeln!(ctx.file, "{ind}((p, d, i) => {{").unwrap();
				ctx.indent += 1;
				component.render_js(ctx);
				ctx.indent -= 1;
				writeln!(ctx.file, "{ind}}})(e, d, {i});").unwrap();
			},
			HtmlContent::Text(value) => {
				render_value_js_coerce(ctx, format!("{ind}UI.__in(e, null, {i}, "), value, ");\n", Coerce::AsPrimitive);
			},
			HtmlContent::Children(_) => {
				writeln!(ctx.file, "{ind}UI.__beginGroup(e, {i}); h(e); UI.__endGroup(e);").unwrap();
			},
		}
	}
}

impl HtmlElement {
	fn new(tag: &'static str, e: &ElementData) -> Self {
		HtmlElement {
			tag,
			position_children: "absolute",
			display_children: "block",
			style: HtmlStyle::default(),
			attrs: HashMap::new(),
			content: Vec::new(),
			repeater: e.repeater.clone(),
			condition: e.condition.clone(),
			temporary_hacky_click_handler: e.temporary_hacky_click_handler.clone(),
		}
	}

	fn render_js(&self, ctx: &mut WebRenderer) {
		let tag = format!("\"{}\"", self.tag);
		let condition = self.condition.as_ref();
		let repeater = self.repeater.as_ref();

		let render = |ctx: &mut WebRenderer| {
			let ind = ctx.indent();
			writeln!(ctx.file, "\
				{ind}e.__positionChildren = \"{}\";\n\
				{ind}e.__displayChildren = \"{}\";\n\
				{ind}d.parent = e.__ctx.parent;\n\
				{ind}d.self = e.__ctx;",
				self.position_children,
				self.display_children).unwrap();

			self.render_style_props(ctx);

			for (k, v) in self.attrs.iter() {
				render_value_js(ctx, format!("{ind}e.setAttribute(\"{k}\", "), v, ");\n");
			}

			if let Some(handler) = self.temporary_hacky_click_handler.as_ref() {
				render_value_js(ctx, format!("{ind}UI.__event(e, \"click\", d, "), handler, ");\n");
			}

			render_content_js(&self.content, ctx);
		};

		render_outer_js(ctx, &tag, condition, repeater, render);
	}

	fn render_style_props(&self, ctx: &mut WebRenderer) {
		let ind = ctx.indent();
		if self.style.position.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.position = "),
				&self.style.position,
				" ?? p.__positionChildren ?? \"absolute\";\n",
				Coerce::AsCss);
		} else {
			writeln!(ctx.file, "{ind}e.style.position = p.__positionChildren ?? \"absolute\";").unwrap();
		}
		if self.style.display.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.display = "),
				&self.style.display,
				" ?? p.__displayChildren ?? \"block\";\n",
				Coerce::AsCss);
		} else {
			writeln!(ctx.file, "{ind}e.style.display = p.__displayChildren ?? \"block\";").unwrap();
		}
		if self.style.flex.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.flex = "),
				&self.style.flex,
				";\n",
				Coerce::AsCss);
		}
		if self.style.flex_direction.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.flexDirection = "),
				&self.style.flex_direction,
				";\n",
				Coerce::AsCss);
		}
		if self.style.background.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.background = "),
				&self.style.background,
				";\n",
				Coerce::AsCss);
		}
		if self.style.left.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.left = "),
				&self.style.left,
				";\n",
				Coerce::AsCss);
		}
		if self.style.top.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.top = "),
				&self.style.top,
				";\n",
				Coerce::AsCss);
		}
		if self.style.width.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.width = "),
				&self.style.width,
				";\n",
				Coerce::AsCss);
		}
		if self.style.height.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.height = "),
				&self.style.height,
				";\n",
				Coerce::AsCss);
		}
		if self.style.font_weight.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.fontWeight = "),
				&self.style.font_weight,
				";\n",
				Coerce::AsCss);
		}
		if self.style.font_style.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.fontStyle = "),
				&self.style.font_style,
				";\n",
				Coerce::AsCss);
		}
		if self.style.color.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.color = "),
				&self.style.color,
				";\n",
				Coerce::AsCss);
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
	AsCss,
	AsIter,
	AsPrimitive
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
						write!(ctx.file, "d.__props.{}", path.join(".__props.")).unwrap();
					}
				}
			},
			Value::Color(r, g, b) => {
				write!(ctx.file, "{{ r: {r}, g: {g}, b: {b} }}").unwrap();
			},
			Value::Px(px) => {
				write!(ctx.file, "{{ length: {px}, unit: \"px\" }}").unwrap();
			},
			Value::Int(n) => {
				write!(ctx.file, "{}", n).unwrap();
			},
			_ => unimplemented!("RenderJs unimplemented for {:?}", self),
		}
	}

	fn render_js_coerce(&self, ctx: &mut WebRenderer, coercion: Coerce) {
		match coercion {
			Coerce::AsCss => {
				match self {
					Value::Binding(Expr::Path(path)) => {
						write!(ctx.file, "d.__props.{}.css()", path.join(".__props.")).unwrap();
					},
					Value::Color(r, g, b) => {
						write!(ctx.file, "\"rgb({r},{g},{b})\"").unwrap();
					},
					Value::Px(px) => {
						write!(ctx.file, "\"{px}px\"").unwrap();
					},
					Value::String(s) => {
						write!(ctx.file, "\"{s}\"").unwrap();
					},
					Value::Direction(d) => {
						match d {
							Direction::Horizontal => write!(ctx.file, "\"row\"").unwrap(),
							Direction::Vertical => write!(ctx.file, "\"column\"").unwrap(),
						}
					},
					_ => unimplemented!("RenderJs Coercion AsCss unimplemented for {:?}", self),
				}
			},
			Coerce::AsIter => {
				match self {
					Value::Binding(Expr::Path(path)) => {
						write!(ctx.file, "d.__props.{}.iter()", path.join(".__props.")).unwrap();
					},
					_ => unimplemented!("RenderJs Coercion AsIter unimplemented for {:?}", self),
				}
			},
			Coerce::AsPrimitive => {
				match self {
					Value::Float(..)|Value::Int(..)|Value::String(..)|Value::Boolean(..) => {
						self.render_js(ctx);
					},
					Value::Binding(Expr::Path(..)) => {
						self.render_js(ctx);
						write!(ctx.file, ".jsValue()").unwrap();
					},
					_ => unimplemented!("RenderJs Coercion AsIter unimplemented for {:?}", self),
				}
			},
		}
	}
}

pub struct WebRenderer {
	file: File,
	name: String,
	indent: u32,
	stack: Vec<HtmlContent>,
}

impl WebRenderer {
	pub fn new<S: Into<String>>(name: S) -> WebRenderer {
		let name = name.into();
		std::fs::create_dir_all("./dist").unwrap();
		let path = format!("./dist/{}.js", name);
		let file = File::create(path).unwrap();
		WebRenderer {
			file,
			name,
			indent: 0,
			stack: Vec::new(),
		}
	}

	fn indent(&self) -> String {
		(0..self.indent).map(|_| '\t').collect()
	}

	fn begin<T: Into<HtmlContent>>(&mut self, content: T) {
		self.stack.push(content.into());
	}

	fn end(&mut self) -> Option<HtmlContent> {
		let item = self.stack.pop().unwrap();
		if let Some(parent) = self.stack.last_mut() {
			match parent {
				HtmlContent::Element(e) => e.content.push(item),
				HtmlContent::Component(c) => c.content.push(item),
				_ => unreachable!(),
			}
			None
		} else {
			Some(item)
		}
	}

	fn append_content(&mut self, content: HtmlContent) {
		let parent = self.stack.last_mut().unwrap();
		match parent {
			HtmlContent::Element(e) => e.content.push(content),
			HtmlContent::Component(c) => c.content.push(content),
			_ => unreachable!(),
		}
	}

	fn render_children(&mut self, children: &Vec<Content>) {
		for child in children.iter() {
			match child {
				Content::Element(element) => {
					element.render_web(self);
				},
				Content::Children(children) => {
					self.append_content(HtmlContent::Children(children.clone()))
				},
			}
		}
	}
}

pub trait RenderWeb {
	fn render(&self, element_data: ElementData, ctx: &mut WebRenderer) -> Option<HtmlContent>;
}

// impl RenderWeb for Window {
// 	fn render(&self, ctx: &mut WebRenderer) -> Option<HtmlContent> {
// 		let mut body = HtmlElement::nnew("body");
// 		body.style.background = self.standard_props.background.clone();
// 		body.style.width = Value::String(String::from("100%"));
// 		body.style.height = Value::String(String::from("100%"));
// 		ctx.begin(body);
// 		ctx.render_children(&self.children);
// 		ctx.end()
// 	}
// }

impl RenderWeb for Rect {
	fn render(&self, e: ElementData, ctx: &mut WebRenderer) -> Option<HtmlContent> {
		let mut div = HtmlElement::new("div", &e);
		div.style.width = self.width.clone();
		div.style.height = self.height.clone();
		div.style.left = self.x.clone();
		div.style.top = self.y.clone();
		div.style.background = self.background.clone();

		ctx.begin(div);
		ctx.render_children(e.children);
		ctx.end()
	}
}

impl RenderWeb for Span {
	fn render(&self, e: ElementData, ctx: &mut WebRenderer) -> Option<HtmlContent> {
		let mut span = HtmlElement::new("span", &e);
		span.style.color = self.color.clone();
		span.position_children = "static";
		span.display_children = "inline";
		ctx.begin(span);
		ctx.render_children(e.children);
		ctx.end()
	}
}

impl RenderWeb for Text {
	fn render(&self, e: ElementData, ctx: &mut WebRenderer) -> Option<HtmlContent> {
		let using_span = ctx.stack.len() == 0;
		if using_span { ctx.begin(HtmlElement::new("span", &e)); }
		ctx.append_content(HtmlContent::Text(self.content.clone()));
		if using_span { ctx.end() } else { None }
	}
}

impl RenderWeb for ComponentInstance {
	fn render(&self, e: ElementData, ctx: &mut WebRenderer) -> Option<HtmlContent> {
		ctx.begin(HtmlComponent::new(&self.name, &e, &self.properties));
		ctx.render_children(e.children);
		ctx.end()
	}
}

impl RenderWeb for Empty {
	fn render(&self, _: ElementData, _: &mut WebRenderer) -> Option<HtmlContent> {
		None
	}
}

impl RenderWeb for Layout {
	fn render(&self, e: ElementData, ctx: &mut WebRenderer) -> Option<HtmlContent> {
		let mut div = HtmlElement::new("div", &e);
		div.style.left = self.x.clone();
		div.style.top = self.y.clone();
		div.style.width = self.width.clone();
		div.style.height = self.height.clone();
		div.style.display = Value::String("flex".into());
		div.style.flex_direction = self.direction.clone();
		div.position_children = "static";
		div.display_children = "block";
		
		ctx.begin(div);
		ctx.render_children(e.children);
		ctx.end()
	}
}

// impl RenderWeb for Img {
// 	fn render(&self, ctx: &mut WebRenderer) -> Option<HtmlContent> {
// 		let mut img = HtmlElement::nnew("img");
// 		img.style.width = self.standard_props.width.clone();
// 		img.style.height = self.standard_props.height.clone();
// 		img.style.left = self.standard_props.x.clone();
// 		img.style.top = self.standard_props.y.clone();
// 		img.attrs.insert("src".to_owned(), self.src.clone());
// 		ctx.empty_element(img)
// 	}
// }