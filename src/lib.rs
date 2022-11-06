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

pub(crate) use context::Evaluation;
pub(crate) use value::{Node, Nodeset};
pub(crate) use tokens::{ExprToken, AxisName, NodeType, Operator, PrincipalNodeType};
pub(crate) use parser::Tokenizer;
pub(crate) use nodetest::{NodeTest, NameTest};

pub use result::{Result, Error};
pub use value::Value;
pub use factory::{Factory, Document};


pub fn parse_document<R: std::io::Read>(data: &mut R) -> Result<Document> {
	let parse: markup5ever_rcdom::RcDom = html5ever::parse_document(markup5ever_rcdom::RcDom::default(), Default::default())
		.from_utf8()
		.read_from(data)?;

	Ok(Document::new(parse.document.into()))
}


pub fn compile_lines(node: &Node) -> String {
	let mut items = Vec::new();

	if node.is_text() {
		items.push(format!("{:?}", node_name(node)));
	} else {
		items.push(node_name(node));
	}

	fn iter_through(parent: Option<Node>, items: &mut Vec<String>) {
		if let Some(item) = parent {
			let node_parent = item.parent();

			if node_parent.is_some() {
				items.push(node_name(&item));
			} else {
				items.push(String::from("ROOT"));
			}

			iter_through(node_parent, items);
		}
	}

	iter_through(node.parent(), &mut items);

	items.reverse();

	items.join("/")
}

fn node_name(node: &Node) -> String {
	if let Some(mut name) = node.as_simple_html() {
		let found = name.find(|c| c == '>');

		if let Some(found) = found {
			name.truncate(found + 1);
		}

		name
	} else {
		String::from("ROOT")
	}
}

#[cfg(test)]
mod tests {
	#![allow(dead_code)]

	use std::io::Cursor;

	use tracing::debug;

pub use crate::nodetest::{NodeTest, NameTest};
	pub use crate::result::{Result, Error};
	pub use crate::value::{Value, Node, Nodeset};
	pub use crate::tokens::{ExprToken, AxisName, NodeType, Operator, PrincipalNodeType};
	pub use crate::context::Evaluation;
	pub use crate::parser::Tokenizer;
	pub use crate::factory::{Factory, Document};
	pub use crate::parse_document;


	const WEBPAGE: &str = r#"
		<!DOCTYPE html>
		<html lang="en">
			<head>
				<meta charset="UTF-8">
				<meta http-equiv="X-UA-Compatible" content="IE=edge">
				<meta name="viewport" content="width=device-width, initial-scale=1.0">
				<title>Document</title>
			</head>
			<body>
				<div class="test1">Testing 1</div>
				<span class="test2">Testing 2</span>
				<span class="test3">Testing 3</span>
				<a>Maybe</a>
				<div class="group1" aria-label="Watch Out!">
					<h1>The Group is here!</h1>
					<br/>
					<a class="clickable1">Don't click!</a>
				</div>
				<a class="clickable2">
					<img src="" alt="unable to display" />
				</a>
				<div class="group2" aria-label="Come in!">
					<a class="clickable1">Open Here!</a>
					<img src="" alt="unable to display" />
				</div>
			</body>
		</html>"#;

	fn evaluate(doc: &Document, search: &str) -> Option<Result<Value>> {
		doc.evaluate(search)
			.and_then(|mut v| v.next().transpose())
			.transpose()
	}

	fn assert_is_some(doc: &Document, search: &str) {
		assert!(evaluate(doc, search).is_some(), "IS SOME {:?}", search);
	}

	fn assert_is_none(doc: &Document, search: &str) {
		assert!(evaluate(doc, search).is_none(), "IS NONE {:?}", search);
	}

	fn assert_is_error(doc: &Document, search: &str) {
		assert_eq!(evaluate(doc, search).map(|v| v.is_err()), Some(true), "IS ERR {:?}", search);
	}

	fn assert_is_ok(doc: &Document, search: &str) {
		assert_eq!(evaluate(doc, search).map(|v| v.is_ok()), Some(true), "IS OK {:?}", search);
	}

	fn assert_eq_count(doc: &Document, search: &str, value: usize) {
		assert_eq!(doc.evaluate(search).map(|v| v.count()), Ok(value), "Count {:?}", search);
	}

	fn assert_eq_eval<I: Into<Value>>(doc: &Document, search: &str, value: I) {
		assert_eq!(evaluate(doc, search), Some(Ok(value.into())), "Eval EQ OK {:?}", search);
	}

	fn assert_eq_eval_to_string<I: ToString>(doc: &Document, search: &str, value: I) {
		assert_eq!(evaluate(doc, search).map(|v| v.and_then(|v| v.convert_to_string())), Some(Ok(value.to_string())), "Eval EQ OK {:?}", search);
	}

