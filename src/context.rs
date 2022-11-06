// What we'll be iterating through.

use crate::{Document, Node, Nodeset, AxisName, NodeTest};
use crate::value;


pub struct Evaluation<'a> {
	pub document: &'a Document,
	pub node: &'a Node,

	pub position: usize,
	pub size: usize
}



impl<'a> Evaluation<'a> {
	pub fn new(node: &'a Node, document: &'a Document) -> Evaluation<'a> {
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

	pub fn find_nodes(&self, context: &AxisName, node_test: &dyn NodeTest, prev_step_axis: Option<AxisName>) -> Nodeset {
		let mut nodeset = Nodeset::new();

		match context {
			AxisName::Ancestor => {
				if let Some(parent) = self.node.parent() {
					let eval = self.new_evaluation_from(&parent);
					node_test.test(&eval, &mut nodeset);
					eval.find_nodes(&AxisName::Ancestor, node_test, prev_step_axis);
				}
			}

			AxisName::AncestorOrSelf => {
				nodeset.extend(self.find_nodes(&AxisName::SelfAxis, node_test, prev_step_axis));
				nodeset.extend(self.find_nodes(&AxisName::Ancestor, node_test, prev_step_axis));
			}

			AxisName::Attribute => {
				if let Node::Element(node) = &self.node {
					if let Some(attrs) = value::Attribute::from_node(node) {
						attrs.into_iter()
						.map(Node::Attribute)
						.for_each(|node| {
							node_test.test(
								&self.new_evaluation_from(&node),
								&mut nodeset
							);
						});
					}
				}
			}

			AxisName::Child => {
				// If our previous step was DescendantOrSelf that means we're going through all its' children
				// so we'll just check out the current node to ensure it doesn't return nodes out of order.
				if prev_step_axis == Some(AxisName::DescendantOrSelf) {
					let new_context = self.new_evaluation_from(self.node);
					node_test.test(&new_context, &mut nodeset);
				} else {
					for child in self.node.children() {
						let new_context = self.new_evaluation_from(&child);
						node_test.test(&new_context, &mut nodeset);
					}
				}
			}

			AxisName::Descendant => {
				for child in self.node.children() {
					let new_context = self.new_evaluation_from(&child);

					node_test.test(&new_context, &mut nodeset);

					nodeset.extend(new_context.find_nodes(&AxisName::Descendant, node_test, prev_step_axis));
				}
			}

			AxisName::DescendantOrSelf => {
				nodeset.extend(self.find_nodes(&AxisName::SelfAxis, node_test, prev_step_axis));
				nodeset.extend(self.find_nodes(&AxisName::Descendant, node_test, prev_step_axis));
			}

			// excluding any descendants and excluding attribute nodes and namespace nodes
			AxisName::Following => {
				// Returns children in current parent after 'self.node'.
				value::following_nodes_from_parent(self.node)
				.into_iter()
				.for_each(|node| nodeset.extend(
					self.new_evaluation_from(&node)
					.find_nodes(&AxisName::DescendantOrSelf, node_test, prev_step_axis)
				));

				// Get the parents children after 'self.node.parent()'
				if let Some(parent) = self.node.parent() {
					nodeset.extend(
						self.new_evaluation_from(&parent)
						.find_nodes(&AxisName::Following, node_test, prev_step_axis)
					);
				}
			}

			// if the context node is an attribute node or namespace node, the following-sibling axis is empty
			AxisName::FollowingSibling => {
				// Returns children in current parent after 'self.node'.
				nodeset.extend(
					value::following_nodes_from_parent(self.node)
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
				value::preceding_nodes_from_parent(self.node)
				.into_iter()
				.for_each(|node| nodeset.extend(
					self.new_evaluation_from(&node)
					.find_nodes(&AxisName::DescendantOrSelf, node_test, prev_step_axis)
				));

				// Get the parents children before 'self.node.parent()'
				if let Some(parent) = self.node.parent() {
					nodeset.extend(
						self.new_evaluation_from(&parent)
						.find_nodes(&AxisName::Preceding, node_test, prev_step_axis)
					);
				}
			}

			// if the context node is an attribute node or namespace node, the preceding-sibling axis is empty
			AxisName::PrecedingSibling => {
				// Returns children in current parent before 'self.node'.
				nodeset.extend(
					value::preceding_nodes_from_parent(self.node)
					.into_iter()
					.collect::<Vec<Node>>()
					.into()
				);
			}

			AxisName::SelfAxis => {
				node_test.test(self, &mut nodeset);
			}
		}

		nodeset
	}

	pub fn new_evaluation_from(&'a self, node: &'a Node) -> Self {
		Self {
			document: self.document,
			node,
			position: 1,
			size: 1
		}
	}
}