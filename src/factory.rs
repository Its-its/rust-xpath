
use std::iter::Peekable;

use crate::{AxisName, DEBUG, Error, Evaluation, ExprToken, Node, NodeTest, NodeType, Nodeset, Operator, PrincipalNodeType, Result, Tokenizer, Value, value::PartialValue};
use crate::expressions::{ExpressionArg, ContextNode, RootNode, Path, Step, Literal, Equal, NotEqual, And, Or, Function};
use crate::nodetest;
use crate::functions;

type ExpressionResult = Result<Option<ExpressionArg>>;


// #[derive(Debug)]
pub struct ProduceIter<'a> {
	eval: Evaluation<'a>,
	expr: ExpressionArg
}

impl<'a> ProduceIter<'a> {
	pub fn collect_nodes(mut self) -> Result<Value> {
		Ok(Value::Nodeset(self.try_fold::<_, _, Result<Nodeset>>(
			Nodeset::new(),
			|mut set, v| {
				set.push(v.into_node()?);
				Ok(set)
			}
		)?))
	}
}

impl<'a> Iterator for ProduceIter<'a> {
	type Item = PartialValue;

	fn next(&mut self) -> Option<Self::Item> {
		self.expr.next_eval(&self.eval).ok().flatten()
	}
}


#[derive(Debug, Clone)]
pub struct Document {
	pub root: Node
}

impl Document {
	pub fn new(root: Node) -> Self {
		Self {
			root
		}
	}

	pub fn evaluate<S: Into<String>>(&self, search: S) -> Result<ProduceIter> {
		self.evaluate_from(search, self.root.clone())
	}

	pub fn evaluate_from<S: Into<String>>(&self, search: S, node: Node) -> Result<ProduceIter> {
		Factory::new(search, self, node).produce()
	}

	pub fn evaluate_steps(&self, steps: Vec<ExprToken>) -> Result<ProduceIter> {
		Factory::new_from_steps(steps, self, self.root.clone()).produce()
	}
}


macro_rules! return_value {
	($stepper:expr, ExprToken::$token:ident) => {{
		let step = $stepper.next().ok_or(Error::InputEmpty)?;

		match step {
			ExprToken::$token(v) => v,
			s => return Err(Error::UnexpectedToken(s))
		}
	}};
}

pub struct Factory<'eval> {
	eval: Evaluation<'eval>,
	tokenizer: Tokenizer,
	token_steps: Vec<ExprToken>,
	error: Option<Error>
}

impl<'eval, 'b: 'eval> Factory<'eval> {
	pub fn new<S: Into<String>>(query: S, document: &'eval Document, node: Node) -> Self {
		Factory {
			eval: Evaluation::new(node, document),
			tokenizer: Tokenizer::new(query),
			token_steps: Vec::new(),
			error: None
		}
	}

	pub fn new_from_steps(steps: Vec<ExprToken>, document: &'eval Document, node: Node) -> Self {
		Factory {
			eval: Evaluation::new(node, document),
			tokenizer: Tokenizer::new(""),
			token_steps: steps,
			error: None
		}
	}


	// Parse query, place tokens into token_steps.
	fn tokenize(&mut self) {
		while !self.tokenizer.is_finished() {
			match self.tokenizer.next().unwrap() {
				Ok(step) => self.expand_abbreviation(step),
				Err(e) => {
					eprintln!("{:?}", e);
					self.error = Some(e);
					return;
				}
			}
		}
	}

	// https://www.w3.org/TR/1999/REC-xpath-19991116/#path-abbrev
	fn expand_abbreviation(&mut self, token: ExprToken) {
		match token {
			//
			ExprToken::AtSign => {
				self.token_steps.push(AxisName::Attribute.into());
			}

			//
            ExprToken::Operator(Operator::DoubleForwardSlash) => {
				self.token_steps.extend([
					Operator::ForwardSlash.into(),
					AxisName::DescendantOrSelf.into(),
					NodeType::Node.into(),
					Operator::ForwardSlash.into()
				].iter().cloned());
			}

			//
            ExprToken::Period => {
				self.token_steps.extend([
					AxisName::SelfAxis.into(),
					NodeType::Node.into()
				].iter().cloned());
			}

			//
            ExprToken::ParentNode => {
				self.token_steps.extend([
					AxisName::Parent.into(),
					NodeType::Node.into()
				].iter().cloned());
			}

            _ => self.token_steps.push(token)
        }
	}

