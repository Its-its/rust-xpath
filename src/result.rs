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
