use std::fmt;

use crate::{Value, Result};
use crate::result::{Error, ValueError};

use crate::expressions::Expression;
use crate::Evaluation;

pub trait Function: fmt::Debug {
	fn exec<'a>(&self, eval: &Evaluation, args: Args<'a>) -> Result<Value>;
}


pub struct Args<'a>(&'a mut [Box<dyn Expression>]);

impl<'a> Args<'a> {
	pub fn new(args: &'a mut [Box<dyn Expression>]) -> Self {
		Self(args)
	}

	pub fn get_required(&mut self, index: usize) -> Result<&mut Box<dyn Expression>> {
		self.get_optional(index).ok_or(Error::MissingFuncArgument)
	}

	pub fn get_required_value(&mut self, index: usize, eval: &Evaluation) -> Result<Value> {
		self.get_required(index)?.next_eval(eval)?.ok_or(Error::UnableToFindValue)
	}

	pub fn get_required_optional_value(&mut self, index: usize, eval: &Evaluation) -> Result<Option<Value>> {
		self.get_required(index)?.next_eval(eval)
	}

	pub fn get_optional(&mut self, index: usize) -> Option<&mut Box<dyn Expression>> {
		self.0.get_mut(index)
	}

	pub fn as_array(&mut self) -> &mut [Box<dyn Expression>] {
		self.0
	}
}


// Node Set Functions

// number last()
#[derive(Debug, Clone)]
pub struct Last;

impl Function for Last {
	fn exec<'a>(&self, eval: &Evaluation, _: Args<'a>) -> Result<Value> {
		Ok(Value::Number(eval.size as f64))
	}
}


// number position()
#[derive(Debug, Clone)]
pub struct Position;

impl Function for Position {
	fn exec<'a>(&self, eval: &Evaluation, _: Args<'a>) -> Result<Value> {
		Ok(Value::Number(eval.position as f64))
	}
}

// number count(node-set)
#[derive(Debug, Clone)]
pub struct Count;

impl Function for Count {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		let arg1 = args.get_required(0)?;

		let mut count = 0.0;

		while arg1.next_eval(eval)?.is_some() {
			count += 1.0;
		}

		Ok(Value::Number(count))
	}
}

// node-set id(object)

// string local-name(node-set?)
#[derive(Debug)]
pub struct LocalName;

impl Function for LocalName {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		if let Some(expr) = args.get_optional(0) {
			if let Some(node) = expr.next_eval(eval)? {
				let node = node.into_node()?;

				let qual = node.name().ok_or_else::<Error, _>(|| ValueError::Nodeset.into())?;

				return Ok(Value::String(qual.local.to_string()));
			}
		}

		Ok(Value::String(String::new()))
	}
}


// string namespace-uri(node-set?)
#[derive(Debug)]
pub struct NamespaceUri;

impl Function for NamespaceUri {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		if let Some(expr) = args.get_optional(0) {
			if let Some(node) = expr.next_eval(eval)? {
				let node = node.into_node()?;

				let qual = node.name().ok_or_else::<Error, _>(|| ValueError::Nodeset.into())?;
				return Ok(Value::String(qual.ns.to_string()));
			}
		}

		Ok(Value::String(String::new()))
	}
}

// string name(node-set?)
#[derive(Debug)]
pub struct Name;

impl Function for Name {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		if let Some(expr) = args.get_optional(0) {
			if let Some(node) = expr.next_eval(eval)? {
				let node = node.into_node()?;

				let qual = node.name().ok_or_else::<Error, _>(|| ValueError::Nodeset.into())?;

				let value = if let Some(mut prefix) = qual.prefix.map(|s| s.to_string()) {
					prefix += ":";
					prefix += &qual.local;

					prefix
				} else {
					qual.local.to_string()
				};

				return Ok(Value::String(value));
			}
		}

		Ok(Value::String(String::new()))
	}
}


// String Functions


