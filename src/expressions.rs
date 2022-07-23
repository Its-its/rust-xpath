// 3 Expressions
// 	3.1 Basics
// 	3.2 Function Calls
// 	3.3 Node-sets
// 	3.4 Booleans
// 	3.5 Numbers
// 	3.6 Strings
// 	3.7 Lexical Structure


// Expression evaluation occurs with respect to a context.
// XSLT and XPointer specify how the context is determined for XPath expressions used in XSLT and XPointer respectively.
// The context consists of:
//     a node (the context node)
//     a pair of non-zero positive integers (the context position and the context size)
//     a set of variable bindings
//     a function library
//     the set of namespace declarations in scope for the expression
// The context position is always less than or equal to the context size.

// Expressions are parsed by first dividing the character string to be parsed into tokens and then parsing the resulting sequence of tokens.
// Whitespace can be freely used between tokens.
// The tokenization process is described in [3.7 Lexical Structure].

use std::{fmt, cell::RefCell};

use crate::{context::{NodeSearch, MoreNodes, FoundNode}, functions::{self, Args}, value::Value};
use crate::{AxisName, Evaluation, Node, NodeTest, Result};

pub type CallFunction = fn(ExpressionArg, ExpressionArg) -> ExpressionArg;
pub type ExpressionArg = Box<dyn Expression>;


pub trait Expression: fmt::Debug {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>>;

	fn count(&mut self, eval: &Evaluation) -> Result<usize> {
		let mut count = 0;

		while self.next_eval(eval)?.is_some() {
			count += 1;
		}

		Ok(count)
	}

	fn collect(&mut self, eval: &Evaluation) -> Result<Vec<Value>> {
		let mut nodes = Vec::new();

		while let Some(node) = self.next_eval(eval)? {
			nodes.push(node);
		}

		Ok(nodes)
	}

	// TODO: Better way
	fn once_wrapped(&self) -> bool {
		true
	}
}




macro_rules! res_opt_def_NAN {
	($val:expr) => {
		match $val? {
			Some(v) => v,
			None => return Ok(Some(Value::Number(f64::NAN)))
		}
	};
}

macro_rules! res_opt_def_false {
	($val:expr) => {
		match $val? {
			Some(v) => v,
			None => return Ok(Some(Value::Boolean(false)))
		}
	};
}


/// Insurance to make sure Addition, Subtration, etc. only run once if they're the first evaluation.
///
/// It'll run forever otherwise.
#[derive(Debug)]
pub struct Once(Option<ExpressionArg>);

impl Once {
	pub fn new(value: ExpressionArg) -> Self {
		Self(Some(value))
	}
}

impl Expression for Once {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
		self.0.take()
			.and_then(|mut v| v.next_eval(eval).transpose())
			.transpose()
	}
}


#[derive(Debug)]
pub struct Addition {
	left: ExpressionArg,
	right: ExpressionArg
}

impl Addition {
	pub fn new(left: ExpressionArg, right: ExpressionArg) -> Self {
		Self { left, right }
	}
}

impl Expression for Addition {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
		let left_value = res_opt_def_NAN!(self.left.next_eval(eval));
		let right_value = res_opt_def_NAN!(self.right.next_eval(eval));

		Ok(Some(Value::Number(left_value.as_number()? + right_value.as_number()?)))
	}
}


#[derive(Debug)]
pub struct Subtraction {
	left: ExpressionArg,
	right: ExpressionArg
}

impl Subtraction {
	pub fn new(left: ExpressionArg, right: ExpressionArg) -> Self {
		Self { left, right }
	}
}

impl Expression for Subtraction {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
		let left_value = res_opt_def_NAN!(self.left.next_eval(eval));
		let right_value = res_opt_def_NAN!(self.right.next_eval(eval));

		Ok(Some(Value::Number(left_value.as_number()? - right_value.as_number()?)))
	}
}


#[derive(Debug)]
pub struct LessThan {
	left: ExpressionArg,
	right: ExpressionArg
}

impl LessThan {
	pub fn new(left: ExpressionArg, right: ExpressionArg) -> Self {
		Self { left, right }
	}
}

impl Expression for LessThan {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
		let left_value = res_opt_def_false!(self.left.next_eval(eval));
		let right_value = res_opt_def_false!(self.right.next_eval(eval));

		Ok(Some(Value::Boolean(left_value.as_number()? < right_value.as_number()?)))
	}
}


#[derive(Debug)]
pub struct LessThanEqual {
	left: ExpressionArg,
	right: ExpressionArg
}

impl LessThanEqual {
	pub fn new(left: ExpressionArg, right: ExpressionArg) -> Self {
		Self { left, right }
	}
}

impl Expression for LessThanEqual {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
		let left_value = res_opt_def_false!(self.left.next_eval(eval));
		let right_value = res_opt_def_false!(self.right.next_eval(eval));

		Ok(Some(Value::Boolean(left_value.as_number()? <= right_value.as_number()?)))
	}
}


#[derive(Debug)]
pub struct GreaterThan {
	left: ExpressionArg,
	right: ExpressionArg
}

