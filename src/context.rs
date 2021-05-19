// What we'll be iterating through.

use crate::{Document, Node, Nodeset, AxisName, NodeTest};
use crate::value;

#[derive(Debug)]
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

	// TODO: Create an "NodeSearchState" struct so we can continue from where we left off.
	pub fn find_nodes(&self, context: &AxisName, node_test: &dyn NodeTest) -> Nodeset {
		let mut nodeset = Nodeset::new();

		match context {
			AxisName::Ancestor => {
				if let Some(parent) = self.node.parent() {
					let eval = self.new_evaluation_from(parent);

					if let Some(node) = node_test.test(&eval) {
						nodeset.add_node(node);
					}

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
							if let Some(node) = node_test.test(&self.new_evaluation_from(node)) {
								nodeset.add_node(node);
							}
						});
					}
				}
			}

			AxisName::Child => {
				for child in self.node.children() {
					let new_context = self.new_evaluation_from(child);
					if let Some(node) = node_test.test(&new_context) {
						nodeset.add_node(node);
					}
				}
			}

			AxisName::Descendant => {
				for child in self.node.children() {
					let new_context = self.new_evaluation_from(child);

					if let Some(node) = node_test.test(&new_context) {
						nodeset.add_node(node);
					}

					nodeset.extend(new_context.find_nodes(&AxisName::Descendant, node_test));
				}
			}

			AxisName::DescendantOrSelf => {
				nodeset.extend(self.find_nodes(&AxisName::SelfAxis, node_test));
				nodeset.extend(self.find_nodes(&AxisName::Descendant, node_test));
			}

			// excluding any descendants and excluding attribute nodes and namespace nodes
			AxisName::Following => {
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
				if let Some(node) = node_test.test(&self) {
					nodeset.add_node(node);
				}
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

	pub fn new_evaluation_from_opts(&'a self, node: Node, position: usize, size: usize) -> Self {
		Self {
			document: self.document,
			node,
			position,
			size
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



pub struct NodeSearch {
	states: Vec<NodeSearchState>
}

impl NodeSearch {
	pub fn new(context: AxisName, node: Node) -> Self {
		Self {
			states: vec![
				NodeSearchState::new(context, node)
			]
		}
	}

	/// Iterate until we (hopefully) find a Node.
	pub fn find_next(&mut self, eval: &Evaluation, node_test: &dyn NodeTest) -> Option<Node> {
		if let Some(mut state) = self.states.pop() {
			// Find Nodes in state.
			let (node, states) = state.find_next_node(eval, node_test);

			let is_state_vec_filled = states.as_ref().map(|v| !v.is_empty()).unwrap_or_default();

			let index = self.states.len();

			// Place any new states into the List.
			if let Some(mut states) = states {
				self.states.append(&mut states);
			}

			if let Some(node) = node {
				// Place state back into array. It could have more Nodes in it.
				self.states.insert(index, state);

				Some(node)
			} else {
				// Place state back into array. It could have more NodeSearchStates in it.
				if is_state_vec_filled {
					self.states.insert(index, state);
				}

				self.find_next(eval, node_test)
			}
		} else {
			None
		}
	}
}


pub struct NodeSearchState {
	context: AxisName,
	last_node_pos: Node,

	offset: usize,

	cached_nodes: Option<Vec<Node>>
}

impl NodeSearchState {
	pub fn new(context: AxisName, node: Node) -> Self {
		Self {
			context,
			last_node_pos: node,
			offset: 0,
			cached_nodes: None
		}
	}

	pub fn find_next_node(&mut self, eval: &Evaluation, node_test: &dyn NodeTest) -> (Option<Node>, Option<Vec<NodeSearchState>>) {
		match &self.context {
			AxisName::Ancestor => {
				if let Some(parent) = self.last_node_pos.parent() {
					let eval = eval.new_evaluation_from(parent);

					if let Some(node) = node_test.test(&eval) {
						let states = if node.is_root() {
							None
						} else {
							Some(vec![Self::new(AxisName::Ancestor, node.clone())])
						};


						return (Some(node), states);
					}
				}
			}

			AxisName::AncestorOrSelf => {
				return (None, Some(vec![Self::new(AxisName::Ancestor, self.last_node_pos.clone()), Self::new(AxisName::SelfAxis, self.last_node_pos.clone())]));
			}

			AxisName::Attribute => {
				if let Node::Element(node) = &self.last_node_pos {
					if let Some(mut attrs) = value::Attribute::from_node(node) {
						if self.offset < attrs.len() {
							let node = Node::Attribute(attrs.remove(self.offset));

							self.offset += 1;

							if let Some(node) = node_test.test(&eval.new_evaluation_from(node)) {
								return (Some(node), None);
							}
						}
					}
				}
			}

			AxisName::Child => {
				let mut children = self.last_node_pos.children();

				if self.offset < children.len() {
					let child = children.remove(self.offset);

					self.offset += 1;

					let new_context = eval.new_evaluation_from(child);

					if let Some(node) = node_test.test(&new_context) {
						return (Some(node), None);
					}
				}
			}

			AxisName::Descendant => {
				let mut children = self.last_node_pos.children();

				if self.offset < children.len() {
					let child = children.remove(self.offset);

					self.offset += 1;

					let new_context = eval.new_evaluation_from(child);

					if let Some(node) = node_test.test(&new_context) {
						return (Some(node), Some(vec![NodeSearchState::new(AxisName::Descendant, new_context.node)]));
					}
				}
			}

			AxisName::DescendantOrSelf => {
				return (None, Some(vec![Self::new(AxisName::Descendant, self.last_node_pos.clone()), Self::new(AxisName::SelfAxis, self.last_node_pos.clone())]));
			}

			// excluding any descendants and excluding attribute nodes and namespace nodes
			AxisName::Following => {
				// Get or Create a cache of Nodes.
				let nodes = if let Some(cache) = self.cached_nodes.as_mut() {
					cache
				} else {
					// Cache Nodes and reverse the array so we can .pop() from start to end.
					let mut nodes = value::following_nodes_from_parent(&self.last_node_pos);
					nodes.reverse();

					self.cached_nodes = Some(nodes);

					self.cached_nodes.as_mut().unwrap()
				};

				let mut states = Vec::new();

				// TODO: Might have to re-arrange these two.

				if let Some(node) = nodes.pop() {
					states.push(Self::new(AxisName::DescendantOrSelf, node));
				}

				// Get the parents children after 'self.node.parent()'
				if let Some(parent) = self.last_node_pos.parent() {
					states.push(Self::new(AxisName::Following, parent));
				}

				return (None, Some(states));
			}

			// if the context node is an attribute node or namespace node, the following-sibling axis is empty
			AxisName::FollowingSibling => {
				// Get or Create a cache of Nodes.
				let nodes = if let Some(cache) = self.cached_nodes.as_mut() {
					cache
				} else {
					// Cache Nodes and reverse the array so we can .pop() from start to end.
					let mut nodes = value::following_nodes_from_parent(&self.last_node_pos);
					nodes.reverse();

					self.cached_nodes = Some(nodes);

					self.cached_nodes.as_mut().unwrap()
				};

				if let Some(node) = nodes.pop() {
					return (Some(node), None);
				}
			}

			// contains the namespace nodes of the context node;
			// the axis will be empty unless the context node is an element
			AxisName::Namespace => {
				unimplemented!("AxisName::Namespace")
			}

			AxisName::Parent => {
				if let Some(node) = self.last_node_pos.parent() {
					return (Some(node), None);
				}
			}

			// excluding any ancestors and excluding attribute nodes and namespace nodes
			AxisName::Preceding => {
				// Get or Create a cache of Nodes.
				let nodes = if let Some(cache) = self.cached_nodes.as_mut() {
					cache
				} else {
					// Cache Nodes and reverse the array so we can .pop() from start to end.
					let mut nodes = value::preceding_nodes_from_parent(&self.last_node_pos);
					nodes.reverse();

					self.cached_nodes = Some(nodes);

					self.cached_nodes.as_mut().unwrap()
				};

				let mut states = Vec::new();

				// TODO: Might have to re-arrange these two.

				if let Some(node) = nodes.pop() {
					// TODO: Double check to ensure this AxisName is correct. I don't believe it is.
					states.push(Self::new(AxisName::DescendantOrSelf, node));
				}

				// Get the parents children before 'self.node.parent()'
				if let Some(parent) = self.last_node_pos.parent() {
					states.push(Self::new(AxisName::Preceding, parent));
				}

				return (None, Some(states));
			}

			// if the context node is an attribute node or namespace node, the preceding-sibling axis is empty
			AxisName::PrecedingSibling => {
				// Get or Create a cache of Nodes.
				let nodes = if let Some(cache) = self.cached_nodes.as_mut() {
					cache
				} else {
					// Cache Nodes and reverse the array so we can .pop() from start to end.
					let mut nodes = value::preceding_nodes_from_parent(&self.last_node_pos);
					nodes.reverse();

					self.cached_nodes = Some(nodes);

					self.cached_nodes.as_mut().unwrap()
				};

				if let Some(node) = nodes.pop() {
					return (Some(node), None);
				}
			}

			AxisName::SelfAxis => {
				if let Some(node) = node_test.test(eval) {
					return (Some(node), None);
				}
			}
		}

		(None, None)
	}
}