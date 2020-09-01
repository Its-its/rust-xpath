use std::cell::Cell;
use std::rc::Rc;
use std::fmt;

use markup5ever::{Attribute as DomAttribute, QualName};
use markup5ever_rcdom::{NodeData, Handle as NodeHandle, WeakHandle as WeakNodeHandle, SerializableHandle};
use html5ever::serialize;

use crate::Document;

#[derive(Debug, Clone)]
pub enum Value {
	Boolean(bool),
	Number(f64),
	String(String),
	Nodeset(Nodeset)
}

impl Value {
	pub fn exists(&self) -> bool {
		match self {
			Value::Boolean(v) => *v,
			Value::Number(v) => !v.is_nan(),
			Value::String(v) => !v.is_empty(),
			Value::Nodeset(v) => !v.nodes.is_empty()
		}
	}

	pub fn into_nodeset(self) -> Nodeset {
		match self {
			Value::Nodeset(s) =>  s,
			_ => panic!("Value is NOT a Nodeset. Cannot convert into it.")
		}
	}

	pub fn into_iterset(self) -> NodeIterset {
		match self {
			Value::Nodeset(s) =>  NodeIterset::new(s.into_iter()),
			_ => panic!("Value is NOT a Nodeset. Cannot convert into it.")
		}
	}

	pub fn vec_string(self) -> Vec<String> {
		self.into_iterset()
		.map(|i| i.value())
		.map(|i| i.string())
		.collect()
	}

	pub fn boolean(self) -> bool {
		match self {
			Value::Boolean(v) =>  v,
			_ => panic!("Value is NOT a Boolean. Cannot convert into it.")
		}
	}

	pub fn number(self) -> f64 {
		match self {
			Value::Number(v) =>  v,
			_ => panic!("Value is NOT a Number. Cannot convert into it.")
		}
	}

	pub fn string(self) -> String {
		match self {
			Value::String(v) =>  v,
			_ => panic!("Value is NOT a String. Cannot convert into it.")
		}
	}
}

impl PartialEq for Value {
	fn eq(&self, other: &Value) -> bool {
		match (self, other) {
			// Noteset == String
			(Value::Nodeset(set), Value::String(value)) |
			(Value::String(value), Value::Nodeset(set)) => {

				if set.nodes.is_empty() {
					return false;
				}

				set.nodes.iter()
				.find(|node| {
					match node {
						Node::Attribute(attr) => {
							attr.value() == value
						}

						_ => false
					}
				}).is_some()
			}

			_ => false
		}
	}
}

// node-set (an unordered collection of nodes without duplicates)
// boolean (true or false)
// number (a floating-point number)
// string (a sequence of UCS characters)


#[derive(Clone)]
pub struct Attribute {
	pub parent: WeakNodeHandle,
	pub attr: DomAttribute
}

impl Attribute {
	pub fn new(parent: WeakNodeHandle, attr: DomAttribute) -> Self {
		Self {
			parent,
			attr
		}
	}

	pub fn from_node(node: &WeakNodeHandle) -> Vec<Attribute> {
		if let NodeData::Element { attrs, .. } = &node.upgrade().unwrap().data {
			attrs.borrow().iter().map(|a| Attribute::new(node.clone(), a.clone())).collect()
		} else {
			panic!("Node is not an Element for Attribute.")
		}
	}

	pub fn name(&self) -> &QualName {
		&self.attr.name
	}

	pub fn name_string(&self) -> String {
		let mut comp = String::new();

		if let Some(prefix) = &self.attr.name.prefix {
			comp.push_str(&prefix);
			comp.push_str(":");
		}

		comp.push_str(&self.attr.name.local);

		comp
	}

	pub fn value(&self) -> &str {
		&*self.attr.value
	}
}


// TODO: Convert to
// pub struct Node(WeakNodeHandle);
// - No way to know if it's an Attribute though.
#[derive(Clone)]
pub enum Node {
	Root(NodeHandle),
    DocType(WeakNodeHandle),
    Element(WeakNodeHandle),
    Attribute(Attribute),
    Text(WeakNodeHandle),
    Comment(WeakNodeHandle),
    ProcessingInstruction(WeakNodeHandle),
    Namespace(WeakNodeHandle), // Mainly used for xml
}

impl Node {
	pub fn enum_name(&self) -> String {
		match self {
			Node::DocType(_) => "DocType".into(),
			Node::Namespace(_) => "Namespace".into(),
			Node::Root(_) => "Root".into(),
			Node::Element(_) => "Element".into(),
			Node::Attribute(_) => "Attribute".into(),
			Node::Text(_) => "Text".into(),
			Node::Comment(_) => "Comment".into(),
			Node::ProcessingInstruction(_) => "ProcessingInstruction".into(),
		}
	}

