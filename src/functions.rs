use std::fmt;

use crate::{Result, Value, expressions::PartialValue};
use crate::result::{Error, ValueError};

use crate::expressions::Expression;
use crate::Evaluation;

pub trait Function: fmt::Debug {
	fn exec<'a>(&self, eval: &Evaluation, args: Args<'a>) -> Result<Value>;
}


pub struct Args<'a>(&'a mut [Box<dyn Expression>]);

impl<'a> Args<'a> {
	pub fn new(mut args: &'a mut [Box<dyn Expression>]) -> Self {
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
		let len = args.get_required(0)?.count();
		Ok(Value::Number(len as f64))
	}
}

// node-set id(object)

// string local-name(node-set?)
#[derive(Debug)]
pub struct LocalName;

impl Function for LocalName {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		if let Some(expr) = args.get_optional(0) {
			if let Some(node) = expr.eval(eval)?.into_nodeset()?.into_iter().next() {
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
			if let Some(node) = expr.eval(eval)?.into_nodeset()?.into_iter().next() {
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
			if let Some(node) = expr.eval(eval)?.into_nodeset()?.into_iter().next() {
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
		let value = match args.get_required(0)?.eval(eval)? {
			Value::Boolean(val) => val.to_string(),
			Value::Number(val) => val.to_string(),
			Value::String(val) => val,
			Value::Nodeset(n) => String::new() //format!("{:?}", n) // TODO
		};

		Ok(Value::String(value))
	}
}

// string concat(string, string, string*)
#[derive(Debug)]
pub struct Concat;

impl Function for Concat {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		let mut value = String::new();

		for expr in args.as_array() {
			let value_eval = expr.eval(eval)?;

			value.push_str(&value_eval.get_first_string()?);
		}

		Ok(Value::String(value))
	}
}

// boolean starts-with(string, string)
#[derive(Debug)]
pub struct StartsWith;

impl Function for StartsWith {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		let left = args.get_required(0)?.eval(eval)?.get_first_string()?;
		let right = args.get_required(1)?.eval(eval)?.get_first_string()?;

		Ok(Value::Boolean(left.starts_with(&right)))
	}
}

// https://www.w3.org/TR/xpath-functions-31/#func-contains
#[derive(Debug)]
pub struct Contains;

impl Function for Contains {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		let left = args.get_required(0)?.eval(eval)?.get_first_string()?;
		let right = args.get_required(1)?.eval(eval)?.get_first_string()?;

		Ok(Value::Boolean(
			match (left, right) {
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
		let left = args.get_required(0)?.eval(eval)?.get_first_string()?;
		let right = args.get_required(1)?.eval(eval)?.get_first_string()?;

		if right.is_empty() {
			Ok(Value::String(String::new()))
		} else {
			let start = left.find(&right).unwrap_or_default();

			Ok(Value::String(left.get(0..start).map(|v| v.to_string()).unwrap_or_default()))
		}
	}
}

// string substring-after(string, string)
#[derive(Debug)]
pub struct SubstringAfter;

impl Function for SubstringAfter {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		let left = args.get_required(0)?.eval(eval)?.get_first_string()?;
		let right = args.get_required(1)?.eval(eval)?.get_first_string()?;

		if right.is_empty() {
			Ok(Value::String(String::new()))
		} else {
			let start = left.find(&right).unwrap_or_default();

			Ok(Value::String(left.get(start + right.len()..).map(|v| v.to_string()).unwrap_or_default()))
		}
	}
}

// string substring(string, number, number?)
#[derive(Debug)]
pub struct Substring;

impl Function for Substring {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		let value = args.get_required(0)?.eval(eval)?.get_first_string()?;

		let start = args.get_required(1)?.eval(eval)?.number()?.round().abs() as isize - 1;

		let end = args.get_optional(2).and_then(|v| v.eval(eval).ok()).map(|v| v.number()).unwrap_or_else(|| Ok(value.len() as f64))?.round() as isize;
		let end = start + end;

		let start = if start < 0 { 0 } else { start };
		let end = if end < 0 { 0 } else { end };

		Ok(Value::String(value.get(start as usize .. end as usize).map(|v| v.to_string()).unwrap_or_default()))
	}
}

// number string-length(string?)
#[derive(Debug)]
pub struct StringLength;

impl Function for StringLength {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		if let Some(arg) = args.get_optional(0) {
			let value = arg.eval(eval)?.get_first_string()?;

			Ok(Value::Number(value.len() as f64))
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
				let value = expr.eval(eval)?.get_first_string()?;

				Ok(Value::String(
					value.trim()
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
		let found = args.get_required(0)?.eval(eval)?;
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
		let node_set = args.get_required(0)?.eval(eval)?.into_nodeset()?;

		let orig_len = node_set.len();

		let values = node_set.nodes.iter()
			.map(|n| n.value().and_then(|v| v.number()))
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
		let val = args.get_required(0)?.eval(eval)?.number()?;

		Ok(Value::Number(val.floor()))
	}
}

// number ceiling(number)
#[derive(Debug, Clone)]
pub struct Ceiling;

impl Function for Ceiling {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		let val = args.get_required(0)?.eval(eval)?.number()?;

		Ok(Value::Number(val.ceil()))
	}
}

// number round(number)
#[derive(Debug, Clone)]
pub struct Round;

impl Function for Round {
	fn exec<'a>(&self, eval: &Evaluation, mut args: Args<'a>) -> Result<Value> {
		let val = args.get_required(0)?.eval(eval)?.number()?;

		Ok(Value::Number(val.round()))
	}
}