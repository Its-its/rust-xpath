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

	pub fn find_nodes(&self, node_search: &mut NodeSearch, node_test: &dyn NodeTest) -> Option<Node> {
		node_search.find_next(self, node_test)
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


#[derive(Debug)]
pub struct NodeSearch {
	states: Vec<NodeSearchState>
}

impl NodeSearch {
	pub fn new() -> Self {
		Self {
			states: Vec::new()
		}
	}

	pub fn new_with_state(context: AxisName, node: Node) -> Self {
		Self {
			states: vec![
				NodeSearchState::new(context, node)
			]
		}
	}

	pub fn add_state(&mut self, state: NodeSearchState) {
		self.states.push(state);
	}

	pub fn get_current_node_pos(&self) -> Option<Node> {
		self.states.last().map(|s| s.last_node_pos.clone())
	}

	/// Iterate until we (hopefully) find a Node.
	pub fn find_next(&mut self, eval: &Evaluation, node_test: &dyn NodeTest) -> Option<Node> {
		// TODO: There has to be a better way to do this.
		while let Some(mut state) = self.states.pop() {
			// Find Nodes in state.
			let (node, states) = state.find_next_node(eval, node_test);

			let prev_state_size = self.states.len();

			// Place any new states into the List.
			if let Some(mut states) = states {
				self.states.append(&mut states);
			}

			if node.is_some() {
				// Place state back into array. It could have more Nodes in it.
				self.states.insert(prev_state_size, state);
			} else if let Some(state) = self.states.last_mut() {
				state.offset = state.offset.saturating_sub(1);
			}

			if let Some(node) = node {
				return Some(node);
			}
		}

		None
	}
}

impl Default for NodeSearch {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug)]
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
						while self.offset < attrs.len() {
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
				// Get or Create a cache of Nodes.
				let children = if let Some(cache) = self.cached_nodes.as_mut() {
					cache
				} else {
					// Cache Nodes and reverse the array so we can .pop() from start to end.
					let nodes = self.last_node_pos.children();

					self.cached_nodes = Some(nodes);

					self.cached_nodes.as_mut().unwrap()
				};

				while self.offset < children.len() {
					let child = children.remove(self.offset);

					self.offset += 1;

					let new_context = eval.new_evaluation_from(child);

					if let Some(node) = node_test.test(&new_context) {
						return (Some(node), None);
					}
				}
			}

			AxisName::Descendant => {
				// Get or Create a cache of Nodes.
				let children = if let Some(cache) = self.cached_nodes.as_mut() {
					cache
				} else {
					// Cache Nodes and reverse the array so we can .pop() from start to end.
					let nodes = self.last_node_pos.children();

					self.cached_nodes = Some(nodes);

					self.cached_nodes.as_mut().unwrap()
				};


				while self.offset < children.len() {
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

			AxisName::Parent => if self.offset == 0 {
				if let Some(node) = self.last_node_pos.parent() {
					self.offset = 1;
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

			AxisName::SelfAxis => if self.offset == 0 {
				if let Some(node) = node_test.test(eval) {
					self.offset = 1;
					return (Some(node), None);
				}
			}
		}

		(None, None)
	}
}