	pub fn produce(mut self) -> Result<ProduceIter<'eval>> {
		self.tokenize();

		if self.error.is_none() {
			if DEBUG {
				println!("Steps");
				self.token_steps
				.iter()
				.for_each(|t| println!(" - {:?}", t));
			}

			let mut stepper = Stepper::new(self.token_steps.clone().into_iter().peekable());

			if stepper.has_more_tokens() {
				let expr = self.parse_expression(&mut stepper)?;

				match expr {
					Some(expr) => {
						// if DEBUG { println!("Parsed: {:#?}", expr); }
						return Ok(ProduceIter::<'eval> { expr, eval: self.eval });
					}

					None => {
						// Couldn't find it. Invalid xpath.
						return Err(Error::InvalidXpath);
					}
				}
			}

			if !stepper.has_more_tokens() {
				println!("Finished.");
			}
		}

		Err(Error::UnableToEvaluate)
	}


	// Parse Types

	// Expr					::= OrExpr
	fn parse_expression<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> ExpressionResult {
		self.parse_or_expression(step)
	}

	// OrExpr				::= AndExpr | Self 'or' AndExpr
	fn parse_or_expression<S: Iterator<Item = ExprToken>>(&'b self, step: &mut Stepper<S>) -> ExpressionResult {
		let left_expr = self.parse_and_expression(step)?;

		// Self 'or' AndExpr
		if step.consume_if_next_token_is(Operator::Or)? {
			let right_expr = self.parse_relational_expression(step)?;

			return Ok(Some(Box::new(Or::new(left_expr.unwrap(), right_expr.ok_or(Error::MissingRightHandExpression)?))));
		}

		Ok(left_expr)
	}

	// AndExpr				::= EqualityExpr | Self 'and' EqualityExpr
	fn parse_and_expression<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> ExpressionResult {
		let left_expr = self.parse_equality_expression(step)?;

		// Self 'and' EqualityExpr
		if step.consume_if_next_token_is(Operator::And)? {
			let right_expr = self.parse_relational_expression(step)?;

			return Ok(Some(Box::new(And::new(left_expr.unwrap(), right_expr.ok_or(Error::MissingRightHandExpression)?))));
		}

		Ok(left_expr)
	}

	// EqualityExpr			::= RelationalExpr | Self '=' RelationalExpr | Self '!=' RelationalExpr
	fn parse_equality_expression<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> ExpressionResult {
		let left_expr = self.parse_relational_expression(step)?;

		// Self '=' RelationalExpr
		if step.consume_if_next_token_is(Operator::Equal)? {
			let right_expr = self.parse_relational_expression(step)?;

			return Ok(Some(Box::new(Equal::new(left_expr.unwrap(), right_expr.ok_or(Error::MissingRightHandExpression)?))));
		}

		// Self '!=' RelationalExpr
		if step.consume_if_next_token_is(Operator::DoesNotEqual)? {
			let right_expr = self.parse_relational_expression(step)?;

			return Ok(Some(Box::new(NotEqual::new(left_expr.unwrap(), right_expr.ok_or(Error::MissingRightHandExpression)?))));
		}

		Ok(left_expr)
	}

	// RelationalExpr		::= AdditiveExpr | Self '<' AdditiveExpr | Self '>' AdditiveExpr | Self '<=' AdditiveExpr | Self '>=' AdditiveExpr
	fn parse_relational_expression<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> ExpressionResult {
		let left_expr = self.parse_additive_expression(step)?;

		// Self '<' AdditiveExpr
		// Self '>' AdditiveExpr
		// Self '<=' AdditiveExpr
		// Self '>=' AdditiveExpr

		Ok(left_expr)
	}

	// AdditiveExpr			::= MultiplicativeExpr | Self '+' MultiplicativeExpr | Self '-' MultiplicativeExpr
	fn parse_additive_expression<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> ExpressionResult {
		let left_expr = self.parse_multiplicative_expression(step)?;

		// Self '+' MultiplicativeExpr
		// Self '-' MultiplicativeExpr

		Ok(left_expr)
	}

