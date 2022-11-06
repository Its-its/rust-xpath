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
use std::sync::Mutex;

use tracing::{trace, Level};

use crate::functions::{self, Args};
use crate::{AxisName, Evaluation, Node, NodeTest, Nodeset, Result, Value};

pub type CallFunction = fn(ExpressionArg, ExpressionArg) -> ExpressionArg;
pub type ExpressionArg = Box<dyn Expression>;

macro_rules! res_opt_def_NAN {
    ($val:expr) => {
        match $val? {
            Some(v) => v,
            None => return Ok(Some(Value::Number(f64::NAN))),
        }
    };
}

macro_rules! res_opt_def_false {
    ($val:expr) => {
        match $val? {
            Some(v) => v,
            None => return Ok(Some(Value::Boolean(false))),
        }
    };
}

pub trait Expression: fmt::Debug {
    fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>>;

    // Helper Functions

    fn collect(&mut self, eval: &Evaluation) -> Result<Vec<Value>> {
        let mut nodes = Vec::new();

        while let Some(node) = self.next_eval(eval)? {
            nodes.push(node);
        }

        Ok(nodes)
    }
}

#[derive(Debug)]
pub struct Addition {
    left: ExpressionArg,
    right: ExpressionArg,
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

        Ok(Some(Value::Number(
            left_value.number()? + right_value.number()?,
        )))
    }
}

#[derive(Debug)]
pub struct Subtraction {
    left: ExpressionArg,
    right: ExpressionArg,
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

        Ok(Some(Value::Number(
            left_value.number()? - right_value.number()?,
        )))
    }
}

#[derive(Debug)]
pub struct LessThan {
    left: ExpressionArg,
    right: ExpressionArg,
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

        Ok(Some(Value::Boolean(
            left_value.number()? < right_value.number()?,
        )))
    }
}

#[derive(Debug)]
pub struct LessThanEqual {
    left: ExpressionArg,
    right: ExpressionArg,
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

        Ok(Some(Value::Boolean(
            left_value.number()? <= right_value.number()?,
        )))
    }
}

#[derive(Debug)]
pub struct GreaterThan {
    left: ExpressionArg,
    right: ExpressionArg,
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

        Ok(Some(Value::Boolean(
            left_value.number()? > right_value.number()?,
        )))
    }
}

#[derive(Debug)]
pub struct GreaterThanEqual {
    left: ExpressionArg,
    right: ExpressionArg,
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

        Ok(Some(Value::Boolean(
            left_value.number()? >= right_value.number()?,
        )))
    }
}

// Operations

#[derive(Debug)]
pub struct Equal {
    left: ExpressionArg,
    right: ExpressionArg,
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
    right: ExpressionArg,
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
    right: ExpressionArg,
}

impl And {
    pub fn new(left: ExpressionArg, right: ExpressionArg) -> Self {
        Self { left, right }
    }
}

impl Expression for And {
    fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
        let left_value = res_opt_def_false!(self.left.next_eval(eval));
        let right_value = res_opt_def_false!(self.right.next_eval(eval));

        Ok(Some(Value::Boolean(
            left_value.boolean()? && right_value.boolean()?,
        )))
    }
}

#[derive(Debug)]
pub struct Or {
    left: ExpressionArg,
    right: ExpressionArg,
}

impl Or {
    pub fn new(left: ExpressionArg, right: ExpressionArg) -> Self {
        Self { left, right }
    }
}

impl Expression for Or {
    fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
        let left_value = res_opt_def_false!(self.left.next_eval(eval));
        let right_value = res_opt_def_false!(self.right.next_eval(eval));

        Ok(Some(Value::Boolean(
            left_value.boolean()? || right_value.boolean()?,
        )))
    }
}

// Primary Expressions
#[derive(Debug)]
pub struct Union {
    left: ExpressionArg,
    right: ExpressionArg,
    skip_left: Mutex<bool>,
}

impl Union {
    pub fn new(left: ExpressionArg, right: ExpressionArg) -> Self {
        Self {
            left,
            right,
            skip_left: Mutex::new(false),
        }
    }
}

impl Expression for Union {
    fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
        if !*self.skip_left.lock().unwrap() {
            *self.skip_left.lock().unwrap() = true;

            let left_value = self.left.next_eval(eval)?;

            if left_value.is_some() {
                return Ok(left_value);
            }
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
        Ok(Some(Value::Node(eval.node.clone())))
    }
}

