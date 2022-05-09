use std::collections::HashMap;

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
		i32,
		none_of,
	},
	bytes::complete::{
		tag,
	},
	// number::complete::{
	// 	float,
	// },
	multi::{
		many0,
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
	},
	branch::{
		alt,
	},
};

use super::{Value, Expr, Import};

type ParseError<'a> = nom::Err<nom::error::Error<&'a str>>;

pub fn parse(input: &str) -> Result<Component, ParseError> {
	let (imports, element) = pair(
		imports,
		terminated(
			delimited(skip_space, element, skip_space),
			eof,
		),
	)
	(input)
	.map(|(_, result)| result)?;

	Ok(Component {
		name: String::new(),
		parse_tree: element,
		import_decls: imports,
		imports_map: HashMap::new(),
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
	pub condition: Option<Value>,
	pub repeater: Option<Repeater>,
	pub properties: HashMap<String, Value>,
	pub event_handlers: HashMap<String, Value>,
	pub children: Vec<Content>,
}

#[derive(Debug)]
pub struct Component {
	pub name: String,
	pub parse_tree: Element,
	pub import_decls: Vec<Import>,
	pub imports_map: HashMap<String, std::path::PathBuf>,
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
		|(path,alias)| Import { path: path.to_owned(), alias: alias.map(|e| e.to_owned()) }
	)
	(input)
}

fn imports(input: &str) -> IResult<&str, Vec<Import>> {
	many0(delimited(skip_space, import, skip_space))(input)
}

fn element(input: &str) -> IResult<&str, Element> {
	let (input, (path, condition, repeater, (properties, event_handlers, children))) = tuple((
		terminated(path, skip_space),
		opt(condition),
		opt(repeater),
		delimited(
			pair(skip_space, char('{')),
			tuple((
				many0(delimited(skip_space, property, skip_space)),
				many0(delimited(skip_space, event_handler, skip_space)),
				many0(delimited(skip_space, content, skip_space)),
			)),
			pair(skip_space, char('}')),
		)
	))
	(input)?;
	
	let properties = properties.into_iter().fold(HashMap::new(), |mut acc, p| {
		acc.insert(p.name.to_owned(), p.value);
		acc
	});

	let event_handlers = event_handlers.into_iter().fold(HashMap::new(), |mut acc, p| {
		acc.insert(p.name.to_owned(), p.value);
		acc
	});

	let path = path.into_iter().map(|e| e.to_owned()).collect();

	let repeater = repeater.map(|(i, e, c)| Repeater {
		index: i.map(|e| e.to_owned()),
		item: e.to_owned(),
		collection: c
	});

	Ok((input, Element {
		path,
		condition,
		repeater,
		properties,
		event_handlers,
		children,
	}))
}

fn text_content(input: &str) -> IResult<&str, Element> {
	let (input, result) = alt((
		map(string, |e: &str| Value::String(e.to_owned())),
		binding
	))
	(input)?;

	Ok((input, Element {
		path: vec!["text".to_owned()],
		condition: None,
		repeater: None,
		properties: hashmap!["content".to_owned() => result],
		event_handlers: HashMap::new(),
		children: Vec::new(),
	}))
}

#[derive(Debug)]
struct Property<'a> {
	name: &'a str,
	value: Value,
}

#[derive(Debug)]
struct EventHandler<'a> {
	name: &'a str,
	value: Value,
}

#[derive(Debug)]
pub struct Children {
	single: bool,
	filter: Option<Vec<String>>,
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

fn repeater(input: &str) -> IResult<&str, (Option<&str>, &str, Value)> {
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
	let (input, (name, value)) = property_or_partial_event_handler(input)?;
	Ok((input, Property {
		name,
		value,
	}))
}

fn event_handler(input: &str) -> IResult<&str, EventHandler> {
	let (input, (name, value)) = preceded(
		char('@'),
		property_or_partial_event_handler,
	)
	(input)?;
	Ok((input, EventHandler {
		name,
		value,
	}))
}

fn property_or_partial_event_handler(input: &str) -> IResult<&str, (&str, Value)> {
	terminated(
		separated_pair(name, delimited(skip_space, char(':'), skip_space), value),
		terminated(
			skip_space,
			alt((
				char(';'),
				peek(char('}')),
			)),
		)
	)
	(input)
}

fn path(input: &str) -> IResult<&str, Vec<&str>> {
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

fn name(input: &str) -> IResult<&str, &str> {
	preceded(
		not(
			terminated(
				alt((tag("import"), tag("as"), tag("if"), tag("for"), tag("in"))),
				not(alphanumeric1)
			)
		),
		recognize(
			pair(
				satisfy(|c| is_alphabetic(c as u8) || c == '_'),
				many0(satisfy(|c| is_alphanumeric(c as u8) || c == '_'))
			)
		)
	)
	(input)
}

fn value(input: &str) -> IResult<&str, Value> {
	alt((
		px,
		// map(float, |e| Value::Float(e)),
		int,
		map(string, |e: &str| Value::String(e.to_owned())),
		color,
		boolean,
		binding,
	))
	(input)
}

fn px(input: &str) -> IResult<&str, Value> {
	terminated(
		map(i32, |e| Value::Px(e)),
		tag("px"),
	)
	(input)
}

fn int(input: &str) -> IResult<&str, Value> {
	terminated(
		map(i32, |e| Value::Int(e)),
		not(alphanumeric1),
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
			Expr::Path(
				v.into_iter()
					.map(|e| e.to_owned())
					.collect()
				)
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