	// MultiplicativeExpr	::= UnaryExpr | Self MultiplyOperator UnaryExpr | Self 'div' UnaryExpr | Self 'mod' UnaryExpr
	fn parse_multiplicative_expression<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> ExpressionResult {
		let left_expr = self.parse_unary_expression(step)?;

		// Self MultiplyOperator UnaryExpr
		// Self 'div' UnaryExpr
		// Self 'mod' UnaryExpr

		Ok(left_expr)
	}

	// UnaryExpr			::= UnionExpr | '-' Self
	fn parse_unary_expression<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> ExpressionResult {
		if step.is_next_token(Operator::Minus) {
			let _ = step.consume(Operator::Minus)?;
		}
		// TODO: If missing union after consuming minus.

		self.parse_union_expression(step)
	}

	// UnionExpr			::= PathExpr | Self '|' PathExpr
	fn parse_union_expression<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> ExpressionResult {
		self.parse_path_expression(step)

		//  Self '|' PathExpr
	}


	// Path


	// PathExpr 			::= LocationPath
	// 							| FilterExpr
	// 							| FilterExpr '/' RelativeLocationPath
	// 							| FilterExpr '//' RelativeLocationPath
	fn parse_path_expression<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> ExpressionResult {
		let expr = self.parse_location_path_expression(step)?;

		if expr.is_some() {
            return Ok(expr);
        } // TODO: investigate if this is a pattern

        match self.parse_filter_expression(step)? {
            Some(expr) => {
                if step.is_next_token(Operator::ForwardSlash) {
                    step.consume(Operator::ForwardSlash)?;

					let expr = self.parse_location_path_raw(step, expr)?;

                    Ok(Some(expr.expect("parse_path_expression")))
                } else {
                    Ok(Some(expr))
                }
            }
            None => Ok(None),
        }
	}

	// LocationPath			::= RelativeLocationPath | AbsoluteLocationPath
	fn parse_location_path_expression<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> ExpressionResult {
		let path = self.parse_relative_location_path(step)?;

		if path.is_some() {
			Ok(path)
		} else {
			self.parse_absolute_location_path(step)
		}
	}

	// RelativeLocationPath	::= Step | RelativeLocationPath '/' Step | AbbreviatedRelativeLocationPath
	fn parse_relative_location_path<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> ExpressionResult {
		self.parse_location_path_raw(step, Box::new(ContextNode))
	}

	// AbsoluteLocationPath	::= '/' RelativeLocationPath? | AbbreviatedAbsoluteLocationPath
	fn parse_absolute_location_path<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> ExpressionResult {
		if step.is_next_token(Operator::ForwardSlash) {
            step.consume(Operator::ForwardSlash)?;

            match self.parse_location_path_raw(step, Box::new(RootNode))? {
                Some(expr) => Ok(Some(expr)),
                None => Ok(Some(Box::new(RootNode))),
            }
        } else {
			Ok(None)
        }
	}

	// AbbreviatedRelativeLocationPath ::= RelativeLocationPath '//' Step
	// fn parse_abbreviated_relative_location_path<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> ExpressionResult {
	// 	// self.parse_relative_location_path(step)

	// 	Ok(None)
	// }

