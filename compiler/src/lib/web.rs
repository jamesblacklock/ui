use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use super::{
	elements::{
		AddedProperties,
		Children,
		ComponentInstance,
		Content,
		// Img,
		ElementData,
		Empty,
		Layout,
		// Window,
		Rect,
		Repeater,
		Scroll,
		Span,
		Text,
	},
	Alignment,
	Component,
	Ctx,
	// Element,
	Expr,
	Type,
	Value,
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
		}
		Type::Length => {
			write!(ctx.file, "w.UI.__types._Length").unwrap();
		}
		Type::Brush => {
			write!(ctx.file, "w.UI.__types._Brush").unwrap();
		}
		Type::Alignment => {
			write!(ctx.file, "w.UI.__types._Alignment").unwrap();
		}
		Type::String => {
			write!(ctx.file, "w.UI.__types._String").unwrap();
		}
		Type::Boolean => {
			write!(ctx.file, "w.UI.__types._Boolean").unwrap();
		}
		Type::Iter(t) => {
			write!(ctx.file, "new w.UI.__types._Iter(\n{ind}\t").unwrap();
			ctx.indent += 1;
			render_data_type(ctx, *t);
			ctx.indent -= 1;
			write!(ctx.file, "\n{ind})").unwrap();
		}
		Type::Callback => {
			write!(ctx.file, "w.UI.__types._Callback").unwrap();
			// ctx.indent += 1;
			// // render_data_type(ctx, ???);
			// ctx.indent -= 1;
			// write!(ctx.file, ")").unwrap();
		}
		_ => {
			unimplemented!("rendering data type: {:?}", data_type);
		}
	}
}

pub fn render<S1: Into<String>, S2: Into<String>, P: Into<PathBuf>>(
	component: &Component,
	script: Option<S1>,
	name: S2,
	path: P,
) {
	let mut ctx = WebRenderer::new(name, path);
	let root_html = component.root.render_web(&mut ctx);

	writeln!(
		ctx.file,
		"(w => {{\n\
		w.UI.{} = (p, __init, i=0, h=(() => null)) => {{\n\
			\tfunction update(d) {{\n\
				\t\tif(i == 0) {{\n\
				\t\t\tw.UI.__begin(p);\n\
				\t\t\tw.UI.__ctx({{}}, p);\n\
				\t\t}}",
		ctx.name
	)
	.unwrap();
	ctx.indent = 2;
	if let Some(root_html) = root_html {
		root_html.render_js(&mut ctx);
	}

	writeln!(ctx.file, "\t}}").unwrap();

	if let Some(script) = script {
		for line in script.into().lines() {
			writeln!(ctx.file, "\t{}", line).unwrap();
		}
	} else {
		writeln!(ctx.file, "\tfunction init() {{ return {{}}; }}").unwrap();
	}

	write!(
		ctx.file,
		"\tlet d = new w.UI.__types._ObjectInstance(\n\t\t"
	)
	.unwrap();

	ctx.indent = 2;
	render_data_type(
		&mut ctx,
		Type::Object(
			component
				.props
				.iter()
				.fold(HashMap::new(), |mut map, (k, v)| {
					map.insert(k.clone(), v.prop_type.clone());
					map
				}),
		),
	);

	writeln!(
		ctx.file,
		",\n\
				\t\t{{ ...init(__init), ...__init }},\n\
				\t\tupdate,\n\
			\t);\n\
			\td.commit();\n\
			\treturn d;\n\
		}};\n}})(window);"
	)
	.unwrap();
	ctx.finalize();
}

fn render_added_properties(ctx: &mut WebRenderer, added_properties: &AddedProperties) {
	if let AddedProperties::Layout(item) = added_properties {
		let ind = ctx.indent();
		if item.stretch.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.flex = "),
				&item.stretch,
				";\n",
				Coerce::AsCss,
			);
		}
		render_value_js_coerce(
			ctx,
			format!("{ind}e.style.alignSelf = "),
			&item.align,
			";\n",
			Coerce::AsCss,
		);
		writeln!(ctx.file, "{ind}w.UI.__fixLayout(e, {});", item.grow).unwrap();
	}
}

