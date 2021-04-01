use std::fmt;

use crate::{Value, Result};
use crate::result::ValueError;

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
// string namespace-uri(node-set?)
// string name(node-set?)



// String Functions
// string string(object?)
// string concat(string, string, string*)
// boolean starts-with(string, string)

// boolean contains(string, string)
#[derive(Debug)]
pub struct Contains(Box<dyn Expression>, Box<dyn Expression>);

impl Contains {
	pub fn new(left: Box<dyn Expression>, right: Box<dyn Expression>) -> Self {
		Contains(left, right)
	}
}

impl Function for Contains {
	fn exec(&self, eval: &Evaluation) -> Result<Value> {
		let right = self.1.eval(eval)?.into_nodeset()?;
		let left = self.0.eval(eval)?.into_nodeset()?;

		match (left.into_iter().next(), right.into_iter().next()) {
			(Some(left), Some(right)) => {
				let left_value = left.value()?.string()?;
				let right_value = left.value()?.string()?;

				Ok(Value::Boolean(left_value.contains(&right_value)))
			}

			_ => Ok(Value::Boolean(false))
		}
	}
}

// string substring-before(string, string)
// string substring-after(string, string)
// string substring(string, number, number?)
// number string-length(string?)
// string normalize-space(string?)
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