	fn parse_location_path_raw<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>, start_point: ExpressionArg) -> ExpressionResult {
		match self.parse_step(step)? {
            Some(expr_step) => {
                let mut steps = vec![expr_step];

                while step.is_next_token(Operator::ForwardSlash) {
					step.consume(Operator::ForwardSlash)?;

					// TODO: Correctly fix Operator::Star in Tokenizer
					// if step.is_next_token(&Operator::Star.into()) {
					// 	step.consume(&Operator::Star.into())?;

					// 	steps.push( Step::new(
					// 		AxisName::Child,
					// 		Box::new(nodetest::Element::new(nodetest::NameTest { prefix: None, local_part: "*".into() })),
					// 		Vec::new()
					// 	));
					// } else {
					// }
					let next = self.parse_step(step)?;
					steps.push(next.ok_or(Error::TrailingSlash)?);

                }

                Ok(Some(Box::new(Path::new(start_point, steps))))
            }
            None => Ok(None),
        }
	}


	// A node test * is true for any node of the principal node type.
	// child::* will select all element children of the context node,
	// attribute::* will select all attributes of the context node.

	// Step					::= AxisSpecifier NodeTest Predicate* | AbbreviatedStep
	fn parse_step<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> Result<Option<Step>> {
		let axis = self.parse_axis_specifier(step)?;


        let node_test = match self.parse_node_test(step)? {
            Some(test) => Some(test),
            None => self.default_node_test(step, axis)?,
		};

        let node_test = match node_test {
            Some(test) => test,
            None => return Ok(None),
        };

        let predicates = self.parse_predicate_expressions(step)?;

		Ok(Some(Step::new(axis, node_test, predicates)))
	}

	// AxisSpecifier			::= AxisName '::' | AbbreviatedAxisSpecifier
	fn parse_axis_specifier<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> Result<AxisName> {
		if step.is_next_token_func(|t| t.is_axis()) {
            Ok(return_value!(step, ExprToken::Axis))
        } else {
            Ok(AxisName::Child)
        }
	}


	// Filter

	// FilterExpr			::= PrimaryExpr | Self Predicate
	fn parse_filter_expression<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> ExpressionResult {
		if let Some(expr) = self.parse_primary_expression(step)? {
	        // let predicates = self.parse_predicates(step)?;

	        // Ok(Some(predicates.into_iter().fold(expr, |expr, pred| {
	        //     Filter::new(expr, pred)
			// })))

			Ok(Some(expr))
		} else {
			Ok(None)
		}
	}

	// PrimaryExpr			::= VariableReference
	// 							| '(' Expr ')'
	// 							| Literal
	// 							| Number
	// 							| FunctionCall
	fn parse_primary_expression<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> ExpressionResult {
        // self.parse_variable_reference(step)
        // self.parse_nested_expression(step)
		// self.parse_string_literal(step)
		if step.is_next_token_func(|i| i.is_literal()) {
			let value = return_value!(step, ExprToken::Literal);
			return Ok(Some(Box::new(Literal::from(PartialValue::String(value)))));
		}

		// self.parse_numeric_literal(step)
		if step.is_next_token_func(|i| i.is_number()) {
			let value = return_value!(step, ExprToken::Number);
			return Ok(Some(Box::new(Literal::from(PartialValue::Number(value)))));
		}


		if let Some(func) = self.parse_function_call(step)? {
			return Ok(Some(Box::new(func)));
		}

		Ok(None)
	}

	// Function Calls
	fn parse_function_call<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> Result<Option<Function>> {
		if step.is_next_token_func(|i| i.is_function_name()) {
			let fn_name = return_value!(step, ExprToken::FunctionName);
			step.consume(ExprToken::LeftParen)?;

			// Function

			let function: Box<dyn functions::Function> = match fn_name.as_str() {
				"last" => Box::new(functions::Last),
				"position" => Box::new(functions::Position),
				"count" => Box::new(functions::Count),
				"local-name" => Box::new(functions::LocalName),
				"namespace-uri" => Box::new(functions::NamespaceUri),
				"name" => Box::new(functions::Name),
				"string" => Box::new(functions::ToString),
				"concat" => Box::new(functions::Concat),
				"starts-with" => Box::new(functions::StartsWith),
				"contains" => Box::new(functions::Contains),
				"substring-before" => Box::new(functions::SubstringBefore),
				"substring-after" => Box::new(functions::SubstringAfter),
				"substring" => Box::new(functions::Substring),
				"string-length" => Box::new(functions::StringLength),
				"normalize-space" => Box::new(functions::NormalizeSpace),
				"not" => Box::new(functions::Not),
				"true" => Box::new(functions::True),
				"false" => Box::new(functions::False),
				"sum" => Box::new(functions::Sum),
				"floor" => Box::new(functions::Floor),
				"ceiling" => Box::new(functions::Ceiling),
				"round" => Box::new(functions::Round),

				_ => return Ok(None)
			};

			let mut args = Vec::new();

			while !step.consume_if_next_token_is(ExprToken::RightParen)? {
				if let Some(expr) = self.parse_expression(step)? {
					args.push(expr);
				}

				step.consume_if_next_token_is(ExprToken::Comma)?;
			}

			Ok(Some(Function::new(function, args)))
		} else {
			Ok(None)
		}
	}

	// Node Test

	fn parse_node_test<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> Result<Option<Box<dyn NodeTest>>> {
		if step.is_next_token_func(|t| t.is_node_type()) {
			let name = return_value!(step, ExprToken::NodeType);

			match name {
				NodeType::Node => Ok(Some(Box::new(nodetest::Node))),
				NodeType::Text => Ok(Some(Box::new(nodetest::Text))),
				NodeType::Comment => Ok(Some(Box::new(nodetest::Comment))),
				NodeType::ProcessingInstruction(target) => Ok(Some(Box::new(
					nodetest::ProcessingInstruction::new(target),
				))),
			}
		} else {
			// if step.is_next_token(&Operator::Star.into()) {
			// 	step.consume(&Operator::Star.into())?;

			// 	Ok(Some(Box::new(nodetest::Element::new(nodetest::NameTest { prefix: None, local_part: "*".into() }))))
			// } else {
				Ok(None)
			// }
		}
	}

	fn default_node_test<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>, axis: AxisName) -> Result<Option<Box<dyn NodeTest>>> {
		if step.is_next_token_func(|t| t.is_name_test()) {
            let name = return_value!(step, ExprToken::NameTest);

            let test: Box<dyn NodeTest> = match axis.principal_node_type() {
                PrincipalNodeType::Attribute => Box::new(nodetest::Attribute::new(name)),
                PrincipalNodeType::Element => Box::new(nodetest::Element::new(name)),
                PrincipalNodeType::Namespace => Box::new(nodetest::Namespace::new(name)),
            };

            Ok(Some(test))
        } else {
            Ok(None)
        }
	}


	// Predicate

	// Predicate			::= '[' PredicateExpr ']'
	// PredicateExpr		::= Expr
	fn parse_predicate_expressions<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> Result<Vec<ExpressionArg>> {
		let mut expr = Vec::new();

		while let Some(pred) = self.parse_predicate_expression(step)? {
			expr.push(pred);
		}

		Ok(expr)
	}


	fn parse_predicate_expression<S: Iterator<Item = ExprToken>>(&self, step: &mut Stepper<S>) -> ExpressionResult {
		if step.is_next_token(ExprToken::LeftBracket) {
			step.consume(ExprToken::LeftBracket)?;

			let val = self.parse_expression(step)?;

			step.consume(ExprToken::RightBracket)?;

			Ok(val)
		} else {
			Ok(None)
		}
	}
}

