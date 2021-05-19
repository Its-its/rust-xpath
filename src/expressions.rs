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

use markup5ever_rcdom::NodeData;

use crate::functions::{self, Args};
use crate::{AxisName, DEBUG, Evaluation, Node, NodeTest, Nodeset, Result, Value, result::ValueError};

pub type CallFunction = fn(ExpressionArg, ExpressionArg) -> ExpressionArg;
pub type ExpressionArg = Box<dyn Expression>;


pub trait Expression: fmt::Debug {
	fn eval(&mut self, eval: &Evaluation) -> Result<Value>;

	fn count(&mut self) -> usize {
		0
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
	fn eval(&mut self, eval: &Evaluation) -> Result<Value> {
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
	fn eval(&mut self, eval: &Evaluation) -> Result<Value> {
		let left_value = self.left.eval(eval)?;
		let right_value = self.right.eval(eval)?;

		Ok(Value::Boolean(left_value != right_value))
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
	fn eval(&mut self, eval: &Evaluation) -> Result<Value> {
		let left_value = self.left.eval(eval)?;
		let right_value = self.right.eval(eval)?;

		Ok(Value::Boolean(left_value.boolean()? && right_value.boolean()?))
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
	fn eval(&mut self, eval: &Evaluation) -> Result<Value> {
		let left_value = self.left.eval(eval)?;
		let right_value = self.right.eval(eval)?;

		Ok(Value::Boolean(left_value.boolean()? || right_value.boolean()?))
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
	fn eval(&mut self, _: &Evaluation) -> Result<Value> {
		Ok(self.0.clone())
	}
}


// Nodeset

#[derive(Debug)]
pub struct RootNode;

impl Expression for RootNode {
	fn eval(&mut self, eval: &Evaluation) -> Result<Value> {
		Ok(Value::Nodeset(vec![eval.root().clone()].into()))
	}
}


#[derive(Debug)]
pub struct ContextNode;

impl Expression for ContextNode {
	fn eval(&mut self, eval: &Evaluation) -> Result<Value> {
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
	fn eval(&mut self, eval: &Evaluation) -> Result<Value> {
		let result = self.start_pos.eval(eval)?;
		let mut set = result.into_nodeset()?;

		// 1st. We evaluate a set of nodes and step for each each one.
		for step in &mut self.steps {
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
		&mut self,
		context: &Evaluation,
		starting_nodes: Nodeset,
	) -> Result<Nodeset> {
		let mut unique = Nodeset::new();

		// 2nd. The each Node has a Evaluation assigned to it and we check if it has the next step in it.
		for node in starting_nodes.nodes {
			let child_context = context.new_evaluation_from(node);

			let mut nodes = child_context.find_nodes(&self.axis, self.node_test.as_ref());

			// 3rd. Predicate check on the found Node(s)
			for predicate in &mut self.predicates {
				nodes = predicate.select(&context, nodes)?;
			}

			unique.extend(nodes);
		}

		if DEBUG && !self.predicates.is_empty() {
			println!("Pre Predicate:");
			println!("{:#?}", unique);
		}

		Ok(unique)
	}
}


// https://www.w3.org/TR/1999/REC-xpath-19991116/#predicates
#[derive(Debug)]
struct Predicate(ExpressionArg);

impl Predicate {
	fn select<'c>(
		&mut self,
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

	fn matches_eval(&mut self, context: &Evaluation<'_>) -> Result<bool> {
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
pub struct Function(Box<dyn functions::Function>, Vec<ExpressionArg>);

impl Function {
	pub fn new(inner: Box<dyn functions::Function>, args: Vec<ExpressionArg>) -> Function {
		Self(inner, args)
	}
}

impl Expression for Function {
	fn eval(&mut self, eval: &Evaluation) -> Result<Value> {
		self.0.exec(eval, Args::new(self.1.as_mut()))
	}
}


#[derive(Debug, Clone)]
pub enum PartialValue {
	Boolean(bool),
	Number(f64),
	String(String),
	Node(Node)
}

impl PartialValue {
	pub fn exists(&self) -> bool {
		match self {
			Self::Boolean(v) => *v,
			Self::Number(v) => !v.is_nan(),
			Self::String(v) => !v.is_empty(),
			Self::Node(_) => true
		}
	}

	pub fn as_node(&self) -> Result<&Node> {
		match self {
			Self::Node(s) =>  Ok(s),
			_ => Err(ValueError::Nodeset.into())
		}
	}

	pub fn is_node(&self) -> bool {
		matches!(self, Self::Node(_))
	}

	pub fn into_node(self) -> Result<Node> {
		match self {
			Self::Node(s) =>  Ok(s),
			_ => Err(ValueError::Nodeset.into())
		}
	}

	pub fn boolean(&self) -> Result<bool> {
		match self {
			Self::Boolean(v) =>  Ok(*v),
			_ => Err(ValueError::Boolean.into())
		}
	}

	pub fn number(&self) -> Result<f64> {
		match self {
			Self::Number(v) =>  Ok(*v),
			_ => Err(ValueError::Number.into())
		}
	}

	pub fn as_string(&self) -> Result<&String> {
		match self {
			Self::String(v) =>  Ok(v),
			_ => Err(ValueError::String.into())
		}
	}

	pub fn string(self) -> Result<String> {
		match self {
			Self::String(v) =>  Ok(v),
			_ => Err(ValueError::String.into())
		}
	}
}

impl Into<Value> for PartialValue {
	fn into(self) -> Value {
		match self {
			Self::Boolean(v) => Value::Boolean(v),
			Self::Number(v) => Value::Number(v),
			Self::String(v) => Value::String(v),
			Self::Node(v) => Value::Nodeset(Nodeset { nodes: vec![v] }),
		}
	}
}


impl PartialEq for PartialValue {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Number(v1), Self::Number(v2)) => v1 == v2,

			// Noteset == String
			(Self::Node(node), Self::String(value)) |
			(Self::String(value), Self::Node(node)) => {
				// TODO: No.
				if &format!("{:?}", node) == value {
					true
				} else {
					match node {
						Node::Attribute(attr) => {
							attr.value() == value
						}

						Node::Text(handle) => {
							let upgrade = handle.upgrade().unwrap();
							if let NodeData::Text { contents } = &upgrade.data {
								contents.try_borrow().map(|v| v.as_ref() == value).unwrap_or_default()
							} else {
								false
							}
						}

						_ => false
					}
				}
			}

			(Self::Node(set1), Self::Node(set2)) => {
				// TODO: No.
				format!("{:?}", set1) == format!("{:?}", set2)
			}

			_ => false
		}
	}
}