	pub fn is_root(&self) -> bool {
		match self {
			Node::Root(_) => true,
			_ => false
		}
	}

	pub fn is_namespace(&self) -> bool {
		match self {
			Node::Namespace(_) => true,
			_ => false
		}
	}

	pub fn is_element(&self) -> bool {
		match self {
			Node::Element(_) => true,
			_ => false
		}
	}

	pub fn is_attribute(&self) -> bool {
		match self {
			Node::Attribute(_) => true,
			_ => false
		}
	}

	pub fn is_text(&self) -> bool {
		match self {
			Node::Text(_) => true,
			_ => false
		}
	}

	pub fn is_comment(&self) -> bool {
		match self {
			Node::Comment(_) => true,
			_ => false
		}
	}

	pub fn is_processing_instruction(&self) -> bool {
		match self {
			Node::ProcessingInstruction(_) => true,
			_ => false
		}
	}

	pub fn value(&self) -> Value {
		match self {
			Node::Attribute(attr) => {
				Value::String(attr.value().to_string())
			}

			Node::Text(node) => {
				if let NodeData::Text { contents } = &node.upgrade().unwrap().data {
					Value::String(contents.borrow().to_string())
				} else {
					panic!()
				}
			}

			_ => panic!("Node not convertable into a Value")
		}
	}

	pub fn as_simple_html(&self) -> String {
		match self {
			Node::Attribute(attr) => {
				format!("@{}={}", attr.name_string(), attr.value())
			}

			_ => {
				let mut st = Vec::new();

				let write = std::io::Cursor::new(&mut st);

				serialize::<_, SerializableHandle>(
					write,
					&self.inner_weak().upgrade().unwrap().into(),
					html5ever::serialize::SerializeOpts { traversal_scope: markup5ever::serialize::TraversalScope::IncludeNode, .. Default::default() })
				.expect("serialzing error");

				String::from_utf8(st).expect("from_utf8 error")
			}
		}
	}

	pub fn attribute(&self) -> &Attribute {
		match self {
			Node::Attribute(attr) => attr,
			_ => panic!("Node::attribute()")
		}
	}

	pub fn parent(&self) -> Option<Node> {
		match self {
			Node::Attribute(attr) => attr.parent.upgrade()
				.and_then(|node| get_opt_node_from_cell(&node.parent).map(|i| Node::Element(i))),
			Node::DocType(_) |
			Node::Namespace(_) |
			Node::Root(_) => None,
			Node::Element(weak) => weak.upgrade()
				.and_then(|node| get_opt_node_from_cell(&node.parent).map(|i| Node::Element(i))),
			Node::Text(weak) => weak.upgrade()
				.and_then(|node| get_opt_node_from_cell(&node.parent).map(|i| Node::Text(i))),
			Node::Comment(weak) => weak.upgrade()
				.and_then(|node| get_opt_node_from_cell(&node.parent).map(|i| Node::Comment(i))),
			Node::ProcessingInstruction(weak) => weak.upgrade()
				.and_then(|node| get_opt_node_from_cell(&node.parent).map(|i| Node::ProcessingInstruction(i)))
		}
	}

	pub fn children(&self) -> Vec<Node> {
		match self {
			Node::Root(handle) => {
				let node = handle.as_ref();

				let items = node.children.borrow()
				.iter()
				.map(|c| c.into())
				.collect();

				items
			}

			Node::Text(handle) |
			Node::Comment(handle) |
			Node::DocType(handle) |
			Node::Element(handle) => {
				let node = handle.upgrade().unwrap();

				let items = node.children.borrow()
				.iter()
				.map(|c| c.into())
				.collect();

				items
			}

			_ => unimplemented!("Node::children(\"{}\")", self.enum_name())
		}
	}


	pub fn name(&self) -> QualName {
		match self {
			Node::Element(node) => {
				if let NodeData::Element { name, .. } = &node.upgrade().unwrap().data {
					name.clone()
				} else {
					panic!("Name")
				}
			}

			Node::Attribute(attr) => {
				if let NodeData::Element { name, .. } = &attr.parent.upgrade().unwrap().data {
					name.clone()
				} else {
					panic!("Name")
				}
			}

			_ => panic!("Name")
		}
	}

