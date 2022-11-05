use std::io::{Error as IoErrorBase, ErrorKind};

use thiserror::Error as ThisError;

use crate::ExprToken;


pub type Result<I> = std::result::Result<I, Error>;


#[derive(Debug, Clone, PartialEq, ThisError)]
pub enum Error {
	#[error("IO Error: {0:?}")]
	Io(ErrorKind),

	#[error("Token Error")]
	Token,
	#[error("Empty Input")]
	InputEmpty,
	#[error("Trailing Slash")]
	TrailingSlash,
	#[error("Expected Right Hand Expression for {0:?}")]
	ExpectedRightHandExpression(ExprToken),
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
	MissingFuncArgument,
	#[error("Unable to find Value")]
	UnableToFindValue
}

impl From<IoErrorBase> for Error {
	fn from(err: IoErrorBase) -> Error {
		Error::Io(err.kind())
	}
}


impl From<ValueError> for Error {
	fn from(err: ValueError) -> Error {
		Error::InvalidValue(err)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueError {
	Boolean,
	Number,
	String,
	Nodeset
}
