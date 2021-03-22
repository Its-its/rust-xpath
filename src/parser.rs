use regex::Regex;

use crate::{Result, Error, NameTest, DEBUG};
use crate::tokens::{AxisName, ExprToken, Operator, NodeType};

pub type Id<T> = (&'static str, T);

pub static SINGLE_CHAR_TOKENS: [Id<ExprToken>; 13] = [
    ("/", ExprToken::Operator(Operator::ForwardSlash)),
    ("(", ExprToken::LeftParen),
    (")", ExprToken::RightParen),
    ("[", ExprToken::LeftBracket),
    ("]", ExprToken::RightBracket),
    ("@", ExprToken::AtSign),
    ("+", ExprToken::Operator(Operator::Plus)),
    ("-", ExprToken::Operator(Operator::Minus)),
    ("|", ExprToken::Operator(Operator::Pipe)),
    ("=", ExprToken::Operator(Operator::Equal)),
    ("<", ExprToken::Operator(Operator::LessThan)),
    (">", ExprToken::Operator(Operator::GreaterThan)),
    (",", ExprToken::Comma)
];

pub static DOUBLE_CHAR_TOKENS: [Id<ExprToken>; 5] = [
    ("<=", ExprToken::Operator(Operator::LessThanOrEqual)),
    (">=", ExprToken::Operator(Operator::GreaterThanOrEqual)),
    ("!=", ExprToken::Operator(Operator::DoesNotEqual)),
    ("//", ExprToken::Operator(Operator::DoubleForwardSlash)),
    ("..", ExprToken::ParentNode)
];

// TODO: Disabled for now.
// Reasons:
//     '/html/body/div[1]/following::*'    'div' being converted
//     '/html/body/*'                      '*' being converted
pub static NAMED_OPERATORS: [Id<ExprToken>; 3] = [
    ("and", ExprToken::Operator(Operator::And)),
    ("or" , ExprToken::Operator(Operator::Or)),
    // ("mod", ExprToken::Operator(Operator::Mod)),
    // ("div", ExprToken::Operator(Operator::Div)),
    ("*"  , ExprToken::Operator(Operator::Star))
];

pub static AXES: [Id<AxisName>; 13] = [
    ("ancestor-or-self", AxisName::AncestorOrSelf),
    ("ancestor", AxisName::Ancestor),
    ("attribute", AxisName::Attribute),
    ("child", AxisName::Child),
    ("descendant-or-self", AxisName::DescendantOrSelf),
    ("descendant", AxisName::Descendant),
    ("following-sibling", AxisName::FollowingSibling),
    ("following", AxisName::Following),
    ("namespace", AxisName::Namespace),
    ("parent", AxisName::Parent),
    ("preceding-sibling", AxisName::PrecedingSibling),
    ("preceding", AxisName::Preceding),
    ("self", AxisName::SelfAxis),
];

pub static NODE_TYPES: [Id<NodeType>; 4] = [
    ("comment", NodeType::Comment),
    ("text", NodeType::Text),
    (
        "processing-instruction",
        NodeType::ProcessingInstruction(None),
    ),
    ("node", NodeType::Node),
];




pub type TokenResult = Result<ExprToken>;

pub type ParseResult = Option<(usize, ExprToken)>;

pub struct Tokenizer {
	xpath: String,
	pos: usize,

}

impl Tokenizer {
	pub fn new<S: Into<String>>(xpath: S) -> Tokenizer {
		Tokenizer {
			xpath: xpath.into(),
			pos: 0
		}
	}

	pub fn is_finished(&self) -> bool {
		self.xpath.len() <= self.pos
	}

	pub fn next_token(&mut self) -> TokenResult {
		let remaining_xpath = {
			// Remove Whitespace from start
			let bytes = self.xpath.as_bytes();
			while bytes[self.pos] == b' ' {
				self.pos += 1;
			}

			&self.xpath[self.pos..]
		};

		let found = None
			// Double Characters
			.or_else(|| Tokenizer::parse_token_array(remaining_xpath, &DOUBLE_CHAR_TOKENS))
			// Single Characters
			.or_else(|| Tokenizer::parse_token_array(remaining_xpath, &SINGLE_CHAR_TOKENS))
			// Literals
			.or_else(|| Tokenizer::parse_literal(remaining_xpath))
			// Number
			.or_else(|| Tokenizer::parse_numbers(remaining_xpath))
			// Current Node
			.or_else(|| Tokenizer::parse_current_node(remaining_xpath))
			// Named Operators
			.or_else(|| Tokenizer::parse_token_array(remaining_xpath, &NAMED_OPERATORS))
			// Axis Specifier
			.or_else(|| Tokenizer::parse_axes(remaining_xpath))
			// Node Type
			.or_else(|| Tokenizer::parse_node_types(remaining_xpath))
			// Function Call
			.or_else(|| Tokenizer::parse_function_call(remaining_xpath))
			// Variable Reference
			.or_else(|| Tokenizer::parse_variable_ref(remaining_xpath))
			// Name Test
			.or_else(|| Tokenizer::parse_name_test(remaining_xpath));

		if DEBUG { println!("--- {:?}", remaining_xpath); }

		if let Some((inc, token)) = found {
			self.pos += inc;
			Ok(token)
		} else {
			self.pos = self.xpath.len();
			Err(Error::Token)
		}
	}


	fn parse_token_array<T: Clone + Into<ExprToken>>(rem_path: &str, identities: &[Id<T>]) -> ParseResult {
		if DEBUG { println!("attempt_parse: {}", identities.len()); }

		for (name, id) in identities {
			if rem_path.len() < name.len() {
				continue;
			}

			if &rem_path[0..name.len()] == *name {
				return Some((name.len(), id.clone().into()));
			}
		}

		None
	}

	fn parse_literal(rem_path: &str) -> ParseResult {
		if DEBUG { println!("parse_literal"); }

		// "[^"]*" | '[^']*'
		let as_bytes = rem_path.as_bytes();

		if as_bytes[0] == b'"' || as_bytes[0] == b'\'' {
			let quote_type = if as_bytes[0] == b'"' {
				b'"'
			} else {
				b'\''
			};

			let mut end_pos = 1;

			while as_bytes.len() > end_pos && as_bytes[end_pos] != quote_type {
				end_pos += 1;
			}

			// Add 1 to include last quote
			end_pos += 1;

			if as_bytes.len() >= end_pos && end_pos - 1 != 1 && as_bytes[end_pos - 1] == quote_type {
				// Add 1 to start, remove 1 from end to remove both quotes.
				Some((end_pos, ExprToken::Literal(rem_path[1..end_pos - 1].to_string())))
			} else {
				eprintln!("Invalid Literal Found");
				// TODO: Error instead since it's not a valid literal.
				None
			}
		} else {
			None
		}
	}

	fn parse_numbers(rem_path: &str) -> ParseResult {
		if DEBUG { println!("parse_numbers"); }
		// Digits = [0-9]+
		// Digits ('.' Digits?)? | '.' Digits

		let numbers = &[b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'.'];

		let as_bytes = rem_path.as_bytes();

		if numbers.contains(&as_bytes[0]) {
			let mut used_decimal = false;
			let mut end_pos = 0;

			while as_bytes.len() > end_pos && numbers.contains(&as_bytes[end_pos]) {
				if as_bytes[end_pos] == b'.' {
					if used_decimal {
						// TODO: Return Error
						eprintln!("Multiple Decimals detected.");
						return None;
					}
					used_decimal = true;
				}

				end_pos += 1;
			}

			// Not a number. Probably referencing current node.
			if used_decimal && end_pos == 1 {
				None
			} else {
				Some((end_pos, ExprToken::Number(rem_path[0..end_pos].parse().expect("ExprToken::Number"))))
			}
		} else {
			None
		}
	}

	fn parse_current_node(rem_path: &str) -> ParseResult {
		if DEBUG { println!("parse_current_node"); }

		if rem_path.get(0..1).expect("parse_current_node") == "." {
			Some((1, ExprToken::Period))
		} else {
			None
		}
	}

	fn parse_axes(rem_path: &str) -> ParseResult {
		if DEBUG { println!("parse_axes"); }

		if let Some(mut parsed) = Tokenizer::parse_token_array(rem_path, &AXES) {
			if rem_path.len() >= parsed.0 + 2 && &rem_path[parsed.0..parsed.0 + 2] == "::" {
				parsed.0 += 2;
				return Some(parsed);
			}
		}

		None
	}

	fn parse_node_types(rem_path: &str) -> ParseResult {
		if DEBUG { println!("parse_node_types"); }

		if let Some((mut last_pos, results)) = Tokenizer::parse_token_array(rem_path, &NODE_TYPES) {
			if let Some((size, inner_str)) = Tokenizer::find_function_parenth(&rem_path[last_pos..]) {
				last_pos += size;

				let mut node_type: NodeType = results.into();

				// Check to see if it's a Processing Instruction. If so, check the parentheses
				if let NodeType::ProcessingInstruction(inner) = &mut node_type {
					*inner = inner_str.map(|i| i.to_string());
				}

				return Some((last_pos, ExprToken::NodeType(node_type)));
			} else {
				return Some((last_pos, results));
			}
		}

		None
	}

	// QName	   	::= Prefix ':' LocalPart | LocalPart
	// Prefix	   	::= NCName
	// LocalPart	::= NCName
	// NCName		::= Name - (Char* ':' Char*) /* An XML Name, minus the ":" */

	fn parse_function_call(rem_path: &str) -> ParseResult {
		if DEBUG { println!("parse_function_call"); }

		// FunctionName ::= QName - NodeType (QName excluding NodeTypes)
		// FunctionCall	::= FunctionName '(' ( Argument ( ',' Argument )* )? ')'
		// Argument		::= Expr

		// xml.txt: NameStartChar
		let reg = Regex::new(r#"^[a-zA-Z0-9:_]+"#).unwrap();

		if let Some(found) = reg.find(rem_path) {
			if Tokenizer::find_function_parenth(&rem_path[found.end()..]).is_some() {
				return Some((found.end(), ExprToken::FunctionName(rem_path[0..found.end()].to_string())));
			}
		}

		None
	}

	fn parse_variable_ref(rem_path: &str) -> ParseResult {
		if DEBUG { println!("parse_variable_ref"); }

		// '$' QName
		let reg = Regex::new(r#"^\$[a-zA-Z0-9:_]+"#).unwrap();

		if let Some(found) = reg.find(rem_path) {
			// Capture QName
			return Some((found.end(), ExprToken::VariableReference(rem_path[1..found.end()].to_string())));
		}

		None
	}

	fn parse_name_test(rem_path: &str) -> ParseResult {
		if DEBUG { println!("parse_name_test"); }

		// '*' | NCName ':' '*' | QName
		let bytes = rem_path.as_bytes();

		// TODO: This is never going to be called since
		// TODO: parse_token_array defines it as an Operator.
		// *
		if bytes[0] == b'*' {
			Some((1, ExprToken::NameTest(NameTest { prefix: None, local_part: "*".into() })))
		} else {
			let reg = Regex::new(r#"^[a-zA-Z0-9_]+:\*"#).unwrap();

			// NCName:*
			if let Some(found) = reg.find(rem_path) {
				let opts = rem_path[0..found.end()].split(':').collect::<Vec<&str>>();

				Some((found.end(), ExprToken::NameTest(NameTest { prefix: Some(opts[0].into()), local_part: opts[1].into() })))
			} else {
				// Prefix ':' LocalPart | LocalPart
				let reg = Regex::new(r#"(^[a-zA-Z0-9_]+:?(?:[a-zA-Z0-9_]+)?)"#).unwrap();

				if let Some(found) = reg.find(rem_path) {
					let opts = rem_path[0..found.end()].split(':').collect::<Vec<&str>>();

					if opts.len() == 1 {
						Some((found.end(), ExprToken::NameTest(NameTest { prefix: None, local_part: opts[0].into() })))
					} else {
						Some((found.end(), ExprToken::NameTest(NameTest { prefix: Some(opts[0].into()), local_part: opts[1].into() })))
					}
				} else {
					None
				}
			}
		}
	}


	fn find_function_parenth(rem_path: &str) -> Option<(usize, Option<&str>)> {
		if DEBUG { println!("parse_function_parenth"); }

		let bytes = rem_path.as_bytes();

		if bytes.len() >= 2 && bytes[0] == b'(' {
			let mut inner_size = 0;

			while inner_size < bytes.len() && bytes[inner_size] != b')' {
				inner_size += 1;
			}

			// Add 1 to capture ")"
			inner_size += 1;

			if rem_path.len() >= inner_size {
				let inner_str = if inner_size == 2 {
					None
				} else {
					Some(&rem_path[1..inner_size - 1])
				};

				return Some((inner_size, inner_str));
			}
		}

		None
	}

	// fn find_qname(_path: &str) -> Option<&str> {
	// 	None
	// }

	// // 'Name' with ':' removed
	// fn find_ncname(_path: &str) -> Option<&str> {
	// 	None
	// }
}


impl Iterator for Tokenizer {
	type Item = TokenResult;

	fn next(&mut self) -> Option<TokenResult> {
		if self.is_finished() {
			None
		} else {
			Some(self.next_token())
		}
	}
}