fn render_outer_js<F>(
	ctx: &mut WebRenderer,
	element_js: &str,
	condition: Option<&Value>,
	repeater: Option<&Repeater>,
	render: F,
) where
	F: FnOnce(&mut WebRenderer),
{
	let mut ind = ctx.indent();

	if let Some(cond) = condition {
		render_value_js_coerce(
			ctx,
			format!("{ind}if("),
			cond,
			format!(") {{\n"),
			Coerce::AsPrimitive,
		);
		ctx.indent += 1;
		ind = ctx.indent();
	}

	if let Some(Repeater { collection, .. }) = repeater {
		render_value_js_coerce(
			ctx,
			format!("{ind}w.UI.__beginGroup(p, i);\n{ind}for(let [i, item] of "),
			collection,
			format!(") {{\n{ind}\t(d => {{\n"),
			Coerce::AsIter,
		);
		ctx.indent += 2;
		ind = ctx.indent();
	}

	writeln!(
		ctx.file,
		"{ind}let e = w.UI.__in(p, {element_js}, i, null, h);"
	)
	.unwrap();

	render(ctx);

	if let Some(Repeater { index, item, .. }) = repeater {
		ctx.indent -= 2;
		ind = ctx.indent();
		let index = index
			.clone()
			.map(|i| format!("{i}: i, "))
			.unwrap_or_default();
		writeln!(
			ctx.file,
			"\
			{ind}\t}})({{ __props: {{ ...d.__props, {index}{item}: item }} }});\n\
			{ind}}}\n\
			{ind}w.UI.__endGroup(p);"
		)
		.unwrap();
	}

	if condition.is_some() {
		ctx.indent -= 1;
		ind = ctx.indent();
		writeln!(
			ctx.file,
			"{ind}}} else {{\n{ind}\tw.UI.__out(p, {}, i, null, h);\n{ind}}}",
			element_js
		)
		.unwrap();
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
			}
			HtmlContent::Component(component) => {
				writeln!(ctx.file, "{ind}((p, d, i) => {{").unwrap();
				ctx.indent += 1;
				component.render_js(ctx);
				ctx.indent -= 1;
				writeln!(ctx.file, "{ind}}})(e, d, {i});").unwrap();
			}
			HtmlContent::Text(value) => {
				render_value_js_coerce(
					ctx,
					format!("{ind}w.UI.__in(e, null, {i}, "),
					value,
					");\n",
					Coerce::AsPrimitive,
				);
			}
			HtmlContent::Children(_) => {
				writeln!(
					ctx.file,
					"{ind}w.UI.__beginGroup(e, {i}); h(e); w.UI.__endGroup(e);"
				)
				.unwrap();
			}
		}
	}
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

#[derive(Debug, Default)]
struct HtmlEvents {
	click: Value,
	mousedown: Value,
	mouseup: Value,
	mouseenter: Value,
	mouseleave: Value,
	mousemove: Value,
}

#[derive(Debug, Default)]
struct HtmlStyle {
	position: Value,
	display: Value,
	// flex: Value,
	color: Value,
	background: Value,
	left: Value,
	top: Value,
	width: Value,
	max_width: Value,
	// max_height: Value,
	height: Value,
	font_weight: Value,
	font_style: Value,
	flex_direction: Value,
	overflow: Value,
	overflow_hidden: Value,
	padding: Value,
	white_space: Value,
	margin: Value,
}

#[derive(Debug)]
pub struct HtmlComponent {
	name: String,
	properties: HashMap<String, Value>,
	content: Vec<HtmlContent>,
	repeater: Option<Repeater>,
	condition: Option<Value>,
	added_properties: AddedProperties,
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
			added_properties: e.added_properties.clone(),
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
		let name = format!("w.UI.{}", self.name);
		let condition = self.condition.as_ref();
		let repeater = self.repeater.as_ref();
		let render = |ctx: &mut WebRenderer| {
			let ind = ctx.indent();
			for (k, v) in &self.properties {
				let assign_target = if let Value::Object(_) = v {
					format!("{ind}e.__d.__props.{k}.__changes = ")
				} else {
					format!("{ind}e.__d.__changes.{k} = ")
				};
				render_value_js_coerce(ctx, assign_target, v, ";\n", Coerce::AsRaw);
			}
			writeln!(ctx.file, "{ind}e.__d.commit(true);").unwrap();
			render_added_properties(ctx, &self.added_properties);
		};
		render_outer_js(ctx, &name, condition, repeater, render);
	}
}

