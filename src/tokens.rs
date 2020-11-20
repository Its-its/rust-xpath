
use crate::{NameTest};

// https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-AxisName
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AxisName {
	/// 'ancestor'
	/// Contains the ancestors of the context node;
	/// the ancestors of the context node consist of the parent of context node and the parent's parent and so on;
	/// thus, the ancestor axis will always include the root node, unless the context node is the root node
	Ancestor,
	/// 'ancestor-or-self'
	/// Contains the context node and the ancestors of the context node;
	/// thus, the ancestor axis will always include the root node
	AncestorOrSelf,
	/// 'attribute'
	/// Contains the attributes of the context node; the axis will be empty unless the context node is an element
	Attribute,
	/// 'child'
	/// Contains the children of the context node
	Child,
	/// 'descendant'
	/// Contains the descendants of the context node;
	/// a descendant is a child or a child of a child and so on;
	/// thus the descendant axis never contains attribute or namespace nodes
	Descendant,
	/// 'descendant-or-self'
	/// Contains the context node and the descendants of the context node
	DescendantOrSelf,
	/// 'following'
	/// Contains all nodes in the same document as the context node that are after the context node in document order, excluding any descendants and excluding attribute nodes and namespace nodes
	Following,
	/// 'following-or-self'
	/// Contains all the following siblings of the context node;
	/// if the context node is an attribute node or namespace node, the following-sibling axis is empty
	FollowingSibling,
	/// 'namespace'
	/// Contains the namespace nodes of the context node;
	/// the axis will be empty unless the context node is an element
	Namespace,
	/// 'parent'
	/// Contains the parent of the context node, if there is one
	Parent,
	/// 'preceding'
	/// Contains all nodes in the same document as the context node that are before the context node in document order, excluding any ancestors and excluding attribute nodes and namespace nodes
	Preceding,
	/// 'preceding-sibling'
	/// Contains all the preceding siblings of the context node;
	/// if the context node is an attribute node or namespace node, the preceding-sibling axis is empty
	PrecedingSibling,
	/// 'self'
	/// Contains just the context node itself
	SelfAxis
}

impl AxisName {
	pub fn principal_node_type(&self) -> PrincipalNodeType {
        match *self {
            AxisName::Attribute => PrincipalNodeType::Attribute,
            AxisName::Namespace => PrincipalNodeType::Namespace,
            _ => PrincipalNodeType::Element,
        }
    }
}

// PartialEq<markup5ever::Attribute> for NameTest


#[derive(Debug, Clone, PartialEq)]
pub enum PrincipalNodeType {
	Attribute,
	Namespace,
	Element
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
	Comment,
	Text,
	ProcessingInstruction(Option<String>),
	Node
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
	// OperatorName
	/// 'and'
	And,
	/// 'or'
	Or,
	/// 'mod'
	Mod,
	/// 'div'
	Div,

	// MultiplyOperator
	/// '*'
	Star,

	// Other
	/// '/'
	ForwardSlash,
	/// '//'
	DoubleForwardSlash,
	/// '|'
	Pipe,
	/// '+'
	Plus,
	/// '-'
	Minus,
	/// '='
	Equal,
	/// '!='
	DoesNotEqual,
	/// '<'
	LessThan,
	/// '<='
	LessThanOrEqual,
	/// '>'
	GreaterThan,
	/// '>='
	GreaterThanOrEqual
}


// https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-ExprToken
#[derive(Debug, Clone, PartialEq)]
pub enum ExprToken {
	/// '('
	LeftParen,
	/// ')'
	RightParen,
	/// '['
	LeftBracket,
	/// ']'
	RightBracket,
	/// '.'
	Period,
	/// '..'
	ParentNode,
	/// '@'
	AtSign,
	/// ','
	Comma,
	/// '::'
	LocationStep,


	// Specializations
    Axis(AxisName),
	Number(f64),
	Literal(String),
	NameTest(NameTest),
	NodeType(NodeType),
	Operator(Operator),
	FunctionName(String),
	VariableReference(String)
}

impl ExprToken {
	pub fn is_node_type(&self) -> bool {
		match self {
			ExprToken::NodeType(_) => true,
			_ => false
		}
	}

	pub fn is_name_test(&self) -> bool {
		match self {
			ExprToken::NameTest(_) => true,
			_ => false
		}
	}

	pub fn is_operator(&self) -> bool {
		match self {
			ExprToken::Operator(_) => true,
			_ => false
		}
	}

	pub fn is_axis(&self) -> bool {
		match self {
			ExprToken::Axis(_) => true,
			_ => false
		}
	}

	pub fn is_literal(&self) -> bool {
		match self {
			ExprToken::Literal(_) => true,
			_ => false
		}
	}

	pub fn is_number(&self) -> bool {
		match self {
			ExprToken::Number(_) => true,
			_ => false
		}
	}

	pub fn is_function_name(&self) -> bool {
		match self {
			ExprToken::FunctionName(_) => true,
			_ => false
		}
	}
}

macro_rules! from_impl {
	($struct:ident, $enum:ident) => {
		impl Into<ExprToken> for $struct {
			fn into(self) -> ExprToken {
				ExprToken::$enum(self)
			}
		}

		impl Into<ExprToken> for &$struct {
			fn into(self) -> ExprToken {
				ExprToken::$enum(self.clone())
			}
		}

		impl Into<$struct> for ExprToken {
			fn into(self) -> $struct {
				match self {
					ExprToken::$enum(op) => op,
					_ => panic!("ExprToken is not an $enum")
				}
			}
		}

		impl Into<$struct> for &ExprToken {
			fn into(self) -> $struct {
				match self {
					ExprToken::$enum(op) => op.clone(),
					_ => panic!("ExprToken is not an $enum")
				}
			}
		}

		// impl From<$struct> for ExprToken {
		// 	fn from(op: $struct) -> ExprToken {
		// 		ExprToken::$enum(op)
		// 	}
		// }

		// impl From<&$struct> for ExprToken {
		// 	fn from(op: &$struct) -> ExprToken {
		// 		ExprToken::$enum(op.clone())
		// 	}
		// }
	};
}

from_impl!(AxisName, Axis);
from_impl!(f64, Number);
from_impl!(String, Literal);
from_impl!(NameTest, NameTest);
from_impl!(NodeType, NodeType);
from_impl!(Operator, Operator);

// impl Into<Operator> for ExprToken {
// 	fn into(self) -> Operator {
// 		match self {
// 			ExprToken::Operator(op) => op,
// 			_ => panic!("ExprToken is not an Operator")
// 		}
// 	}
// }