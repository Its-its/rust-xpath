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

use std::fmt;

use crate::{context::{NodeSearch, NodeSearchState}, functions::{self, Args}, value::PartialValue};
use crate::{AxisName, Evaluation, Node, NodeTest, Result};

pub type CallFunction = fn(ExpressionArg, ExpressionArg) -> ExpressionArg;
pub type ExpressionArg = Box<dyn Expression>;


pub trait Expression: fmt::Debug {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<PartialValue>>;

	fn count(&mut self, eval: &Evaluation) -> Result<usize> {
		let mut count = 0;

		while self.next_eval(eval)?.is_some() {
			count += 1;
		}

		Ok(count)
	}

	fn collect(&mut self, eval: &Evaluation) -> Result<Vec<PartialValue>> {
		let mut nodes = Vec::new();

		while let Some(node) = self.next_eval(eval)? {
			nodes.push(node);
		}

		Ok(nodes)
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
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<PartialValue>> {
		let left_value = res_opt_catch!(self.left.next_eval(eval));
		let right_value = res_opt_catch!(self.right.next_eval(eval));

		Ok(Some(PartialValue::Boolean(left_value == right_value)))
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
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<PartialValue>> {
		let left_value = res_opt_catch!(self.left.next_eval(eval));
		let right_value = res_opt_catch!(self.right.next_eval(eval));

		Ok(Some(PartialValue::Boolean(left_value != right_value)))
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
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<PartialValue>> {
		let left_value = res_opt_catch!(self.left.next_eval(eval));
		let right_value = res_opt_catch!(self.right.next_eval(eval));

		Ok(Some(PartialValue::Boolean(left_value.as_boolean()? && right_value.as_boolean()?)))
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
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<PartialValue>> {
		let left_value = res_opt_catch!(self.left.next_eval(eval));
		let right_value = res_opt_catch!(self.right.next_eval(eval));

		Ok(Some(PartialValue::Boolean(left_value.as_boolean()? || right_value.as_boolean()?)))
	}
}

// Primary Expressions

#[derive(Debug)]
pub struct Literal(PartialValue);

impl From<PartialValue> for Literal {
	fn from(value: PartialValue) -> Self {
		Literal(value)
	}
}

impl Expression for Literal {
	fn next_eval(&mut self, _: &Evaluation) -> Result<Option<PartialValue>> {
		Ok(Some(self.0.clone()))
	}
}


// Nodeset

#[derive(Debug)]
pub struct RootNode;

impl Expression for RootNode {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<PartialValue>> {
		Ok(Some(PartialValue::Node(eval.root().clone())))
	}
}


#[derive(Debug)]
pub struct ContextNode;

impl Expression for ContextNode {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<PartialValue>> {
		// TODO: Figure out. Cannot clone an Rc
		Ok(Some(PartialValue::Node(eval.node.clone())))
	}
}


#[derive(Debug)]
pub struct Path {
	pub start_pos: ExpressionArg,
	pub steps: Vec<Step>,
	pub search_steps: Vec<NodeSearch>,
	pub steps_initiated: bool
}

impl Path {
	pub fn new(start_pos: ExpressionArg, steps: Vec<Step>) -> Self {
		Self {
			start_pos,
			steps,
			search_steps: Vec::new(),
			steps_initiated: false
		}
	}

	pub fn find_next_node_with_steps(&mut self, eval: &Evaluation) -> Result<Option<Node>> {
		if self.search_steps.is_empty() {
			Ok(None)
		} else {
			while let Some(mut search_state) = self.search_steps.pop() {
				let step = &mut self.steps[self.search_steps.len()];

				let found_node_eval = step.evaluate(eval, &mut search_state)?;

				if let Some(passed_pred_eval) = found_node_eval {
					self.search_steps.push(search_state);

					if let Some(node) = passed_pred_eval {
						if self.steps.len() == self.search_steps.len() {
							return Ok(Some(node));
						} else {
							// Add to step state
							let step = &self.steps[self.search_steps.len()];

							self.search_steps.push(NodeSearch::new_with_state(step.axis, node, eval, &*step.node_test));
						}
					}
				}
			}

			Ok(None)
		}
	}

	pub fn find_next_node(&mut self, eval: &Evaluation) -> Result<Option<Node>> {
		if self.steps_initiated && self.search_steps.is_empty() {
			return Ok(None);
		}

		let result = res_opt_catch!(self.start_pos.next_eval(eval));

		let node = if self.search_steps.is_empty() {
			self.steps_initiated = true;

			let mut found = result.into_node()?;

			for step in &mut self.steps {
				let mut state = NodeSearch::new_with_state(step.axis, found, eval, &*step.node_test);

				found = match step.evaluate(eval, &mut state)?.flatten() {
					Some(v) => v,
					None => {
						return Ok(Some(res_opt_catch!(self.find_next_node_with_steps(eval))));
					}
				};

				self.search_steps.push(state);
			}

			found
		} else {
			res_opt_catch!(self.find_next_node_with_steps(eval))
		};

		Ok(Some(node))
	}
}

impl Expression for Path {
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<PartialValue>> {
		let found_node = res_opt_catch!(self.find_next_node(eval));

		//

		Ok(Some(PartialValue::Node(found_node)))
	}
}



#[derive(Debug)]
pub struct Step {
	axis: AxisName,
	node_test: Box<dyn NodeTest>, // A Step Test
	predicates: Vec<Predicate>,
	search_cache: Option<NodeSearchState>
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
			predicates: preds,
			search_cache: None
		}
	}

	fn evaluate(
		&mut self,
		context: &Evaluation,
		state: &mut NodeSearch
	) -> Result<Option<Option<Node>>> {
		// Option<Option<Node>> - 1st Option is used to check if we found a node. 2nd is returning Node if predicates succeed.

		let found_node = match state.find_and_cache_next_node(context, self.node_test.as_ref()) {
			Some(v) => v,
			None => return Ok(None)
		};

		let mut node = found_node.node;

		// Check specifiers.
		for predicate in &mut self.predicates {
			let mut eval = context.new_evaluation_from_with_pos(node, found_node.position);
			eval.is_last_node = state.is_finished();

			node = match predicate.select(eval)? {
				Some(v) => v,
				None => return Ok(Some(None))
			}
		}

		Ok(Some(Some(node)))
	}
}


// https://www.w3.org/TR/1999/REC-xpath-19991116/#predicates
#[derive(Debug)]
struct Predicate(ExpressionArg);

impl Predicate {
	fn select(
		&mut self,
		ctx: Evaluation
	) -> Result<Option<Node>> {
		if res_opt_catch!(self.matches_eval(&ctx)) {
			Ok(Some(ctx.node))
		} else {
			Ok(None)
		}
	}

	fn matches_eval(&mut self, context: &Evaluation<'_>) -> Result<Option<bool>> {
		let value = res_opt_catch!(self.0.next_eval(context));

		println!("{:?} == {:?}", value, context.node_position);

		Ok(Some(match value {
			// Is Node in the correct position? ex: //node[3]
			PartialValue::Number(v) => context.node_position == v as usize,
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
	fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<PartialValue>> {
		self.0.exec(eval, Args::new(self.1.as_mut())).map(Some)
	}
}