	pub fn target(&self) -> String {
		match self {
			Node::ProcessingInstruction(node) => {
				if let NodeData::ProcessingInstruction { target, .. } = &node.upgrade().unwrap().data {
					target.to_string()
				} else {
					panic!("Name")
				}
			}

			_ => panic!("Name")
		}
	}

	pub fn prefix(&self) -> String {
		unimplemented!("Node::prefix()");
	}

	pub fn inner_weak(&self) -> &WeakNodeHandle {
		match self {
			Node::Root(..) => panic!(),
			Node::DocType(weak) |
			Node::Namespace(weak) |
			Node::Element(weak) |
			Node::Text(weak) |
			Node::Comment(weak) |
			Node::ProcessingInstruction(weak) => weak,
			Node::Attribute(weak) => &weak.parent
		}
	}


	pub fn evaluate_from<S: Into<String>>(&self, search: S, doc: &Document) -> Option<Value> {
		doc.evaluate_from(search, self.clone())
	}
}

impl From<&NodeHandle> for Node {
	fn from(handle: &NodeHandle) -> Self {
		match &handle.data {
			NodeData::Comment{ .. } => {
				Node::Comment(Rc::downgrade(handle))
			}

			NodeData::Document => {
				panic!("Cannot convert borrowed Document to Node.")
			}

			NodeData::Element{ .. } => {
				Node::Element(Rc::downgrade(handle))
			}

			NodeData::ProcessingInstruction{ .. } => {
				Node::ProcessingInstruction(Rc::downgrade(handle))
			}

			NodeData::Text{ .. } => {
				Node::Text(Rc::downgrade(handle))
			}

			NodeData::Doctype { .. } => {
				Node::DocType(Rc::downgrade(handle))
			}

			i @ _ => panic!("From NodeHandle: {:?}", i)
		}
	}
}

impl From<NodeHandle> for Node {
	fn from(handle: NodeHandle) -> Self {
		match handle.data {
			NodeData::Comment{ .. } => {
				Node::Comment(Rc::downgrade(&handle))
			}

			NodeData::Document => {
				Node::Root(handle)
			}

			NodeData::Element{ .. } => {
				Node::Element(Rc::downgrade(&handle))
			}

			NodeData::Doctype { .. } => {
				Node::DocType(Rc::downgrade(&handle))
			}

			NodeData::ProcessingInstruction{ .. } => {
				Node::ProcessingInstruction(Rc::downgrade(&handle))
			}

			NodeData::Text{ .. } => {
				Node::Text(Rc::downgrade(&handle))
			}

			_ => panic!("From NodeHandle")
		}
	}
}

impl PartialEq for Node {
	fn eq(&self, other: &Node) -> bool {
		if self.is_root() || other.is_root() {
			return self.is_root() == other.is_root();
		}

		compare_weak_nodes(self.inner_weak(), other.inner_weak())
	}
}

pub fn compare_weak_nodes(left: &WeakNodeHandle, right: &WeakNodeHandle) -> bool {
	let left_upgrade = left.upgrade().unwrap();
	let right_upgrade = left.upgrade().unwrap();

	compare_nodes(&left_upgrade, &right_upgrade)
}


pub fn following_nodes_from_parent(node: &Node) -> Vec<Node> {
	find_nodes_from_parent(node, |child_pos, node_pos| child_pos > node_pos)
}

pub fn preceding_nodes_from_parent(node: &Node) -> Vec<Node> {
	find_nodes_from_parent(node, |child_pos, node_pos| child_pos < node_pos)
}

fn find_nodes_from_parent<F: Fn(usize, usize) -> bool>(node: &Node, f_capture: F) -> Vec<Node> {
	let node = node.inner_weak().upgrade().unwrap();

	// Taken from markup5ever_rcdom
	if let Some(weak) = node.parent.take() {
		let parent = weak.upgrade().expect("dangling weak pointer");
		node.parent.set(Some(weak));

		let children = parent.children.borrow();

		let i = match children
			.iter()
			.enumerate()
			.find(|&(_, child)| Rc::ptr_eq(&child, &node))
		{
			Some((i, _)) => i,
			None => panic!("have parent but couldn't find in parent's children!"),
		};

		children
		.iter()
		.enumerate()
		.filter(|c| f_capture(c.0, i))
		.map(|i| i.1.into())
		.collect()
	} else {
		Vec::new()
	}
}