	fn assert_eq_err(doc: &Document, search: &str, value: Error) {
		assert_eq!(evaluate(doc, search), Some(Err(value)), "Eval EQ ERR {:?}", search);
	}


	#[test]
	fn expressions() {
		let doc = parse_document(&mut Cursor::new(WEBPAGE)).unwrap();

		// Simple
		assert_eq_eval(&doc, r#"1 + 1"#, 2.0);
		assert_eq_eval(&doc, r#"0 - 2"#, -2.0);

		assert_eq_eval(&doc, r#"-2"#, -2.0);

		assert_eq_eval(&doc, r#"1 != 1"#, false);
		assert_eq_eval(&doc, r#"1 != 2"#, true);

		assert_eq_eval(&doc, r#"1 = 1"#, true);
		assert_eq_eval(&doc, r#"1 = 2"#, false);

		assert_eq_eval(&doc, r#"2 > 1"#, true);
		assert_eq_eval(&doc, r#"1 > 2"#, false);
		// assert_eq_eval(&doc, r#"3 > 2 > 1"#, false);
		// assert_eq_eval(&doc, r#"1 > 2 > 3"#, false);

		assert_eq_eval(&doc, r#"2 < 1"#, false);
		assert_eq_eval(&doc, r#"1 < 2"#, true);

		assert_eq_eval(&doc, r#"2 >= 1"#, true);
		assert_eq_eval(&doc, r#"1 >= 1"#, true);

		assert_eq_eval(&doc, r#"2 <= 1"#, false);
		assert_eq_eval(&doc, r#"1 <= 1"#, true);


		// NaN (using true/false since NaNs' aren't equal)
		assert_eq!(evaluate(&doc, r#"1 + A"#).and_then(|v| v.ok()?.number().ok()).map(|v| v.is_nan()), Some(true));
		assert_eq!(evaluate(&doc, r#"A + 1"#).and_then(|v| v.ok()?.number().ok()).map(|v| v.is_nan()), Some(true));
	}

	#[test]
	fn paths() {
		let doc = parse_document(&mut Cursor::new(WEBPAGE)).unwrap();


		// == Counting ==

		assert_eq_count(&doc, r#"//div"#, 3);
		assert_eq_count(&doc, r#"//img"#, 2);


		// == Bug corrections ==

		// FIXED BUG: Was causing an error (UnableToFindValue) if element it was comparing against didn't contain class attribute.
		assert_is_ok(&doc, r#"//div[contains(@class, "group2")]"#);
		// FIXED BUG: Wasn't prioritizing going into nested elements.
		assert_eq_eval_to_string(&doc, r#"//a[starts-with(@class, "click")]/@class"#, "clickable1");


		debug!("Location Paths (Unabbreviated Syntax)");
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
		// let doc = parse_document(&mut Cursor::new(WEBPAGE)).unwrap();

		// debug!("Location Paths (Abbreviated Syntax)");
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
		let doc = parse_document(&mut Cursor::new(WEBPAGE)).unwrap();

		// Simple

		assert_eq_eval(&doc, r#"contains("abc123", "bc12")"#, true);
		assert_eq_eval(&doc, r#"contains("abc123", "4")"#, false);

		assert_eq_eval(&doc, r#"concat(true, "123")"#, Value::String("123".into()));
		assert_eq_eval(&doc, r#"concat(false, "123")"#, Value::String("123".into()));
		assert_eq_eval(&doc, r#"concat(1, "123")"#, Value::String("1123".into()));
		assert_eq_eval(&doc, r#"concat("abc", "123")"#, Value::String("abc123".into()));

		// TODO: Below doesn't work.

		// assert_eq_eval(&doc, r#"starts-with("abc123", "abc")"#, true);
		// assert_eq_eval(&doc, r#"starts-with("123", 1)"#, true);

		// assert_eq_eval(&doc, r#"substring-before("abc123", "1")"#, Value::String("abc".into()));

		// assert_eq_eval(&doc, r#"substring-after("abc123", "c")"#, Value::String("123".into()));


		// Document Lookups

		assert_eq_eval(&doc, r#"//div[contains(text(), "Testing 1")]/@class"#, Value::String("test1".into()));

		// debug!("Examples");
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


	#[test]
	fn general_errors() {
		// let doc = parse_document(&mut Cursor::new(WEBPAGE)).unwrap();

		// assert_eq_err(&doc, r#"contains("abc123")"#, Error::FunctionError("alloc::boxed::Box<dyn xpather::functions::Function>".to_string(), Box::new(Error::MissingFuncArgument)));
	}
}