use std::collections::HashMap;
use std::cell::Cell;
use std::path::PathBuf;

use nom::{
	IResult,
	character::{
		is_alphanumeric,
		is_alphabetic,
		is_hex_digit,
	},
	character::complete::{
		alphanumeric1,
		multispace1,
		satisfy,
		char,
		digit0,
		digit1,
		none_of,
		one_of,
	},
	bytes::complete::{
		tag,
	},
	multi::{
		many0,
		many1,
		separated_list1,
		many1_count,
	},
	sequence::{
		delimited,
		preceded,
		terminated,
		pair,
		separated_pair,
		tuple,
	},
	combinator::{
		eof,
		map,
		not,
		opt,
		peek,
		recognize,
		verify,
		fail,
	},
	branch::{
		alt,
	},
};

use super::{
	Alignment,
	Value,
	Expr,
	Import,
	Ctx,
	Type,
};

type ParseError<'a> = nom::Err<nom::error::Error<&'a str>>;

pub fn parse(input: &str) -> Result<Component, ParseError> {
	let (imports, prop_decls, element) = tuple((
		many0(delimited(skip_space, import, skip_space)),
		many0(delimited(skip_space, prop_decl, skip_space)),
		terminated(delimited(skip_space, element, skip_space), eof),
	))
	(input)
	.map(|(_, result)| result)?;

	if element.condition.is_some() || element.repeater.is_some() {
		fail::<_, &str, _>(input)?;
	}

	Ok(Component {
		name: String::new(),
		parse_tree: element,
		import_decls: imports,
		imports_map: HashMap::new(),
		status: Cell::new(CompileStatus::Ready),
	})
}

#[derive(Debug)]
pub struct Repeater {
	pub index: Option<String>,
	pub item: String,
	pub collection: Value,
}

#[derive(Debug)]
pub struct Element {
	pub path: Vec<String>,
	pub data: Option<Value>,
	pub condition: Option<Value>,
	pub repeater: Option<Repeater>,
	pub properties: HashMap<String, Value>,
	pub children: Vec<Content>,
}

#[derive(Debug)]
pub struct PropDecl {
	pub name: String,
	pub prop_type: Type,
	pub default: Option<Value>,
}

#[derive(Debug, Clone, Copy)]
pub enum CompileStatus {
	Ready,
	Building,
	Done,
}

#[derive(Debug)]
pub struct Component {
	pub name: String,
	pub parse_tree: Element,
	pub import_decls: Vec<Import>,
	pub imports_map: HashMap<String, std::path::PathBuf>,
	pub status: Cell<CompileStatus>,
}

fn import(input: &str) -> IResult<&str, Import> {
	map(
		terminated(
			pair(
				delimited(
					pair(tag("import"), skip_space),
					string,
					skip_space,
				),
				opt(
					delimited(
						pair(tag("as"), skip_space),
						name,
						skip_space
					),
				),
			),
			char(';')
		),
		|(path,alias)| Import { path: PathBuf::from(path), alias: alias.map(|e| e.to_owned()) }
	)
	(input)
}

fn add_property(map: &mut HashMap<String, Value>, path: &[String], value: Value) {
	if path.len() == 1 {
		if map.contains_key(&path[0]) {
			eprintln!("tried to assign property `{}` more than once", path[0]);
		} else {
			map.insert(path[0].clone(), value);
		}
	} else if let Some(Value::Object(map)) = map.get_mut(&path[0]) {
		add_property(map, &path[1..], value);
	} else {
		let mut new_map = HashMap::new();
		add_property(&mut new_map, &path[1..], value);
		map.insert(path[0].clone(), Value::Object(new_map));
	}
}

fn element(input: &str) -> IResult<&str, Element> {
	let (input, (path, data, condition, repeater, (properties, children))) = tuple((
		terminated(path, skip_space),
		opt(binding),
		opt(condition),
		opt(repeater),
		delimited(
			pair(skip_space, char('{')),
			tuple((
				many0(delimited(skip_space, property, skip_space)),
				many0(delimited(skip_space, content, skip_space)),
			)),
			pair(skip_space, char('}')),
		)
	))
	(input)?;
	
	let mut props_map = HashMap::new();
	for prop in properties.into_iter() {
		add_property(&mut props_map, &prop.path, prop.value);
	}

	let path = path.into_iter().map(|e| e.to_owned()).collect();

	let repeater = repeater.map(|(i, e, c)| Repeater {
		index: i.map(|e| e.to_owned()),
		item: e.to_owned(),
		collection: c
	});

	Ok((input, Element {
		path,
		data,
		condition,
		repeater,
		properties: props_map,
		children,
	}))
}