pub fn compare_nodes(left_upgrade: &NodeHandle, right_upgrade: &NodeHandle) -> bool {
	let matched = match (&left_upgrade.data, &right_upgrade.data) {
		(
			NodeData::Text {
				contents: b_contents
			},
			NodeData::Text {
				contents
			}
		) => {
			b_contents == contents
		}

		(
			NodeData::Comment {
				contents: b_contents
			},
			NodeData::Comment {
				contents
			}
		) => {
			b_contents == contents
		}

		(
			NodeData::Doctype {
				name: b_name,
				public_id: b_public_id,
				system_id: b_system_id
			},
			NodeData::Doctype {
				name,
				public_id,
				system_id
			}
		) => {
			b_name == name ||
			b_public_id == public_id ||
			b_system_id == system_id
		}

		(
			NodeData::Element {
				name: b_name,
				attrs: b_attr,
				template_contents: b_template_contents,
				mathml_annotation_xml_integration_point: b_mathml
			},
			NodeData::Element {
				name,
				attrs,
				template_contents,
				mathml_annotation_xml_integration_point
			}
		) => {
			b_name == name ||
			b_attr == attrs ||
			Some((b_template_contents, template_contents))
			.filter(|c| c.0.is_some() || c.1.is_some())
			.map(|i| compare_nodes(i.0.as_ref().unwrap(), i.1.as_ref().unwrap()))
			.unwrap_or_default() ||
			b_mathml == mathml_annotation_xml_integration_point
		}

		(
			NodeData::ProcessingInstruction {
				target: b_target,
				contents: b_contents
			},
			NodeData::ProcessingInstruction {
				target,
				contents
			}
		) => {
			b_target == target ||
			b_contents == contents
		}

		_ => false
	};

	if matched {
		return true;
	}

	// Compare children
	let l_children = left_upgrade.children.borrow();
	let r_children = right_upgrade.children.borrow();

	if l_children.len() != r_children.len() {
		return false;
	}

	// Find first position where it's false.
	// If we found a non-equal child it'll return Some(pos)
	// So we need to ensure it's None
	l_children.iter()
	.zip(r_children.iter())
	.position(|c| !compare_nodes(c.0, c.1))
	.is_none()
}


// impl From<Attribute> for Node {
// 	fn from(handle: Attribute) -> Self {
// 		Node::Attribute(handle)
// 	}
// }

// impl From<&Attribute> for Node {
// 	fn from(handle: &Attribute) -> Self {
// 		Node::Attribute(handle.clone())
// 	}
// }

impl fmt::Debug for Node {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Node::Root(weak) => {
				f.debug_tuple("Root")
					.field(&weak)
					.finish()
			}

			Node::Attribute(weak) => {
				f.debug_tuple("Attribute")
					.field(&weak.parent.upgrade().unwrap().data)
					.finish()
			}

			Node::DocType(weak) |
			Node::Element(weak) |
			Node::Namespace(weak) |
			Node::Text(weak) |
			Node::Comment(weak) |
			Node::ProcessingInstruction(weak) => {
				f.debug_tuple("Node")
					.field(&weak.upgrade().unwrap().data)
					.finish()
			}
		}

	}
}

// TODO: Ensure no duplicate nodes
#[derive(Clone)]
pub struct Nodeset {
	pub nodes: Vec<Node>
}

impl Nodeset {
	pub fn new() -> Self {
		Nodeset {
			nodes: Vec::new()
		}
	}

	pub fn add_node_handle(&mut self, node: &NodeHandle) {
		self.nodes.push(node.into());
	}

	pub fn add_node(&mut self, node: Node) {
		self.nodes.push(node);
	}

	pub fn extend(&mut self, nodeset: Nodeset) {
		self.nodes.extend(nodeset.nodes);
	}
}

impl IntoIterator for Nodeset {
	type Item = Node;
	type IntoIter = std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		self.nodes.into_iter()
	}
}

impl From<Vec<Node>> for Nodeset {
	fn from(nodes: Vec<Node>) -> Self {
		Self {
			nodes
		}
	}
}

impl fmt::Debug for Nodeset {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut list = f.debug_list();

		self.nodes.iter()
		.for_each(|node| {
			list.entry(&node.as_simple_html());
		});


		list.finish()
	}
}

pub struct NodeIterset(std::vec::IntoIter<Node>);

impl NodeIterset {
	pub fn new(set: std::vec::IntoIter<Node>) -> Self {
		Self(set)
	}
}

impl Iterator for NodeIterset {
	type Item = Node;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

pub struct Valueset(Vec<Value>);

impl Valueset {
	//
}


pub fn get_opt_node_from_cell(cell: &Cell<Option<WeakNodeHandle>>) -> Option<WeakNodeHandle> {
	let item = cell.take();

	let cloned = item.clone();

	cell.set(item);

	cloned
}