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


pub static DEBUG: bool = false;


pub fn parse_doc<R: std::io::Read>(data: &mut R) -> Document {
	let parse: markup5ever_rcdom::RcDom = html5ever::parse_document(markup5ever_rcdom::RcDom::default(), Default::default())
		.from_utf8()
		.read_from(data)
		.expect("html5ever");

	Document::new(parse.document.into())
}


#[cfg(test)]
mod tests {
	use std::fs::File;

	pub use crate::nodetest::{NodeTest, NameTest};
	pub use crate::result::{Result, Error};
	pub use crate::value::{Value, Node, Nodeset};
	pub use crate::tokens::{ExprToken, AxisName, NodeType, Operator, PrincipalNodeType};
	pub use crate::context::Evaluation;
	pub use crate::parser::Tokenizer;
	pub use crate::factory::{Factory, Document};
	pub use crate::parse_doc;

	#[test]
	fn paths() {
		// let doc = parse_doc(&mut File::open("./doc/example.html").expect("File::open"));

		println!("Location Paths (Unabbreviated Syntax)");
		// assert_eq!(doc.evaluate("//head/title"), Ok(Value::Nodeset(vec![].into()))); // selects the document root (which is always the parent of the document element)
		// dbg!(doc.evaluate("self::para")); // selects the context node if it is a para element, and otherwise selects nothing
		// dbg!(doc.evaluate("child::para")); // selects the para element children of the context node
		// dbg!(doc.evaluate("child::*")); // selects all element children of the context node
		// dbg!(doc.evaluate("child::text()")); // selects all text node children of the context node
		// dbg!(doc.evaluate("child::node()")); // selects all the children of the context node, whatever their node type
		// dbg!(doc.evaluate("child::chapter/descendant::para")); // selects the para element descendants of the chapter element children of the context node
		// dbg!(doc.evaluate("child::*/child::para")); // selects all para grandchildren of the context node
		// dbg!(doc.evaluate("child::para[position()=1]")); // selects the first para child of the context node
		// dbg!(doc.evaluate("child::para[position()=last()]")); // selects the last para child of the context node
		// dbg!(doc.evaluate("child::para[position()=last()-1]")); // selects the last but one para child of the context node
		// dbg!(doc.evaluate("child::para[position()>1]")); // selects all the para children of the context node other than the first para child of the context node
		// dbg!(doc.evaluate("/child::doc/child::chapter[position()=5]/child::section[position()=2]")); // selects the second section of the fifth chapter of the doc document element
		// dbg!(doc.evaluate("child::para[attribute::type=\"warning\"]")); // selects all para children of the context node that have a type attribute with value warning
		// dbg!(doc.evaluate("child::para[attribute::type='warning'][position()=5]")); // selects the fifth para child of the context node that has a type attribute with value warning
		// dbg!(doc.evaluate("child::para[position()=5][attribute::type=\"warning\"]")); // selects the fifth para child of the context node if that child has a type attribute with value warning
		// dbg!(doc.evaluate("child::chapter[child::title='Introduction']")); // selects the chapter children of the context node that have one or more title children with string-value equal to Introduction
		// dbg!(doc.evaluate("child::chapter[child::title]")); // selects the chapter children of the context node that have one or more title children
		// dbg!(doc.evaluate("child::*[self::chapter or self::appendix]")); // selects the chapter and appendix children of the context node
		// dbg!(doc.evaluate("child::*[self::chapter or self::appendix][position()=last()]")); // selects the last chapter or appendix child of the context node
		// dbg!(doc.evaluate("attribute::name")); // selects the name attribute of the context node
		// dbg!(doc.evaluate("attribute::*")); // selects all the attributes of the context node
		// dbg!(doc.evaluate("ancestor::div")); // selects all div ancestors of the context node
		// dbg!(doc.evaluate("ancestor-or-self::div")); // selects the div ancestors of the context node and, if the context node is a div element, the context node as well
		// dbg!(doc.evaluate("following-sibling::chapter[position()=1]")); // selects the next chapter sibling of the context node
		// dbg!(doc.evaluate("preceding-sibling::chapter[position()=1]")); // selects the previous chapter sibling of the context node
		// dbg!(doc.evaluate("descendant::para")); // selects the para element descendants of the context node
		// dbg!(doc.evaluate("descendant-or-self::para")); // selects the para element descendants of the context node and, if the context node is a para element, the context node as well
		// dbg!(doc.evaluate("/descendant::para")); // selects all the para elements in the same document as the context node
		// dbg!(doc.evaluate("/descendant::olist/child::item")); // selects all the item elements that have an olist parent and that are in the same document as the context node
		// dbg!(doc.evaluate("/descendant::figure[position()=42]")); // selects the forty-second figure element in the document
	}