fn prop_decl(input: &str) -> IResult<&str, PropDecl> {
	let (input, (name, (prop_type, default))) = separated_pair(
		name,
		delimited(skip_space, char(':'), skip_space),
		prop_type,
	)
	(input)?;
	Ok((input, PropDecl {
		name,
		prop_type,
		default,
	}))
}

fn prop_type(input: &str) -> IResult<&str, (Type, Option<Value>)> {
	alt((
		pair(
			alt((
				map(tag("Length"),    |_| Type::Length),
				map(tag("Brush"),     |_| Type::Brush),
				map(tag("String"),    |_| Type::String),
				map(tag("Boolean"),   |_| Type::Boolean),
				map(tag("Alignment"), |_| Type::Alignment),
				map(
					delimited(
						pair(char('['), skip_space),
						prop_type,
						pair(skip_space, char(']'))
					),
					|t| Type::Iter(Box::new(t.0)),
				),
			)),
			terminated(
				opt(
					preceded(
						delimited(skip_space, char('='), skip_space),
						value,
					)
				),
				pair(
					skip_space,
					alt((
						char(';'),
						peek(char('}')),
					))
				),
			),
		),
		map(
			delimited(
				pair(char('{'), skip_space),
				many1(delimited(skip_space, prop_decl, skip_space)),
				pair(
					char('}'),
					pair(skip_space, opt(char(';')))
				),
			),
			|t| {
				let map = t.into_iter().fold(HashMap::new(), |mut acc, decl| {
					if acc.contains_key(&decl.name) {
						eprintln!("tried to redeclare property `{}`", decl.name);
					} else {
						acc.insert(decl.name, decl.prop_type);
					}
					acc
				});
				(Type::Object(map), None)
			},
		),
	))
	(input)
}

fn text_content(input: &str) -> IResult<&str, Element> {
	let (input, result) = alt((
		map(string, |e: &str| Value::String(e.to_owned())),
		binding
	))
	(input)?;

	Ok((input, Element {
		path: vec!["text".to_owned()],
		data: None,
		condition: None,
		repeater: None,
		properties: hashmap!["content".to_owned() => result],
		children: Vec::new(),
	}))
}

#[derive(Debug)]
struct Property {
	path: Vec<String>,
	value: Value,
}

#[derive(Debug, Clone)]
pub struct Children {
	pub single: bool,
	pub filter: Option<Vec<String>>,
}

#[derive(Debug)]
pub enum Content {
	Element(Element),
	Children(Children),
}

fn content(input: &str) -> IResult<&str, Content> {
	alt((
		map(text_content, |e| Content::Element(e)),
		map(element, |e| Content::Element(e)),
		map(children, |e| Content::Children(e)),
	))
	(input)
}

fn children(input: &str) -> IResult<&str, Children> {
	alt((
		map(
			delimited(
				tuple((
					tag("@child"),
					skip_space,
					char('('),
					skip_space,
				)),
				separated_list1(char(','), map(name, |e| e.to_owned())),
				pair(
					skip_space,
					char(')'),
				),
			),
			|filter| Children { single: true, filter: Some(filter) },
		),
		map(
			delimited(
				tuple((
					tag("@children"),
					skip_space,
					char('('),
					skip_space,
				)),
				separated_list1(char(','), map(name, |e| e.to_owned())),
				pair(
					skip_space,
					char(')'),
				),
			),
			|filter| Children { single: false, filter: Some(filter) },
		),
		map(
			terminated(tag("@child"), not(alphanumeric1)),
			|_| Children { single: true, filter: None },
		),
		map(
			terminated(tag("@children"), not(alphanumeric1)),
			|_| Children { single: false, filter: None },
		),
	))
	(input)
}

fn condition(input: &str) -> IResult<&str, Value> {
	preceded(
		terminated(tag("if"), skip_space),
		terminated(binding, skip_space),
	)
	(input)
}

fn repeater(input: &str) -> IResult<&str, (Option<String>, String, Value)> {
	preceded(
		terminated(tag("for"), skip_space),
		tuple((
			opt(terminated(name, delimited(skip_space, char(':'), skip_space))),
			terminated(name, skip_space),
			preceded(
				terminated(tag("in"), skip_space),
				terminated(value, skip_space),
			),
		))
	)
	(input)
}

fn property(input: &str) -> IResult<&str, Property> {
	let (input, (path, value)) = terminated(
		separated_pair(path, delimited(skip_space, char(':'), skip_space), value),
		terminated(
			skip_space,
			alt((
				char(';'),
				peek(char('}')),
			)),
		)
	)
	(input)?;
	Ok((input, Property {
		path,
		value,
	}))
}