#[derive(Debug)]
pub struct HtmlElement {
	tag: &'static str,
	style: HtmlStyle,
	events: HtmlEvents,
	position_children: &'static str,
	display_children: &'static str,
	space_children: Value,
	attrs: HashMap<String, Value>,
	content: Vec<HtmlContent>,
	repeater: Option<Repeater>,
	condition: Option<Value>,
	added_properties: AddedProperties,
	original_element_type: Option<String>,
}

impl Into<HtmlContent> for HtmlElement {
	fn into(self) -> HtmlContent {
		HtmlContent::Element(self)
	}
}

impl HtmlElement {
	fn new(tag: &'static str, e: &ElementData) -> Self {
		let events = HtmlEvents {
			click: e.events.pointer_click.clone(),
			mousemove: e.events.pointer_move.clone(),
			mousedown: e.events.pointer_press.clone(),
			mouseup: e.events.pointer_release.clone(),
			mouseenter: e.events.pointer_in.clone(),
			mouseleave: e.events.pointer_out.clone(),
			..HtmlEvents::default()
		};
		HtmlElement {
			tag,
			position_children: "absolute",
			display_children: "block",
			space_children: Default::default(),
			style: HtmlStyle::default(),
			events: events,
			attrs: HashMap::new(),
			content: Vec::new(),
			repeater: e.repeater.clone(),
			condition: e.condition.clone(),
			added_properties: e.added_properties.clone(),
			original_element_type: Some(e.tag.clone()),
		}
	}

	// fn plain(tag: &'static str) -> Self {
	// 	HtmlElement {
	// 		tag,
	// 		position_children: "absolute",
	// 		display_children: "block",
	// 		style: HtmlStyle::default(),
	// 		events: HtmlEvents::default(),
	// 		attrs: HashMap::new(),
	// 		content: Vec::new(),
	// 		repeater: None,
	// 		condition: None,
	// 		added_properties: AddedProperties::None,
	// 		original_element_type: None,
	// 	}
	// }

	fn render_js(&self, ctx: &mut WebRenderer) {
		let tag = format!("\"{}\"", self.tag);
		let condition = self.condition.as_ref();
		let repeater = self.repeater.as_ref();

		let render = |ctx: &mut WebRenderer| {
			let ind = ctx.indent();
			writeln!(
				ctx.file,
				"\
				{ind}e.__positionChildren = \"{}\";\n\
				{ind}e.__displayChildren = \"{}\";",
				self.position_children, self.display_children
			)
			.unwrap();

			self.render_style_props(ctx);
			self.render_event_props(ctx);

			for (k, v) in self.attrs.iter() {
				render_value_js(ctx, format!("{ind}e.setAttribute(\"{k}\", "), v, ");\n");
			}
			if let Some(t) = &self.original_element_type {
				writeln!(
					ctx.file,
					"{ind}e.setAttribute(\"element_type\", \"{}\");",
					t
				)
				.unwrap();
			}

			render_content_js(&self.content, ctx);
		};

		render_outer_js(ctx, &tag, condition, repeater, render);
	}

	fn render_event_props(&self, ctx: &mut WebRenderer) {
		let ind = ctx.indent();
		macro_rules! render_event_handler {
			($name:ident) => {
				if self.events.$name.is_set() {
					render_value_js_coerce(
						ctx,
						format!("{ind}w.UI.__event(e, \"{}\", d, ", stringify!($name)),
						&self.events.$name,
						");\n",
						Coerce::AsPrimitive,
					);
				}
			};
		}
		render_event_handler!(click);
		render_event_handler!(mousemove);
		render_event_handler!(mousedown);
		render_event_handler!(mouseup);
		render_event_handler!(mouseenter);
		render_event_handler!(mouseleave);
	}

