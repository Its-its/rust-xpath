use std::fs::File;
use html5ever::tendril::TendrilSink;


pub mod value;
pub mod result;
pub mod parser;
pub mod tokens;
pub mod context;
pub mod factory;
pub mod functions;
pub mod expressions;
pub mod nodetest;

pub use nodetest::{NodeTest, NameTest};
pub use result::{Result, Error};
pub use value::{Value, Node, Nodeset};
pub use tokens::{ExprToken, AxisName, NodeType, Operator, PrincipalNodeType};
pub use context::Evaluation;
pub use parser::Tokenizer;
pub use factory::{Factory, Document};


pub fn parse_doc<R: std::io::Read>(data: &mut R) -> Document {
	let parse: markup5ever_rcdom::RcDom = html5ever::parse_document(markup5ever_rcdom::RcDom::default(), Default::default())
		.from_utf8()
		.read_from(data)
		.expect("html5ever");

	Document::new(parse.document.into())
}