fn path(input: &str) -> IResult<&str, Vec<String>> {
	let (input, (first, mut rest)) = pair(
		name,
		many0(
			preceded(char('.'), name)
		)
	)
	(input)?;
	rest.insert(0, first);
	Ok((input, rest))
}

fn name(input: &str) -> IResult<&str, String> {
	map(
		preceded(
			not(
				terminated(
					alt((tag("import"), tag("as"), tag("if"), tag("for"), tag("in"))),
					not(alphanumeric1)
				),
			),
			recognize(
				pair(
					satisfy(|c| is_alphabetic(c as u8) || c == '_'),
					many0(satisfy(|c| is_alphanumeric(c as u8) || c == '_'))
				),
			),
		),
		|e: &str| e.to_owned()
	)
	(input)
}

fn value(input: &str) -> IResult<&str, Value> {
	alt((
		px,
		map(float, |e| Value::Float(e)),
		map(int, |e| Value::Int(e)),
		map(string, |e: &str| Value::String(e.to_owned())),
		color,
		boolean,
		enum_value,
		binding,
	))
	(input)
}

fn float(input: &str) -> IResult<&str, f32> {
	map(
		recognize(
			pair(
				opt(one_of("+-")),
				alt((
					recognize(pair(
						digit0,
						pair(char('.'), digit1),
					)),
					recognize(digit1),
				)),
			),
		),
		|e| { str::parse(e).unwrap() }
	)
	(input)
}

fn enum_value(input: &str) -> IResult<&str, Value> {
	alt((
		map(tag(".stretch"),    |_| Value::Alignment(Alignment::Stretch)),
		map(tag(".center"),     |_| Value::Alignment(Alignment::Center)),
		map(tag(".start"),      |_| Value::Alignment(Alignment::Start)),
		map(tag(".end"),        |_| Value::Alignment(Alignment::End)),
	))
	(input)
}

fn px(input: &str) -> IResult<&str, Value> {
	terminated(
		map(float, |e| Value::Px(e)),
		tag("px"),
	)
	(input)
}

fn int(input: &str) -> IResult<&str, i32> {
	map(
		recognize(
			pair(opt(one_of("+-")), digit1),
		),
		|e| { str::parse(e).unwrap() }
	)
	(input)
}

fn string(input: &str) -> IResult<&str, &str> {
	delimited(
		char('"'),
		recognize(many0(satisfy(|c| c != '"'))),
		char('"')
	)
	(input)
}

fn color(input: &str) -> IResult<&str, Value> {
	let (input, value) = preceded(
		char('#'), 
		map(
			terminated(
				recognize(
					verify(
						many1_count(
							satisfy(|c| is_hex_digit(c as u8))
						),
						|&n| n == 3 || n == 6
					),
				),
				not(alphanumeric1),
			),
			|e| Value::color_from_hex(e)
		)
	)
	(input)?;
	Ok((input, value))
}

fn boolean(input: &str) -> IResult<&str, Value> {
	terminated(
		alt((
			map(tag("true"), |_| Value::Boolean(true)),
			map(tag("false"), |_| Value::Boolean(false)),
		)),
		not(alphanumeric1)
	)
	(input)
}

fn expr(input: &str) -> IResult<&str, Expr> {
	map(
		path,
		|v| {
			let mut it = v.into_iter().peekable();
			let ctx = match it.peek().unwrap().as_str() {
				"self"   => { it.next().unwrap(); Ctx::Element },
				"parent" => { it.next().unwrap(); Ctx::Parent },
				_        => Ctx::Component,
			};
			let path = it.map(|e| e.to_owned()).collect();
			Expr::Path(path, ctx)
		}
	)
	(input)
}

fn binding(input: &str) -> IResult<&str, Value> {
	delimited(
		pair(char('('), skip_space),
		map(expr, |e| Value::Binding(e)),
		pair(skip_space, char(')')),
	)
	(input)
}

fn multiline_comment(input: &str) -> IResult<&str, &str> {
	recognize(
		delimited(
			tag("/*"),
			recognize(
				many0(
					alt((
						multiline_comment,
						recognize(none_of("*")),
						recognize(pair(char('*'), not(char('/')))),
					)),
				)
			),
			tag("*/"),
		),
	)
	(input)
}

fn line_comment(input: &str) -> IResult<&str, &str> {
	recognize(
		preceded(
			tag("//"),
			many0(satisfy(|c| c != '\r' && c != '\n')),
		)
	)
	(input)
}

fn comment(input: &str) -> IResult<&str, &str> {
	alt((line_comment, multiline_comment))
	(input)
}

fn skip_space(input: &str) -> IResult<&str, &str> {
	recognize(many0(alt((multispace1, comment))))
	(input)
}