// Expr							::= OrExpr


// let tokenizer = parser::Tokenizer::new(query);
// let found = tokenizer.collect::<Vec<parser::TokenResult>>();

// if found.iter().find(|i| i.is_err()).is_some() {
// 	println!("Err('{}'): {:?}", query, found);
// } else {
// 	println!("Ok('{}'): {:?}", query, found.into_iter().map(|i| i.unwrap()).collect::<Vec<_>>());
// }

//

pub struct Stepper<S: Iterator<Item = ExprToken>>(Peekable<S>);

impl<S: Iterator<Item = ExprToken>> Stepper<S> {
	pub fn new(steps: Peekable<S>) -> Self {
		Stepper(steps)
	}

	pub fn has_more_tokens(&mut self) -> bool {
		self.peek().is_some()
	}

	pub fn is_next_token<T: Into<ExprToken>>(&mut self, token: T) -> bool {
		match self.peek() {
			Some(t) => t == &token.into(),
			None => false
		}
	}

	pub fn is_next_token_func<F: FnOnce(&S::Item) -> bool>(&mut self, token: F) -> bool {
		match self.peek() {
			Some(t) => token(t),
			None => false
		}
	}

	pub fn consume_if_next_token_is<T: Into<ExprToken> + Clone>(&mut self, token: T) -> Result<bool> {
		if self.is_next_token(token.clone()) {
			self.consume(token)?;

			Ok(true)
		} else {
			Ok(false)
		}
	}

	pub fn consume<T: Into<ExprToken>>(&mut self, token: T) -> Result<()> {
		let step = self.next().ok_or(Error::InputEmpty)?;

		if step == token.into() {
			Ok(())
		} else {
			Err(Error::UnexpectedToken(step))
		}
	}

	pub fn peek(&mut self) -> Option<&S::Item> {
		self.0.peek()
	}
}

impl<S: Iterator<Item = ExprToken>> Iterator for Stepper<S> {
	type Item = S::Item;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}