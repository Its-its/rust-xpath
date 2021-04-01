use std::fmt;

use crate::{Value, Result};
use crate::result::{Error, ValueError};

use crate::expressions::Expression;
use crate::{Evaluation, Nodeset};

pub trait Function: fmt::Debug {
	fn exec(&self, eval: &Evaluation) -> Result<Value>;
}


// Node Set Functions

// number last()
#[derive(Debug, Clone)]
pub struct Last;

impl Function for Last {
	fn exec(&self, eval: &Evaluation) -> Result<Value> {
		Ok(Value::Number(eval.size as f64))
	}
}


// number position()
#[derive(Debug, Clone)]
pub struct Position;

impl Function for Position {
	fn exec(&self, eval: &Evaluation) -> Result<Value> {
		Ok(Value::Number(eval.position as f64))
	}
}

// number count(node-set)
#[derive(Debug, Clone)]
pub struct Count(Nodeset);

impl Function for Count {
	fn exec(&self, _context: &Evaluation) -> Result<Value> {
		Ok(Value::Number(self.0.nodes.len() as f64))
	}
}

// node-set id(object)

// string local-name(node-set?)
#[derive(Debug)]
pub struct LocalName(Option<Box<dyn Expression>>);

impl Function for LocalName {
	fn exec(&self, eval: &Evaluation) -> Result<Value> {
		if let Some(expr) = self.0.as_ref() {
			let mut nodeset = expr.eval(eval)?.into_iterset()?;

			if let Some(node) = nodeset.next() {
				let qual = node.name().ok_or_else::<Error, _>(|| ValueError::Nodeset.into())?;

				return Ok(Value::String(qual.local.to_string()));
			}
		}

		Ok(Value::String(String::new()))
	}
}


// string namespace-uri(node-set?)
#[derive(Debug)]
pub struct NamespaceUri(Option<Box<dyn Expression>>);

impl Function for NamespaceUri {
	fn exec(&self, eval: &Evaluation) -> Result<Value> {
		if let Some(expr) = self.0.as_ref() {
			let mut nodeset = expr.eval(eval)?.into_iterset()?;

			if let Some(node) = nodeset.next() {
				let qual = node.name().ok_or_else::<Error, _>(|| ValueError::Nodeset.into())?;
				return Ok(Value::String(qual.ns.to_string()));
			}
		}

		Ok(Value::String(String::new()))
	}
}

// string name(node-set?)
#[derive(Debug)]
pub struct Name(Option<Box<dyn Expression>>);

