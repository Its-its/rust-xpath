use std::fmt;

use crate::ExprToken;


pub type Result<I> = std::result::Result<I, Error>;


#[derive(Debug, Clone)]
pub enum Error {
	Token,
	InputEmpty,
	TrailingSlash,
	MissingRightHandExpression,
	UnexpectedToken(ExprToken),
	InvalidValue(ValueError)
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use Error::*;

		match self {
			Token => write!(f, "Token Error"),
			InputEmpty => write!(f, "Empty Input"),
			TrailingSlash => write!(f, "Trailing Slash"),
			MissingRightHandExpression => write!(f, "Missing Right Hand Expression"),
			UnexpectedToken(v) => write!(f, "Unexpected Token: {:?}", v),
			InvalidValue(v) => write!(f, "Invalid Value: {:?}", v),
		}
	}
}


impl From<ValueError> for Error {
	fn from(err: ValueError) -> Error {
		Error::InvalidValue(err)
	}
}

#[derive(Debug, Clone)]
pub enum ValueError {
	Boolean,
	Number,
	String,
	Nodeset
}
