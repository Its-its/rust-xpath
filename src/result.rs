use crate::ExprToken;


pub type Result<I> = std::result::Result<I, Error>;


#[derive(Debug, Clone)]
pub enum Error {
	Token,
	InputEmpty,
	TrailingSlash,
	MissingRightHandExpression,
	UnexpectedToken(ExprToken)
}



//