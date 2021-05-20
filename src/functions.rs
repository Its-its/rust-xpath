use std::fmt;

use crate::Result;
use crate::result::{Error, ValueError};
use crate::value::PartialValue;

use crate::expressions::Expression;
use crate::Evaluation;

pub trait Function: fmt::Debug {
	fn exec<'a>(&self, eval: &Evaluation, args: Args<'a>) -> Result<PartialValue>;
}


pub struct Args<'a>(&'a mut [Box<dyn Expression>]);

impl<'a> Args<'a> {
	pub fn new(args: &'a mut [Box<dyn Expression>]) -> Self {
		Self(args)
	}

	pub fn get_required(&mut self, index: usize) -> Result<&mut Box<dyn Expression>> {
		self.get_optional(index).ok_or(Error::MissingFuncArgument)
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
	fn exec<'a>(&self, eval: &Evaluation, _: Args<'a>) -> Result<PartialValue> {
		Ok(PartialValue::Number(eval.size as f64))
	}
}


// number position()
#[derive(Debug, Clone)]
pub struct Position;

impl Function for Position {
	fn exec<'a>(&self, eval: &Evaluation, _: Args<'a>) -> Result<PartialValue> {
		Ok(PartialValue::Number(eval.position as f64))
	}
}

// number count(node-set)
#[derive(Debug, Clone)]
pub struct Count;

impl Function for Count {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<PartialValue> {
		let len = args.get_required(0)?.count(eval)?;
		Ok(PartialValue::Number(len as f64))
	}
}

// node-set id(object)

// string local-name(node-set?)
#[derive(Debug)]
pub struct LocalName;

impl Function for LocalName {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<PartialValue> {
		if let Some(expr) = args.get_optional(0) {
			let value = expr.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;
			let node = value.into_node()?;

			let qual = node.name().ok_or_else::<Error, _>(|| ValueError::Nodeset.into())?;

			Ok(PartialValue::String(qual.local.to_string()))
		} else {
			Ok(PartialValue::String(String::new()))
		}
	}
}


// string namespace-uri(node-set?)
#[derive(Debug)]
pub struct NamespaceUri;

impl Function for NamespaceUri {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<PartialValue> {
		if let Some(expr) = args.get_optional(0) {
			let value = expr.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;
			let node = value.into_node()?;

			let qual = node.name().ok_or_else::<Error, _>(|| ValueError::Nodeset.into())?;

			Ok(PartialValue::String(qual.ns.to_string()))
		} else {
			Ok(PartialValue::String(String::new()))
		}
	}
}

// string name(node-set?)
#[derive(Debug)]
pub struct Name;

impl Function for Name {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<PartialValue> {
		if let Some(expr) = args.get_optional(0) {
			let value = expr.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;
			let node = value.into_node()?;

			let qual = node.name().ok_or_else::<Error, _>(|| ValueError::Nodeset.into())?;

			let value = if let Some(mut prefix) = qual.prefix.map(|s| s.to_string()) {
				prefix += ":";
				prefix += &qual.local;

				prefix
			} else {
				qual.local.to_string()
			};

			Ok(PartialValue::String(value))
		} else {
			Ok(PartialValue::String(String::new()))
		}
	}
}


// String Functions


// https://www.w3.org/TR/xpath-functions-31/#func-string
#[derive(Debug)]
pub struct ToString;

impl Function for ToString {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<PartialValue> {
		let value = match args.get_required(0)?.next_eval(eval)?.ok_or(Error::MissingFuncArgument)? {
			PartialValue::Boolean(val) => val.to_string(),
			PartialValue::Number(val) => val.to_string(),
			PartialValue::String(val) => val,
			PartialValue::Node(n) => format!("{:?}", n) // TODO
		};

		Ok(PartialValue::String(value))
	}
}

// string concat(string, string, string*)
#[derive(Debug)]
pub struct Concat;

impl Function for Concat {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<PartialValue> {
		let mut concat_value = String::new();

		for expr in args.as_array() {
			let value_eval = expr.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;

			let node = value_eval.into_node()?;

			let string_value = node.get_string_value()?;

			concat_value.push_str(&string_value);
		}

		Ok(PartialValue::String(concat_value))
	}
}

// boolean starts-with(string, string)
#[derive(Debug)]
pub struct StartsWith;

impl Function for StartsWith {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<PartialValue> {
		let left = args.get_required(0)?.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;
		let right = args.get_required(1)?.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;

		let left_node = left.into_node()?;
		let right_node = right.into_node()?;

		let left_value = left_node.get_string_value()?;
		let right_value = right_node.get_string_value()?;

		Ok(PartialValue::Boolean(left_value.starts_with(&right_value)))
	}
}

// https://www.w3.org/TR/xpath-functions-31/#func-contains
#[derive(Debug)]
pub struct Contains;

impl Function for Contains {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<PartialValue> {
		let left = args.get_required(0)?.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;
		let right = args.get_required(1)?.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;

		let left_node = left.into_node()?;
		let right_node = right.into_node()?;