	fn render_style_props(&self, ctx: &mut WebRenderer) {
		let ind = ctx.indent();
		writeln!(ctx.file, "{ind}e.style.boxSizing = \"border-box\";").unwrap();
		if self.space_children.is_set() {
			writeln!(ctx.file, "{ind}e.__spaceChildren = \"border-box\";").unwrap();
			render_value_js_coerce(
				ctx,
				format!("{ind}e.__spaceChildren = "),
				&self.space_children,
				";\n",
				Coerce::AsCss,
			);
		}
		if self.style.position.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.position = "),
				&self.style.position,
				" ?? p.__positionChildren ?? \"absolute\";\n",
				Coerce::AsCss,
			);
		} else {
			writeln!(
				ctx.file,
				"{ind}e.style.position = p.__positionChildren ?? \"absolute\";"
			)
			.unwrap();
		}
		if self.style.display.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.display = "),
				&self.style.display,
				" ?? p.__displayChildren ?? \"block\";\n",
				Coerce::AsCss,
			);
		} else {
			writeln!(
				ctx.file,
				"{ind}e.style.display = p.__displayChildren ?? \"block\";"
			)
			.unwrap();
		}
		if self.style.margin.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.margin = "),
				&self.style.margin,
				";\n",
				Coerce::AsCss,
			);
		}
		if self.style.flex_direction.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.flexDirection = "),
				&self.style.flex_direction,
				";\n",
				Coerce::AsCss,
			);
		}
		if self.style.background.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.background = "),
				&self.style.background,
				";\n",
				Coerce::AsCss,
			);
		}
		if self.style.left.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.left = "),
				&self.style.left,
				";\n",
				Coerce::AsCss,
			);
		}
		if self.style.top.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.top = "),
				&self.style.top,
				";\n",
				Coerce::AsCss,
			);
		}
		if self.style.width.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.width = "),
				&self.style.width,
				";\n",
				Coerce::AsCss,
			);
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.maxWidth = "),
				&self.style.width,
				";\n",
				Coerce::AsCss,
			);
		} else if self.style.max_width.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.maxWidth = "),
				&self.style.max_width,
				";\n",
				Coerce::AsCss,
			);
		}
		if self.style.height.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.height = "),
				&self.style.height,
				";\n",
				Coerce::AsCss,
			);
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.maxHeight = "),
				&self.style.height,
				";\n",
				Coerce::AsCss,
			);
		}
		if self.style.font_weight.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.fontWeight = "),
				&self.style.font_weight,
				";\n",
				Coerce::AsCss,
			);
		}
		if self.style.font_style.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.fontStyle = "),
				&self.style.font_style,
				";\n",
				Coerce::AsCss,
			);
		}
		if self.style.color.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.color = "),
				&self.style.color,
				";\n",
				Coerce::AsCss,
			);
		}
		if self.style.padding.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.padding = "),
				&self.style.padding,
				";\n",
				Coerce::AsCss,
			);
		}
		if self.style.white_space.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.whiteSpace = "),
				&self.style.white_space,
				";\n",
				Coerce::AsCss,
			);
		}
		if self.style.overflow_hidden.is_set() {
			render_value_js(
				ctx,
				format!("{ind}e.style.overflow = "),
				&self.style.overflow_hidden,
				" ? \"hidden\" : \"\";\n",
			);
		} else if self.style.overflow.is_set() {
			render_value_js_coerce(
				ctx,
				format!("{ind}e.style.overflow = "),
				&self.style.overflow,
				";\n",
				Coerce::AsCss,
			);
		}
		render_added_properties(ctx, &self.added_properties);
	}
}

fn render_value_js<S: AsRef<str>, T: AsRef<str>>(
	ctx: &mut WebRenderer,
	before: S,
	value: &Value,
	after: T,
) {
	ctx.file.write(before.as_ref().as_bytes()).unwrap();
	value.render_js(ctx);
	ctx.file.write(after.as_ref().as_bytes()).unwrap();
}

fn render_value_js_coerce<S: AsRef<str>, T: AsRef<str>>(
	ctx: &mut WebRenderer,
	before: S,
	value: &Value,
	after: T,
	coercion: Coerce,
) {
	ctx.file.write(before.as_ref().as_bytes()).unwrap();
	value.render_js_coerce(ctx, coercion);
	ctx.file.write(after.as_ref().as_bytes()).unwrap();
}