	#[test]
	fn paths_abbreviated() {
		// println!("Location Paths (Abbreviated Syntax)");
		// para selects the para element children of the context node
		// * selects all element children of the context node
		// text() selects all text node children of the context node
		// @name selects the name attribute of the context node
		// @* selects all the attributes of the context node
		// para[1] selects the first para child of the context node
		// para[last()] selects the last para child of the context node
		// */para selects all para grandchildren of the context node
		// /doc/chapter[5]/section[2] selects the second section of the fifth chapter of the doc
		// chapter//para selects the para element descendants of the chapter element children of the context node
		// //para selects all the para descendants of the document root and thus selects all para elements in the same document as the context node
		// //olist/item selects all the item elements in the same document as the context node that have an olist parent
		// . selects the context node
		// .//para selects the para element descendants of the context node
		// .. selects the parent of the context node
		// ../@lang selects the lang attribute of the parent of the context node
		// para[@type="warning"] selects all para children of the context node that have a type attribute with value warning
		// para[@type="warning"][5] selects the fifth para child of the context node that has a type attribute with value warning
		// para[5][@type="warning"] selects the fifth para child of the context node if that child has a type attribute with value warning
		// chapter[title="Introduction"] selects the chapter children of the context node that have one or more title children with string-value equal to Introduction
		// chapter[title] selects the chapter children of the context node that have one or more title children
		// employee[@secretary and @assistant] selects all the employee children of the context node that have both a secretary attribute and an assistant attribute
	}

	#[test]
	fn general_examples() {
		// println!("Examples");
		// dbg!(doc.evaluate("//*[@id='rcTEST']//*[contains(text(), 'TEST Interactive')]/../button[2]"));
		// dbg!(doc.evaluate("//*[@id='rcTEST']//*[contains(text(), 'TEST Interactive')]/..//*[contains(text(), 'Setting')]"));
		// dbg!(doc.evaluate("//*[@id='rcTEST']//*[contains(text(), 'TEST Interactive')]/following-sibling::button"));
		// dbg!(doc.evaluate("// address[@class='ng-scope ng-isolate-scope']//div[contains('Testing') and @id='msgTitle']"));
		// dbg!(doc.evaluate("//*[@name='myForm']//table[@id='tbl_ testdm']/tbody/tr/td[6]/"));
		// dbg!(doc.evaluate("input[@value='Open RFS']"));
		// dbg!(doc.evaluate("//*[@title='Songs List Applet']//table//td[contains(text(),'Author')]"));
		// dbg!(doc.evaluate("//*[@id='parameters']//*[@id='testUpdateTime']"));
		// dbg!(doc.evaluate("//*[@id='MODEL/PLAN']/div[1]/div[2]/div[1]/div[1]/widget/section/div[1]/div/div[1]/div/div/button[1]"));
		// dbg!(doc.evaluate("//*[contains(text(),'Watch Dial')]/../div/select[@data-ng-model='context.questions[subqts.subHandleSubId]']"));
		// dbg!(doc.evaluate("//*[@id='RESEARCH/PLAN']//*[contains(@id, 'A4')]/../../following-sibling::div[1]/div[1]/span[1]/span[1]"));
		// dbg!(doc.evaluate("//*[@id='ALARMDATA']//*[contains(@id, 'AFC2')]/../../preceding-sibling::div[1]/div[1]/span[1]/span[1]"));
		// dbg!(doc.evaluate("//*[@id='RESEARCH/REVIEW']//widget/section/div[1]/div/div[2]/div[1]/div[3]/div[1]//span[@class='details']"));
		// dbg!(doc.evaluate("//a[contains(.,'Parameter Data Manual Entry')]"));
		// dbg!(doc.evaluate("//*[contains(@style,'display: block; top:')]//input[@name='daterangepicker_end']"));
		// dbg!(doc.evaluate("//*[@id='dropdown-filter-serviceTools']/following-sibling::ul/descendant::a[text()='Notepad']"));
		// dbg!(doc.evaluate("//*[@id='dropdown-filter-serviceTools']/following-sibling::ul/descendant::a[text()='Trigger Dashboard']"));
		// dbg!(doc.evaluate("//h3[text()='Internal Debrief']"));
		// dbg!(doc.evaluate("//h3[contains(text(),'Helium Level')]/following-sibling::div/label/input"));
		// dbg!(doc.evaluate("//div[div[p[contains(text(),'Status')]]]/preceding-sibling::div/div/span[3]/span"));
		// dbg!(doc.evaluate("//*[@id='COUPLING']//*[contains(text(),'COUPLE Trend')]/../div/select"));
		// dbg!(doc.evaluate("//*[@id='ffaHeaderDropdown']//a[contains(text(),'Start Workflow')]"));
	}
}