impl GreaterThan {
	pub fn new(left: ExpressionArg, right: ExpressionArg) -> Self {
		Self { left, right }
	}
}

impl Expression for GreaterThan {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
		let left_value = res_opt_def_false!(self.left.next_eval(eval));
		let right_value = res_opt_def_false!(self.right.next_eval(eval));

		Ok(Some(Value::Boolean(left_value.as_number()? > right_value.as_number()?)))
	}
}


#[derive(Debug)]
pub struct GreaterThanEqual {
	left: ExpressionArg,
	right: ExpressionArg
}

impl GreaterThanEqual {
	pub fn new(left: ExpressionArg, right: ExpressionArg) -> Self {
		Self { left, right }
	}
}

impl Expression for GreaterThanEqual {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
		let left_value = res_opt_def_false!(self.left.next_eval(eval));
		let right_value = res_opt_def_false!(self.right.next_eval(eval));

		Ok(Some(Value::Boolean(left_value.as_number()? >= right_value.as_number()?)))
	}
}


// Operations

#[derive(Debug)]
pub struct Equal {
	left: ExpressionArg,
	right: ExpressionArg
}

impl Equal {
	pub fn new(left: ExpressionArg, right: ExpressionArg) -> Self {
		Self { left, right }
	}
}

impl Expression for Equal {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
		let left_value = res_opt_def_false!(self.left.next_eval(eval));
		let right_value = res_opt_def_false!(self.right.next_eval(eval));

		Ok(Some(Value::Boolean(left_value == right_value)))
	}
}


#[derive(Debug)]
pub struct NotEqual {
	left: ExpressionArg,
	right: ExpressionArg
}

impl NotEqual {
	pub fn new(left: ExpressionArg, right: ExpressionArg) -> Self {
		Self { left, right }
	}
}

impl Expression for NotEqual {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
		let left_value = res_opt_def_false!(self.left.next_eval(eval));
		let right_value = res_opt_def_false!(self.right.next_eval(eval));

		Ok(Some(Value::Boolean(left_value != right_value)))
	}
}


#[derive(Debug)]
pub struct And {
	left: ExpressionArg,
	right: ExpressionArg
}

impl And {
	pub fn new(left: ExpressionArg, right: ExpressionArg) -> Self {
		Self { left, right }
	}
}

impl Expression for And {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
		let left_value = res_opt_catch!(self.left.next_eval(eval));
		let right_value = res_opt_catch!(self.right.next_eval(eval));

		Ok(Some(Value::Boolean(left_value.as_boolean()? && right_value.as_boolean()?)))
	}
}



#[derive(Debug)]
pub struct Or {
	left: ExpressionArg,
	right: ExpressionArg
}

impl Or {
	pub fn new(left: ExpressionArg, right: ExpressionArg) -> Self {
		Self { left, right }
	}
}

impl Expression for Or {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
		let left_value = self.left.next_eval(eval)?;
		let right_value = self.right.next_eval(eval)?;

		match (left_value, right_value) {
			(Some(value), None) |
			(None, Some(value)) => {
				Ok(Some(Value::Boolean(value.as_boolean()?)))
			}

			(Some(value1), Some(value2)) => {
				Ok(Some(Value::Boolean(value1.as_boolean()? || value2.as_boolean()?)))
			}

			_ => Ok(None)
		}
	}
}


// Primary Expressions

#[derive(Debug)]
pub struct Union {
	left: ExpressionArg,
	right: ExpressionArg,
	skip_left: bool
}

impl Union {
	pub fn new(left: ExpressionArg, right: ExpressionArg) -> Self {
		Self { left, right, skip_left: false }
	}
}

impl Expression for Union {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
		if !self.skip_left {
			let left_value = self.left.next_eval(eval)?;

			if left_value.is_some() {
				return Ok(left_value);
			}

			self.skip_left = true;
		}

		let right_value = self.right.next_eval(eval)?;

		if right_value.is_some() {
			return Ok(right_value);
		}

		Ok(None)
	}
}


#[derive(Debug)]
pub struct Literal(Value);

impl From<Value> for Literal {
	fn from(value: Value) -> Self {
		Literal(value)
	}
}

impl Expression for Literal {
	fn next_eval(&mut self, _: &Evaluation) -> Result<Option<Value>> {
		Ok(Some(self.0.clone()))
	}
}


// Nodeset

#[derive(Debug)]
pub struct RootNode;

impl Expression for RootNode {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
		Ok(Some(Value::Node(eval.root().clone())))
	}
}


#[derive(Debug)]
pub struct ContextNode;

impl Expression for ContextNode {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
		// TODO: Figure out. Cannot clone an Rc
		Ok(Some(Value::Node(eval.node.clone())))
	}
}


#[derive(Debug)]
pub struct Path {
	pub start_pos: ExpressionArg,
	pub steps: Vec<RefCell<Step>>,
	// TODO: Utilize NodeSearch::Group and add a Vec<> inside it to allow for nested calls.
	// Each nest should be placed into here. Example: these would be in here: ROOT, html, html/head, html/body, html/body/..., etc..
	pub search_groupings: Vec<NodeSearch>,
	pub steps_initiated: bool,
}