#[derive(Debug, Clone, Copy)]
enum Coerce {
	AsCss,
	AsIter,
	AsPrimitive,
	AsRaw,
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
			}
			Value::Binding(expr) => match expr {
				Expr::Path(path, prop_ctx) => {
					let base = match prop_ctx {
						Ctx::Component => "d.__props.",
						Ctx::Element => "e.__ctx.",
						Ctx::Parent => "e.__ctx.parent.",
						Ctx::Repeater => unimplemented!(),
					};
					write!(ctx.file, "{base}{}", path.join(".__props.")).unwrap();
				}
			},
			Value::Color(r, g, b, a) => {
				write!(ctx.file, "{{ r: {r}, g: {g}, b: {b}, a: {a} }}").unwrap();
			}
			Value::Px(px) => {
				write!(ctx.file, "{{ length: {px}, unit: \"px\" }}").unwrap();
			}
			Value::Int(n) => {
				write!(ctx.file, "{}", n).unwrap();
			}
			Value::Boolean(n) => {
				write!(ctx.file, "{}", n).unwrap();
			}
			_ => unimplemented!("RenderJs unimplemented for {:?}", self),
		}
	}

	fn render_js_coerce(&self, ctx: &mut WebRenderer, coercion: Coerce) {
		match coercion {
			Coerce::AsCss => match self {
				Value::Binding(Expr::Path(..)) => {
					self.render_js(ctx);
					write!(ctx.file, ".css()").unwrap();
				}
				Value::Color(r, g, b, a) => {
					write!(ctx.file, "\"rgba({r},{g},{b},{a})\"").unwrap();
				}
				Value::Px(px) => {
					write!(ctx.file, "\"{px}px\"").unwrap();
				}
				Value::String(s) => {
					write!(ctx.file, "\"{s}\"").unwrap();
				}
				Value::Float(f) => {
					write!(ctx.file, "\"{f}\"").unwrap();
				}
				Value::Int(i) => {
					write!(ctx.file, "\"{i}\"").unwrap();
				}
				Value::Alignment(a) => match a {
					Alignment::Stretch => write!(ctx.file, "\"stretch\"").unwrap(),
					Alignment::Start => write!(ctx.file, "\"start\"").unwrap(),
					Alignment::Center => write!(ctx.file, "\"center\"").unwrap(),
					Alignment::End => write!(ctx.file, "\"end\"").unwrap(),
				},
				_ => unimplemented!("RenderJs Coercion AsCss unimplemented for {:?}", self),
			},
			Coerce::AsIter => match self {
				Value::Binding(Expr::Path(..)) => {
					self.render_js(ctx);
					write!(ctx.file, ".iter()").unwrap();
				}
				_ => unimplemented!("RenderJs Coercion AsIter unimplemented for {:?}", self),
			},
			Coerce::AsRaw => match self {
				Value::Px(px) => {
					write!(ctx.file, "new w.UI.__types._Length(null, \"px\", {px})").unwrap()
				}
				Value::Float(f) => write!(ctx.file, "new w.UI.__types._Float(null, {f})").unwrap(),
				Value::Int(n) => write!(ctx.file, "new w.UI.__types._Int(null, {n})").unwrap(),
				Value::String(s) => {
					write!(ctx.file, "new w.UI.__types._String(null, \"{s}\")").unwrap()
				}
				Value::Boolean(b) => {
					write!(ctx.file, "new w.UI.__types._Boolean(null, {b})").unwrap()
				}
				Value::Color(r, g, b, a) => write!(
					ctx.file,
					"new w.UI.__types._Brush(null, \"color\", {{r:{r},g:{g},b:{b},a:{a}}})"
				)
				.unwrap(),
				Value::Object(map) => {
					write!(ctx.file, "{{ ").unwrap();
					for (k, v) in map {
						write!(ctx.file, "{k}: ").unwrap();
						v.render_js_coerce(ctx, Coerce::AsRaw);
						write!(ctx.file, ", ").unwrap();
					}
					write!(ctx.file, "}}").unwrap();
				}
				Value::Binding(Expr::Path(..)) => self.render_js(ctx),
				_ => unimplemented!("RenderJs Coercion AsRaw unimplemented for {:?}", self),
			},
			Coerce::AsPrimitive => match self {
				Value::Float(..) | Value::Int(..) | Value::String(..) | Value::Boolean(..) => {
					self.render_js(ctx);
				}
				Value::Binding(Expr::Path(..)) => {
					self.render_js(ctx);
					write!(ctx.file, ".jsValue()").unwrap();
				}
				_ => unimplemented!("RenderJs Coercion AsIter unimplemented for {:?}", self),
			},
		}
	}
}