// https://www.w3.org/TR/xpath-functions-31/#func-string
#[derive(Debug)]
pub struct ToString;

impl Function for ToString {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		let value = match args.get_required_value(0, eval)? {
			Value::Boolean(val) => val.to_string(),
			Value::Number(val) => val.to_string(),
			Value::String(val) => val,
			Value::Node(n) => format!("{n:?}") // TODO
		};

		Ok(Value::String(value))
	}
}

// string concat(string, string, string*)
#[derive(Debug)]
pub struct Concat;

impl Function for Concat {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		let mut concat_value = String::new();

		for expr in args.as_array() {
			// It's okay if we don't find the value. We will not insert anything.
			if let Some(value_eval) = expr.next_eval(eval)? {
				let string_value = value_eval.convert_to_string()?;

				concat_value.push_str(&string_value);
			}
		}

		Ok(Value::String(concat_value))
	}
}

// boolean starts-with(string, string)
#[derive(Debug)]
pub struct StartsWith;

impl Function for StartsWith {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		// Required since if the required value does not contain wanted result it will error with UnableToFindValue. We don't want an error. We want a Boolean(false).
		let (left, right) = match (args.get_required_optional_value(0, eval)?, args.get_required_optional_value(1, eval)?) {
			(Some(a), Some(b)) => (a, b),
			(None, None) |
			(None, Some(_)) |
			(Some(_), None) => return Ok(Value::Boolean(false))
		};

		let left_value = left.convert_to_string()?;
		let right_value = right.convert_to_string()?;

		Ok(Value::Boolean(left_value.starts_with(&right_value)))
	}
}

// https://www.w3.org/TR/xpath-functions-31/#func-contains
#[derive(Debug)]
pub struct Contains;

impl Function for Contains {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		// Required since if the required value does not contain wanted result it will error with UnableToFindValue. We don't want an error. We want a Boolean(false).
		let (left, right) = match (args.get_required_optional_value(0, eval)?, args.get_required_optional_value(1, eval)?) {
			(Some(a), Some(b)) => (a, b),
			(None, None) |
			(None, Some(_)) |
			(Some(_), None) => return Ok(Value::Boolean(false))
		};

		let left_value = left.convert_to_string()?;
		let right_value = right.convert_to_string()?;

		Ok(Value::Boolean(
			match (left_value, right_value) {
				(left, _) if left.is_empty() => false,
				(_, right) if right.is_empty() => true,

				(left, right) => left.contains(&right)
			}
		))
	}
}

// string substring-before(string, string)
#[derive(Debug)]
pub struct SubstringBefore;


impl Function for SubstringBefore {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		let left = args.get_required_value(0, eval)?;
		let right = args.get_required_value(1, eval)?;

		let left_value = left.convert_to_string()?;
		let right_value = right.convert_to_string()?;

		if right_value.is_empty() {
			Ok(Value::String(String::new()))
		} else {
			let start = left_value.find(&right_value).unwrap_or_default();

			Ok(Value::String(left_value.get(0..start).map(|v| v.to_string()).unwrap_or_default()))
		}
	}
}

// string substring-after(string, string)
#[derive(Debug)]
pub struct SubstringAfter;

impl Function for SubstringAfter {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		let left = args.get_required_value(0, eval)?;
		let right = args.get_required_value(1, eval)?;

		let left_value = left.convert_to_string()?;
		let right_value = right.convert_to_string()?;

		if right_value.is_empty() {
			Ok(Value::String(String::new()))
		} else {
			let start = left_value.find(&right_value).unwrap_or_default();

			Ok(Value::String(left_value.get(start + right_value.len()..).map(|v| v.to_string()).unwrap_or_default()))
		}
	}
}

// string substring(string, number, number?)
#[derive(Debug)]
pub struct Substring;

impl Function for Substring {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		let value_0 = args.get_required_value(0, eval)?;
		let value_1 = args.get_required_value(1, eval)?;