		let left_value = left_node.get_string_value()?;
		let right_value = right_node.get_string_value()?;

		Ok(PartialValue::Boolean(
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
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<PartialValue> {
		let left = args.get_required(0)?.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;
		let right = args.get_required(1)?.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;

		let left_node = left.into_node()?;
		let right_node = right.into_node()?;

		let left_value = left_node.get_string_value()?;
		let right_value = right_node.get_string_value()?;


		if right_value.is_empty() {
			Ok(PartialValue::String(String::new()))
		} else {
			let start = left_value.find(&right_value).unwrap_or_default();

			Ok(PartialValue::String(left_value.get(0..start).map(|v| v.to_string()).unwrap_or_default()))
		}
	}
}

// string substring-after(string, string)
#[derive(Debug)]
pub struct SubstringAfter;

impl Function for SubstringAfter {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<PartialValue> {
		let left = args.get_required(0)?.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;
		let right = args.get_required(1)?.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;

		let left_node = left.into_node()?;
		let right_node = right.into_node()?;

		let left_value = left_node.get_string_value()?;
		let right_value = right_node.get_string_value()?;


		if right_value.is_empty() {
			Ok(PartialValue::String(String::new()))
		} else {
			let start = left_value.find(&right_value).unwrap_or_default();

			Ok(PartialValue::String(left_value.get(start + right_value.len()..).map(|v| v.to_string()).unwrap_or_default()))
		}
	}
}

// string substring(string, number, number?)
#[derive(Debug)]
pub struct Substring;

impl Function for Substring {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<PartialValue> {
		let value_0 = args.get_required(0)?.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;
		let value_1 = args.get_required(1)?.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;

		let node = value_0.into_node()?;

		let value_str = node.get_string_value()?;


		let start = value_1.as_number()?.round().abs() as isize - 1;

		let end = args.get_optional(2)
			.and_then(|v| v.next_eval(eval).ok().flatten())
			.map(|v| v.as_number())
			.unwrap_or_else(|| Ok(value_str.len() as f64))?
			.round() as isize;

		let end = start + end;

		let start = if start < 0 { 0 } else { start };
		let end = if end < 0 { 0 } else { end };

		Ok(PartialValue::String(value_str.get(start as usize .. end as usize).map(|v| v.to_string()).unwrap_or_default()))
	}
}

// number string-length(string?)
#[derive(Debug)]
pub struct StringLength;

impl Function for StringLength {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<PartialValue> {
		if let Some(arg) = args.get_optional(0) {
			let value = arg.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;

			let node = value.into_node()?;

			let value_str = node.get_string_value()?;

			Ok(PartialValue::Number(value_str.len() as f64))
		} else {
			Ok(PartialValue::Number(0.0))
		}
	}
}

// string normalize-space(string?)
#[derive(Debug)]
pub struct NormalizeSpace;

impl Function for NormalizeSpace {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<PartialValue> {
		match args.get_optional(0) {
			Some(expr) => {
				let value = expr.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;

				let node = value.into_node()?;

				let value_str = node.get_string_value()?;

				Ok(PartialValue::String(
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

			_ => Ok(PartialValue::String(String::new()))
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
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<PartialValue> {
		let found = args.get_required(0)?.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;
		Ok(PartialValue::Boolean(!found.as_boolean()?))
	}
}


// boolean true()
#[derive(Debug)]
pub struct True;

impl Function for True {
	fn exec<'a>(&self, _: &Evaluation, _: Args<'a>) -> Result<PartialValue> {
		Ok(PartialValue::Boolean(true))
	}
}

// boolean false()
#[derive(Debug)]
pub struct False;

impl Function for False {
	fn exec<'a>(&self, _: &Evaluation, _: Args<'a>) -> Result<PartialValue> {
		Ok(PartialValue::Boolean(false))
	}
}

// boolean lang(string)

// Number Functions
// number number(object?)


// number sum(node-set)
#[derive(Debug, Clone)]
pub struct Sum;

impl Function for Sum {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<PartialValue> {
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

		Ok(PartialValue::Number(values.into_iter().sum()))
	}
}

// number floor(number)
#[derive(Debug, Clone)]
pub struct Floor;

impl Function for Floor {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<PartialValue> {
		let val = args.get_required(0)?.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;

		let val = val.as_number()?;

		Ok(PartialValue::Number(val.floor()))
	}
}

// number ceiling(number)
#[derive(Debug, Clone)]
pub struct Ceiling;

impl Function for Ceiling {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<PartialValue> {
		let val = args.get_required(0)?.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;

		let val = val.as_number()?;

		Ok(PartialValue::Number(val.ceil()))
	}
}

// number round(number)
#[derive(Debug, Clone)]
pub struct Round;

impl Function for Round {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<PartialValue> {
		let val = args.get_required(0)?.next_eval(eval)?.ok_or(Error::MissingFuncArgument)?;

		let val = val.as_number()?;

		Ok(PartialValue::Number(val.round()))
	}
}