impl Path {
	pub fn new(start_pos: ExpressionArg, steps: Vec<Step>) -> Self {
		Self {
			start_pos,
			steps: steps.into_iter().map(RefCell::new).collect(),
			search_groupings: Vec::new(),
			steps_initiated: false,
		}
	}

	pub fn find_next_node_with_steps(&mut self, eval: &Evaluation) -> Result<Option<Node>> {
		while let Some(mut grouping) = self.search_groupings.pop() {
			let (found_node, append_states) = grouping.find_and_cache_next_node(eval, &self.steps)?;

			match found_node {
				MoreNodes::PassedPredicate(node) => {
					self.search_groupings.push(grouping);

					if let Some(mut append) = append_states {
						self.search_groupings.append(&mut append);
					}

					return Ok(Some(node));
				}

				MoreNodes::No => (),// println!("find_next_node_with_steps() -> MoreNodes::No"),

				MoreNodes::Possible |
				MoreNodes::FailedPredicate => {
					// println!("find_next_node_with_steps() -> MoreNodes::Possible | MoreNodes::FailedPredicate");

					self.search_groupings.push(grouping);
				}

				MoreNodes::Found(node) => unreachable!("find_next_node_with_steps: {:?}", node.as_simple_html()),
			}

			if let Some(mut append) = append_states {
				self.search_groupings.append(&mut append);
			}
		}

		Ok(None)
	}

	pub fn find_next_node(&mut self, eval: &Evaluation) -> Result<Option<Node>> {
		if let Some(node) = self.find_next_node_with_steps(eval)? {
			Ok(Some(node))
		} else {
			let found = res_opt_catch!(self.start_pos.next_eval(eval)).into_node()?;

			// Here to ensure we don't loop back around.
			if &found == eval.starting_eval_node {
				if self.steps_initiated {
					return Ok(None);
				}

				self.steps_initiated = true;
			}

			// Creates self.search_steps from self.steps.
			if let Some(step) = self.steps.first() {
				self.search_groupings.push(NodeSearch::new(step.borrow().axis, found, 0));
			}

			self.find_next_node_with_steps(eval)
		}
	}
}

impl Expression for Path {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
		Ok(Some(Value::Node(res_opt_catch!(self.find_next_node(eval)))))
	}

	fn once_wrapped(&self) -> bool {
		false
	}
}


#[derive(Debug)]
pub struct Step {
	pub axis: AxisName,
	pub node_test: Box<dyn NodeTest>, // A Step Test
	predicates: Vec<Predicate>
}

impl Step {
	pub fn new(
		axis: AxisName,
		node_test: Box<dyn NodeTest>,
		predicates: Vec<ExpressionArg>,
	) -> Step {
		let preds = predicates
			.into_iter()
			.map(Predicate)
			.collect();

		Step {
			axis,
			node_test,
			predicates: preds
		}
	}

	pub fn evaluate(
		&mut self,
		context: &Evaluation,
		found_node: FoundNode,
	) -> Result<MoreNodes<Node>> {
		let eval = context.new_evaluation_from_with_pos(&found_node.node, found_node.position);
		// eval.is_last_node = state.is_finished();
		// TODO: Fix

		for predicate in &mut self.predicates {
			if let Some(false) = predicate.matches_eval(&eval)? {
				return Ok(MoreNodes::FailedPredicate);
			}
		}

		Ok(MoreNodes::PassedPredicate(found_node.node))
	}
}


// https://www.w3.org/TR/1999/REC-xpath-19991116/#predicates
#[derive(Debug)]
struct Predicate(ExpressionArg);

impl Predicate {
	fn matches_eval(&mut self, context: &Evaluation) -> Result<Option<bool>> {
		let value = res_opt_catch!(self.0.next_eval(context));

		Ok(Some(match value {
			// Is Node in the correct position? ex: //node[3]
			Value::Number(v) => context.node_position == v as usize,
			// Otherwise ensure a value properly exists.
			_ => value.is_something()
		}))
	}
}


#[derive(Debug)]
pub struct Function(Box<dyn functions::Function>, Vec<ExpressionArg>);

impl Function {
	pub fn new(inner: Box<dyn functions::Function>, args: Vec<ExpressionArg>) -> Function {
		Self(inner, args)
	}
}

impl Expression for Function {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
		self.0.exec(eval, Args::new(self.1.as_mut())).map(Some)

		// TODO: Can't get type_name of dyn Functions' struct.
		// match self.0.exec(eval, Args::new(self.1.as_mut())) {
		// 	Ok(v) => Ok(Some(v)),
		// 	Err(v) => {
		// 		fn type_name_of_val<T: ?Sized>(_val: &T) -> String {
		// 			std::any::type_name::<T>().to_string()
		// 		}

		// 		Err(Error::FunctionError(type_name_of_val(&*self.0), Box::new(v)))
		// 	}
		// }
	}
}