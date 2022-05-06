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
	number::complete::{
		float,
	},
	multi::{
		many0,
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

use super::Value;

type ParseError<'a> = nom::Err<nom::error::Error<&'a str>>;

pub fn parse(input: &str) -> Result<Element, ParseError> {
	terminated(
		delimited(skip_space, element, skip_space),
		eof,
	)
	(input)
	.map(|(_, result)| result)
}

#[derive(Debug)]
pub struct Repeater<'a> {
	pub index: Option<&'a str>,
	pub item: &'a str,
	pub collection: Value,
}

#[derive(Debug)]
pub struct Element<'a> {
	pub path: Vec<&'a str>,
	pub condition: Option<Value>,
	pub repeater: Option<Repeater<'a>>,
	pub properties: HashMap<&'a str, Value>,
	pub event_handlers: HashMap<&'a str, Value>,
	pub children: Vec<Element<'a>>,
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
		acc.insert(p.name, p.value);
		acc
	});

	let event_handlers = event_handlers.into_iter().fold(HashMap::new(), |mut acc, p| {
		acc.insert(p.name, p.value);
		acc
	});

	Ok((input, Element {
		path,
		condition,
		repeater: repeater.map(|(i, e, c)| Repeater { index: i, item: e, collection: c }),
		properties,
		event_handlers,
		children,
	}))
}

fn text_content(input: &str) -> IResult<&str, Element> {
	let (input, result) = alt((string, binding))(input)?;
	Ok((input, Element {
		path: vec!["text"],
		condition: None,
		repeater: None,
		properties: hashmap!["content" => result],
		event_handlers: HashMap::new(),
		children: Vec::new()
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

fn content(input: &str) -> IResult<&str, Element> {
	alt((text_content, element))
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
				terminated(binding, skip_space),
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
	recognize(
		pair(
			satisfy(|c| is_alphabetic(c as u8) || c == '_'),
			many0(satisfy(|c| is_alphanumeric(c as u8) || c == '_'))
		)
	)(input)
}

fn value(input: &str) -> IResult<&str, Value> {
	alt((
		px,
		map(float, |e| Value::Float(e)),
		string,
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

fn string(input: &str) -> IResult<&str, Value> {
	delimited(
		char('"'),
		map(recognize(many0(satisfy(|c| c != '"'))), |e: &str| Value::String(e.to_owned())),
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

fn expr(input: &str) -> IResult<&str, &str> {
	name(input)
}

fn binding(input: &str) -> IResult<&str, Value> {
	delimited(
		pair(char('('), skip_space),
		map(expr, |e| Value::Binding(e.to_owned())),
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