#[derive(Debug)]
pub struct Path {
    pub start_pos: ExpressionArg,
    pub steps: Vec<Step>,

    // TODO: We just cache everything it validated. Later we'll make it more ergonomic.
    found_cache: Option<Vec<Node>>,
    cached_from: Option<Node>,
}

impl Path {
    pub fn new(start_pos: ExpressionArg, steps: Vec<Step>) -> Self {
        Self {
            start_pos,
            steps,
            found_cache: None,
            cached_from: None,
        }
    }
}

impl Expression for Path {
    fn next_eval(&mut self, eval: &Evaluation) -> Result<Option<Value>> {
        // TODO: Better way to handle this.
        // Needed for Predicate Function Path. They're re-used for each node check.
        if self.cached_from.as_ref() != Some(eval.node) {
            self.found_cache = None;
        }

        if self.found_cache.is_none() {
            self.cached_from = Some(eval.node.clone());

            trace!("VVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVV");

            let Some(result) = self.start_pos.next_eval(eval)? else {
				return Ok(None);
			};

            let node = result.into_node()?;

            trace!("-> {}", crate::compile_lines(&node));

            let mut nodes = Nodeset { nodes: vec![node] };

            let mut prev_step_axis = None;
            for (i, step) in self.steps.iter_mut().enumerate() {
                nodes = step.evaluate(eval, nodes, prev_step_axis)?;
                prev_step_axis = Some(step.axis);

                if tracing::enabled!(Level::TRACE) {
                    trace!("Step [{i}]");
                    nodes
                        .nodes
                        .iter()
                        .for_each(|node| trace!("    {}", crate::compile_lines(node)));
                }
            }

            trace!("<- {nodes:?}");
            trace!("^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^");

            // Reverse it so we can use .pop later.
            nodes.nodes.reverse();

            self.found_cache = Some(nodes.nodes);
        }

        let nodes = self.found_cache.as_mut().unwrap();

        Ok(nodes.pop().map(Value::Node))
    }
}

#[derive(Debug)]
pub struct Step {
    axis: AxisName,
    node_test: Box<dyn NodeTest>, // A Step Test
    predicates: Vec<Predicate>,
}

impl Step {
    pub fn new(
        axis: AxisName,
        node_test: Box<dyn NodeTest>,
        predicates: Vec<ExpressionArg>,
    ) -> Step {
        let preds = predicates.into_iter().map(|p| Predicate(p)).collect();

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
        prev_step_axis: Option<AxisName>,
    ) -> Result<Nodeset> {
        let mut unique = Nodeset::new();

        for node in starting_nodes {
            let child_context = context.new_evaluation_from(&node);
            let mut nodes =
                child_context.find_nodes(&self.axis, self.node_test.as_ref(), prev_step_axis);

            for predicate in &mut self.predicates {
                nodes = predicate.select(context, nodes)?;
            }

            unique.extend(nodes);
        }

        if !self.predicates.is_empty() {
            trace!("Pre Predicate:");
            trace!("{:#?}", unique);
        }

        Ok(unique)
    }
}

// https://www.w3.org/TR/1999/REC-xpath-19991116/#predicates
#[derive(Debug)]
struct Predicate(ExpressionArg);

impl Predicate {
    fn select(&mut self, context: &Evaluation<'_>, nodes: Nodeset) -> Result<Nodeset> {
        let node_count = nodes.len();

        let mut found = Vec::new();

        for (index, node) in nodes.into_iter().enumerate() {
            let mut ctx = context.new_evaluation_from(&node);
            // TODO: Manage Better.
            ctx.position = index + 1;
            ctx.size = node_count;

            trace!("Pred [{index}] {}", crate::compile_lines(&node));

            if let Some(true) = self.matches_eval(&ctx)? {
                found.push(node)
            }
        }

        Ok(found.into())
    }

    fn matches_eval(&mut self, eval: &Evaluation<'_>) -> Result<Option<bool>> {
        let Some(value) = self.0.next_eval(eval)? else {
			return Ok(None);
		};

        Ok(Some(match value {
            // Is Node in the correct position? ex: //node[3]
            Value::Number(v) => eval.position == v as usize,
            // Otherwise ensure a value properly exists.
            _ => value.is_something(),
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
