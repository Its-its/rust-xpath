// What we'll be iterating through.

use crate::{Document, Node, Nodeset, AxisName, NodeTest};
use crate::value;


pub struct Evaluation<'a> {
	pub document: &'a Document,
	pub node: Node,

	pub position: usize,
	pub size: usize
}



impl<'a> Evaluation<'a> {
	pub fn new(node: Node, document: &'a Document) -> Evaluation<'a> {
		Evaluation {
			document,
			node,
			position: 1,
			size: 1
		}
	}

	pub fn root(&'a self) -> &'a Node {
		&self.document.root
	}

	pub fn find_nodes(&self, context: &AxisName, node_test: &dyn NodeTest) -> Nodeset {
		let mut nodeset = Nodeset::new();

		match context {
			AxisName::Ancestor => {
				if let Some(parent) = self.node.parent() {
					let eval = self.new_evaluation_from(parent);
					node_test.test(&eval, &mut nodeset);
					eval.find_nodes(&AxisName::Ancestor, node_test);
				}
			}

			AxisName::AncestorOrSelf => {
				nodeset.extend(self.find_nodes(&AxisName::SelfAxis, node_test));
				nodeset.extend(self.find_nodes(&AxisName::Ancestor, node_test));
			}

			AxisName::Attribute => {
				if let Node::Element(node) = &self.node {
					if let Some(attrs) = value::Attribute::from_node(node) {
						attrs.into_iter()
						.map(Node::Attribute)
						.for_each(|node| {
							node_test.test(
								&self.new_evaluation_from(node),
								&mut nodeset
							);
						});
					}
				}
			}

			AxisName::Child => {
				for child in self.node.children() {
					let new_context = self.new_evaluation_from(child);
					node_test.test(&new_context, &mut nodeset);
				}
			}

			AxisName::Descendant => {
				for child in self.node.children() {
					let new_context = self.new_evaluation_from(child);

					node_test.test(&new_context, &mut nodeset);

					nodeset.extend(new_context.find_nodes(&AxisName::Descendant, node_test));
				}
			}

			AxisName::DescendantOrSelf => {
				nodeset.extend(self.find_nodes(&AxisName::SelfAxis, node_test));
				nodeset.extend(self.find_nodes(&AxisName::Descendant, node_test));
			}

			// excluding any descendants and excluding attribute nodes and namespace nodes
			AxisName::Following => {
				// Returns children in current parent after 'self.node'.
				value::following_nodes_from_parent(&self.node)
				.into_iter()
				.for_each(|node| nodeset.extend(
					self.new_evaluation_from(node)
					.find_nodes(&AxisName::DescendantOrSelf, node_test)
				));

				// Get the parents children after 'self.node.parent()'
				if let Some(parent) = self.node.parent() {
					nodeset.extend(
						self.new_evaluation_from(parent)
						.find_nodes(&AxisName::Following, node_test)
					);
				}
			}

			// if the context node is an attribute node or namespace node, the following-sibling axis is empty
			AxisName::FollowingSibling => {
				// Returns children in current parent after 'self.node'.
				nodeset.extend(
					value::following_nodes_from_parent(&self.node)
					.into_iter()
					.collect::<Vec<Node>>()
					.into()
				);
			}

			// contains the namespace nodes of the context node;
			// the axis will be empty unless the context node is an element
			AxisName::Namespace => {
				unimplemented!("AxisName::Namespace")
			}

			AxisName::Parent => {
				if let Some(p_node) = self.node.parent() {
					nodeset.add_node(p_node);
				}
			}

			// excluding any ancestors and excluding attribute nodes and namespace nodes
			AxisName::Preceding => {
				// Returns children in current parent before 'self.node'.
				value::preceding_nodes_from_parent(&self.node)
				.into_iter()
				.for_each(|node| nodeset.extend(
					self.new_evaluation_from(node)
					.find_nodes(&AxisName::DescendantOrSelf, node_test)
				));

				// Get the parents children before 'self.node.parent()'
				if let Some(parent) = self.node.parent() {
					nodeset.extend(
						self.new_evaluation_from(parent)
						.find_nodes(&AxisName::Preceding, node_test)
					);
				}
			}

			// if the context node is an attribute node or namespace node, the preceding-sibling axis is empty
			AxisName::PrecedingSibling => {
				// Returns children in current parent before 'self.node'.
				nodeset.extend(
					value::preceding_nodes_from_parent(&self.node)
					.into_iter()
					.collect::<Vec<Node>>()
					.into()
				);
			}

			AxisName::SelfAxis => {
				node_test.test(&self, &mut nodeset);
			}
		}

		nodeset
	}

	pub fn new_evaluation_from(&'a self, node: Node) -> Self {
		Self {
			document: self.document,
			node,
			position: 1,
			size: 1
		}
	}

	pub fn new_evaluation_set_from(&'a self, nodes: Nodeset) -> EvaluationNodesetIter<'a> {
		EvaluationNodesetIter {
			parent: self,
			size: nodes.nodes.len(),
			nodes: nodes.nodes.into_iter().enumerate(),
		}
	}
}

pub struct EvaluationNodesetIter<'a> {
	parent: &'a Evaluation<'a>,
	nodes: std::iter::Enumerate<std::vec::IntoIter<Node>>,
	size: usize
}

impl<'a> Iterator for EvaluationNodesetIter<'a> {
    type Item = Evaluation<'a>;

    fn next(&mut self) -> Option<Evaluation<'a>> {
        if let Some((idx, node)) = self.nodes.next() {
			Some(Evaluation {
				document: self.parent.document,
				node,
				position: idx + 1,
				size: self.size
			})
		} else {
			None
		}
    }
}