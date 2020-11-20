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

use crate::functions;
use crate::{DEBUG, Value, Evaluation, Result, AxisName, Nodeset, NodeTest, Node};

pub type CallFunction = fn(ExpressionArg, ExpressionArg) -> ExpressionArg;
pub type ExpressionArg = Box<dyn Expression>;


pub trait Expression: fmt::Debug {
	fn eval(&self, eval: &Evaluation) -> Result<Value>;
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
	fn eval(&self, eval: &Evaluation) -> Result<Value> {
		let left_value = self.left.eval(eval)?;
		let right_value = self.right.eval(eval)?;

		Ok(Value::Boolean(left_value == right_value))
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
	fn eval(&self, eval: &Evaluation) -> Result<Value> {
		let left_value = self.left.eval(eval)?;
		let right_value = self.right.eval(eval)?;

		Ok(Value::Boolean(left_value != right_value))
	}
}



// Primary Expressions

#[derive(Debug)]
pub struct Literal(Value);

impl From<Value> for Literal {
	fn from(value: Value) -> Self {
		Literal(value)
	}
}

impl Expression for Literal {
	fn eval(&self, _: &Evaluation) -> Result<Value> {
		Ok(self.0.clone())
	}
}


// Nodeset

#[derive(Debug)]
pub struct RootNode;

impl Expression for RootNode {
	fn eval(&self, eval: &Evaluation) -> Result<Value> {
		Ok(Value::Nodeset(vec![eval.root().clone()].into()))
	}
}


#[derive(Debug)]
pub struct ContextNode;

impl Expression for ContextNode {
	fn eval(&self, eval: &Evaluation) -> Result<Value> {
		// TODO: Figure out. Cannot clone an Rc
		Ok(Value::Nodeset(vec![eval.node.clone()].into()))
	}
}


#[derive(Debug)]
pub struct Path {
	pub start_pos: ExpressionArg,
	pub steps: Vec<Step>
}

impl Path {
	pub fn new(start_pos: ExpressionArg, steps: Vec<Step>) -> Self {
		Self {
			start_pos,
			steps
		}
	}
}

impl Expression for Path {
	fn eval(&self, eval: &Evaluation) -> Result<Value> {
		let result = self.start_pos.eval(eval)?;
		let mut set = result.into_nodeset()?;

		for step in &self.steps {
			set = step.evaluate(eval, set)?;
		}

		Ok(Value::Nodeset(set))
	}
}



#[derive(Debug)]
pub struct Step {
	axis: AxisName,
	node_test: Box<dyn NodeTest>, // A Step Test
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
			.map(|p| Predicate(p))
			.collect();

		Step {
			axis,
			node_test,
			predicates: preds,
		}
	}

	fn evaluate(
		&self,
		context: &Evaluation,
		starting_nodes: Nodeset,
	) -> Result<Nodeset> {
		// For every starting node, we collect new nodes based on the
		// axis and node-test. We evaluate the predicates on each node.

		// This seems like a likely place where we could differ from
		// the spec, so thorough testing is key.

		let mut unique = Nodeset::new();

		for node in starting_nodes.nodes {
			let child_context = context.new_evaluation_from(node);
			let nodes = child_context.find_nodes(&self.axis, self.node_test.as_ref());

			unique.extend(nodes);
		}

		if DEBUG && !self.predicates.is_empty() {
			println!("Pre Predicate:");
			println!("{:#?}", unique);
		}

		for predicate in &self.predicates {
			unique = predicate.select(&context, unique)?;
		}

		Ok(unique)
	}
}


// https://www.w3.org/TR/1999/REC-xpath-19991116/#predicates
#[derive(Debug)]
struct Predicate(ExpressionArg);

impl Predicate {
	fn select<'c>(
		&self,
		context: &Evaluation<'c>,
		nodes: Nodeset,
	) -> Result<Nodeset> {
		let found: Vec<Node> = context.new_evaluation_set_from(nodes)
			.filter_map(|ctx| {
				match self.matches_eval(&ctx) {
					Ok(true) => Some(Ok(ctx.node)),
					Ok(false) => None,
					Err(e) => Some(Err(e)),
				}
			})
			.collect::<Result<Vec<Node>>>()?;

		Ok(found.into())
	}

	fn matches_eval(&self, context: &Evaluation<'_>) -> Result<bool> {
		let value = self.0.eval(context)?;

		Ok(match value {
			// Is Node in the correct position? ex: //node[3]
			Value::Number(v) => context.position == v as usize,
			// Otherwise ensure a value properly exists.
			_ => value.exists()
		})
	}
}


#[derive(Debug)]
pub struct Function(Box<dyn functions::Function>);

impl Function {
	pub fn new(inner: Box<dyn functions::Function>) -> Function {
		Function(inner)
	}
}

impl Expression for Function {
	fn eval(&self, eval: &Evaluation) -> Result<Value> {
		self.0.exec(eval)
	}
}