impl Function for Name {
	fn exec(&self, eval: &Evaluation) -> Result<Value> {
		if let Some(expr) = self.0.as_ref() {
			let mut nodeset = expr.eval(eval)?.into_iterset()?;

			if let Some(node) = nodeset.next() {
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
// string string(object?)


// string concat(string, string, string*)
#[derive(Debug)]
pub struct Concat(Vec<Box<dyn Expression>>);

impl Concat {
	pub fn new(expr: Vec<Box<dyn Expression>>) -> Self {
		Self(expr)
	}
}

impl Function for Concat {
	fn exec(&self, eval: &Evaluation) -> Result<Value> {
		let mut value = String::new();

		for expr in &self.0 {
			let value_eval = expr.eval(eval)?;

			value.push_str(&value_eval.get_first_string()?);
		}

		Ok(Value::String(value))
	}
}

// boolean starts-with(string, string)
#[derive(Debug)]
pub struct StartsWith(Box<dyn Expression>, Box<dyn Expression>);

impl StartsWith {
	pub fn new(left: Box<dyn Expression>, right: Box<dyn Expression>) -> Self {
		Self(left, right)
	}
}

impl Function for StartsWith {
	fn exec(&self, eval: &Evaluation) -> Result<Value> {
		let left = self.0.eval(eval)?.get_first_string()?;
		let right = self.1.eval(eval)?.get_first_string()?;

		Ok(Value::Boolean(left.starts_with(&right)))
	}
}

// https://www.w3.org/TR/xpath-functions-31/#func-contains
#[derive(Debug)]
pub struct Contains(Box<dyn Expression>, Box<dyn Expression>);

impl Contains {
	pub fn new(left: Box<dyn Expression>, right: Box<dyn Expression>) -> Self {
		Self(left, right)
	}
}

impl Function for Contains {
	fn exec(&self, eval: &Evaluation) -> Result<Value> {
		let left = self.0.eval(eval)?.get_first_string()?;
		let right = self.1.eval(eval)?.get_first_string()?;

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
pub struct SubstringBefore(Box<dyn Expression>, Box<dyn Expression>);

impl SubstringBefore {
	pub fn new(left: Box<dyn Expression>, right: Box<dyn Expression>) -> Self {
		Self(left, right)
	}
}

impl Function for SubstringBefore {
	fn exec(&self, eval: &Evaluation) -> Result<Value> {
		let left = self.0.eval(eval)?.get_first_string()?;
		let right = self.1.eval(eval)?.get_first_string()?;

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
pub struct SubstringAfter(Box<dyn Expression>, Box<dyn Expression>);

impl SubstringAfter {
	pub fn new(left: Box<dyn Expression>, right: Box<dyn Expression>) -> Self {
		Self(left, right)
	}
}

impl Function for SubstringAfter {
	fn exec(&self, eval: &Evaluation) -> Result<Value> {
		let left = self.0.eval(eval)?.get_first_string()?;
		let right = self.1.eval(eval)?.get_first_string()?;

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
pub struct Substring(Box<dyn Expression>, Value, Option<Value>);

impl Substring {
	pub fn new(value: Box<dyn Expression>, start: Value, len: Option<Value>) -> Self {
		Self(value, start, len)
	}
}

impl Function for Substring {
	fn exec(&self, eval: &Evaluation) -> Result<Value> {
		let value = self.0.eval(eval)?.get_first_string()?;

		let start = self.1.number()?.round().abs() as isize - 1;

		let end = self.2.as_ref().map(|v| v.number()).unwrap_or_else(|| Ok(value.len() as f64))?.round() as isize;
		let end = start + end;

		let start = if start < 0 { 0 } else { start };
		let end = if end < 0 { 0 } else { end };

		Ok(Value::String(value.get(start as usize .. end as usize).map(|v| v.to_string()).unwrap_or_default()))
	}
}

// number string-length(string?)
#[derive(Debug)]
pub struct StringLength(Box<dyn Expression>);

impl StringLength {
	pub fn new(value: Box<dyn Expression>) -> Self {
		Self(value)
	}
}

impl Function for StringLength {
	fn exec(&self, eval: &Evaluation) -> Result<Value> {
		let value = self.0.eval(eval)?.get_first_string()?;

		Ok(Value::Number(value.len() as f64))
	}
}

// string normalize-space(string?)
#[derive(Debug)]
pub struct NormalizeSpace(Option<Box<dyn Expression>>);

impl NormalizeSpace {
	pub fn new(value: Option<Box<dyn Expression>>) -> Self {
		Self(value)
	}
}

impl Function for NormalizeSpace {
	fn exec(&self, eval: &Evaluation) -> Result<Value> {
		match self.0.as_ref() {
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
pub struct Not(Box<dyn Expression>);

impl Not {
	pub fn new(value: Box<dyn Expression>) -> Self {
		Not(value)
	}
}

impl Function for Not {
	fn exec(&self, eval: &Evaluation) -> Result<Value> {
		let found = self.0.eval(eval)?;
		Ok(Value::Boolean(!found.boolean()?))
	}
}


// boolean true()
#[derive(Debug)]
pub struct True;

impl Function for True {
	fn exec(&self, _: &Evaluation) -> Result<Value> {
		Ok(Value::Boolean(true))
	}
}

// boolean false()
#[derive(Debug)]
pub struct False;

impl Function for False {
	fn exec(&self, _: &Evaluation) -> Result<Value> {
		Ok(Value::Boolean(false))
	}
}

// boolean lang(string)

// Number Functions
// number number(object?)


// number sum(node-set)
#[derive(Debug, Clone)]
pub struct Sum(Value);

impl Function for Sum {
	fn exec(&self, _context: &Evaluation) -> Result<Value> {
		let node_set = self.0.as_nodeset()?;

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
pub struct Floor(Value);

impl Function for Floor {
	fn exec(&self, _context: &Evaluation) -> Result<Value> {
		let val = self.0.number()?;

		Ok(Value::Number(val.floor()))
	}
}

// number ceiling(number)
#[derive(Debug, Clone)]
pub struct Ceiling(Value);

impl Function for Ceiling {
	fn exec(&self, _context: &Evaluation) -> Result<Value> {
		let val = self.0.number()?;

		Ok(Value::Number(val.ceil()))
	}
}

// number round(number)
#[derive(Debug, Clone)]
pub struct Round(Value);

impl Function for Round {
	fn exec(&self, _context: &Evaluation) -> Result<Value> {
		let val = self.0.number()?;

		Ok(Value::Number(val.round()))
	}
}