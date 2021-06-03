use thiserror::Error as ThisError;

use std::io::{Error as IoErrorBase, ErrorKind};

use crate::ExprToken;


pub type Result<I> = std::result::Result<I, Error>;


#[derive(Debug, Clone, PartialEq, ThisError)]
pub enum Error {
	#[error("IO Error")]
	Io(ErrorKind),

	#[error("Token Error")]
	Token,
	#[error("Empty Input")]
	InputEmpty,
	#[error("Trailing Slash")]
	TrailingSlash,
	#[error("Missing Right Hand Expression")]
	MissingRightHandExpression,
	#[error("Unexpected Token {0:?}")]
	UnexpectedToken(ExprToken),
	#[error("Invalid Value {0:?}")]
	InvalidValue(ValueError),
	#[error("Cannot convert Node into Value")]
	CannotConvertNodeToValue,
	#[error("Node did not contain Text")]
	NodeDidNotContainText,
	#[error("Unable to Evaluate")]
	UnableToEvaluate,
	#[error("Invalid Xpath")]
	InvalidXpath,
	#[error("Missing Function Argument")]
	MissingFuncArgument
}


impl From<ValueError> for Error {
	fn from(err: ValueError) -> Error {
		Error::InvalidValue(err)
	}
}

impl From<IoErrorBase> for Error {
	fn from(err: IoErrorBase) -> Error {
		Error::Io(err.kind())
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueError {
	None,
	Boolean,
	Number,
	String,
	Nodeset
}