		let value_str = value_0.convert_to_string()?;

		let start = value_1.number()?.round().abs() as isize - 1;

		let end = args.get_optional(2)
			.and_then(|v| v.next_eval(eval).ok().flatten())
			.map(|v| v.number())
			.unwrap_or_else(|| Ok(value_str.len() as f64))?
			.round() as isize;

		let end = start + end;

		Ok(Value::String(value_str.get(start.min(0) as usize .. end.min(0) as usize).map(|v| v.to_string()).unwrap_or_default()))
	}
}

// number string-length(string?)
#[derive(Debug)]
pub struct StringLength;

impl Function for StringLength {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		if let Some(arg) = args.get_optional(0) {
			let value = arg.next_eval(eval)?.ok_or(Error::UnableToFindValue)?;

			let value_str = value.convert_to_string()?;

			Ok(Value::Number(value_str.len() as f64))
		} else {
			Ok(Value::Number(0.0))
		}
	}
}

// string normalize-space(string?)
#[derive(Debug)]
pub struct NormalizeSpace;

impl Function for NormalizeSpace {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		match args.get_optional(0) {
			Some(expr) => {
				let value = expr.next_eval(eval)?.ok_or(Error::UnableToFindValue)?;

				let value_str = value.convert_to_string()?;

				Ok(Value::String(
					value_str.trim()
						.chars()
						.fold((String::new(), false), |(mut value, mut ignore_spaces), ch| {
							if ch.is_whitespace() {
								if !ignore_spaces {
									value.push(ch);
									ignore_spaces = true;
								}
							} else {
								value.push(ch);
								ignore_spaces = false;
							}

							(value, ignore_spaces)
						}).0
				))
			}

			_ => Ok(Value::String(String::new()))
		}
	}
}

// string translate(string, string, string)



// Boolean Functions
// boolean boolean(object)

// boolean not(boolean)
#[derive(Debug)]
pub struct Not;

impl Function for Not {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		let found = args.get_required_value(0, eval)?;
		Ok(Value::Boolean(!found.boolean()?))
	}
}


// boolean true()
#[derive(Debug)]
pub struct True;

impl Function for True {
	fn exec<'a>(&self, _: &Evaluation, _: Args<'a>) -> Result<Value> {
		Ok(Value::Boolean(true))
	}
}

// boolean false()
#[derive(Debug)]
pub struct False;

impl Function for False {
	fn exec<'a>(&self, _: &Evaluation, _: Args<'a>) -> Result<Value> {
		Ok(Value::Boolean(false))
	}
}

// boolean lang(string)

// Number Functions
// number number(object?)


// number sum(node-set)
#[derive(Debug, Clone)]
pub struct Sum;

impl Function for Sum {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		let values = args.get_required(0)?.collect(eval)?;

		let orig_len = values.len();

		let values = values.into_iter()
			.map(|n| {
				let node = n.into_node()?;
				let value = node.value()?;

				value.number()
			})
			.collect::<Result<Vec<f64>>>()?;

		if orig_len != values.len() {
			return Err(ValueError::Number.into());
		}

		Ok(Value::Number(values.into_iter().sum()))
	}
}

// number floor(number)
#[derive(Debug, Clone)]
pub struct Floor;

impl Function for Floor {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		let val = args.get_required_value(0, eval)?;

		let val = val.number()?;

		Ok(Value::Number(val.floor()))
	}
}

// number ceiling(number)
#[derive(Debug, Clone)]
pub struct Ceiling;

impl Function for Ceiling {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		let val = args.get_required_value(0, eval)?;

		let val = val.number()?;

		Ok(Value::Number(val.ceil()))
	}
}

// number round(number)
#[derive(Debug, Clone)]
pub struct Round;

impl Function for Round {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		let val = args.get_required_value(0, eval)?;

		let val = val.number()?;

		Ok(Value::Number(val.round()))
	}
}