use std::io::Cursor;

use html5ever::tendril::TendrilSink;
use tracing::{Level, debug};
use tracing::subscriber::set_global_default;
use tracing_subscriber::FmtSubscriber;
use xpather::result::Result;
use xpather::factory::Document;


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
				<a class="clickable3">Open Here!</a>
				<img src="" alt="unable to display" />
			</div>
		</body>
	</html>"#;

pub fn main() -> Result<()> {
	let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_file(false)
        .with_line_number(true)
        .finish();

    set_global_default(subscriber).expect("setting default subscriber failed");

	let document = parse_doc(&mut Cursor::new(WEBPAGE));

	let mut eval = document.evaluate(
		r#"//a[starts-with(@class, "click")]/@class"#
	)?;

	debug!("{:?}", eval.next());

	// let factory = Factory::new(r#"2 + A"#, &doc, &doc.root);

	// let now = Instant::now();

	// let mut prod = factory.produce()?;

	// debug!("{:?}", now.elapsed());

	// let now = Instant::now();

	// debug!("Output");

	// // debug!("{:#?}", prod.collect_nodes());
	// debug!("{:#?}", prod.next());

	// debug!("{:?}", now.elapsed());

	Ok(())
}


pub fn parse_doc<R: std::io::Read>(data: &mut R) -> Document {
	let parse: markup5ever_rcdom::RcDom = html5ever::parse_document(markup5ever_rcdom::RcDom::default(), Default::default())
		.from_utf8()
		.read_from(data)
		.expect("html5ever");

	Document::new(parse.document.into())
}