pub struct WebRenderer {
	file: File,
	name: String,
	indent: u32,
	stack: Vec<HtmlContent>,
	dir: PathBuf,
	tempname: PathBuf,
}

impl WebRenderer {
	pub fn new<S: Into<String>, P: Into<PathBuf>>(name: S, dir: P) -> WebRenderer {
		let name = name.into();
		let dir = dir.into();
		std::fs::create_dir_all(&dir).unwrap();
		let mut tempname = dir.clone();
		let timestamp = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_millis();
		tempname.push(format!("{}.js.{}", name, timestamp));
		let file = File::create(&tempname).unwrap();
		WebRenderer {
			file,
			name,
			indent: 0,
			stack: Vec::new(),
			dir,
			tempname,
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
			self.render_child(child);
		}
	}
	fn render_child(&mut self, child: &Content) {
		match child {
			Content::Element(element) => {
				element.render_web(self);
			}
			Content::Children(children) => {
				self.append_content(HtmlContent::Children(children.clone()))
			}
		}
	}

	fn finalize(self) {
		std::mem::drop(self.file);
		let mut path = self.dir;
		path.push(format!("{}.js", self.name));
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

impl RenderWeb for Scroll {
	fn render(&self, e: ElementData, ctx: &mut WebRenderer) -> Option<HtmlContent> {
		let mut div = HtmlElement::new("div", &e);
		div.style.overflow = Value::String(String::from("auto"));
		div.style.width = self.width.clone();
		div.style.height = self.height.clone();
		div.style.left = self.x.clone();
		div.style.top = self.y.clone();

		ctx.begin(div);
		ctx.render_children(e.children);
		ctx.end()
	}
}

impl RenderWeb for Rect {
	fn render(&self, e: ElementData, ctx: &mut WebRenderer) -> Option<HtmlContent> {
		let mut div = HtmlElement::new("div", &e);
		div.style.overflow_hidden = self.clip.clone();
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
		let fit_content = Value::String(String::from("fit-content"));
		span.style.width = if let AddedProperties::Layout(layout) = e.added_properties {
			if layout.column {
				fit_content.clone()
			} else {
				Value::Unset
			}
		} else {
			fit_content.clone()
		};
		span.style.height = if let AddedProperties::Layout(layout) = e.added_properties {
			if !layout.column {
				fit_content.clone()
			} else {
				Value::Unset
			}
		} else {
			fit_content.clone()
		};
		span.style.max_width = self.max_width.clone();
		span.style.color = self.color.clone();
		span.style.padding = self.padding.clone();
		span.style.white_space = Value::String(String::from("nowrap"));
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
		if using_span {
			ctx.begin(HtmlElement::new("span", &e));
		}
		ctx.append_content(HtmlContent::Text(self.content.clone()));
		if using_span {
			ctx.end()
		} else {
			None
		}
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
		let direction = if self.column { "column" } else { "row" };
		let mut div = HtmlElement::new("div", &e);
		div.style.overflow_hidden = self.rect.clip.clone();
		div.style.background = self.rect.background.clone();
		div.style.padding = self.padding.clone();
		div.style.width = self.rect.width.clone();
		div.style.height = self.rect.height.clone();
		div.style.left = self.rect.x.clone();
		div.style.top = self.rect.y.clone();
		div.style.display = Value::String("flex".into());
		div.style.flex_direction = Value::String(direction.into());
		div.space_children = self.spacing.clone();
		div.position_children = "relative";
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
