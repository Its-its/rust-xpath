// Used to tell us how to iterate through the Nodes.

use crate::{
	value,
	Document, Node, AxisName, NodeTest
};

#[derive(Debug)]
pub struct Evaluation<'a> {
	pub document: &'a Document,
	pub node: &'a Node,

	pub node_position: usize,

	pub is_last_node: bool
}



impl<'a> Evaluation<'a> {
	pub fn new(node: &'a Node, document: &'a Document) -> Evaluation<'a> {
		Evaluation {
			document,
			node,
			node_position: 1,
			is_last_node: false
		}
	}

	pub fn root(&'a self) -> &'a Node {
		&self.document.root
	}

	pub fn new_evaluation_from(&'a self, node: &'a Node) -> Self {
		Self {
			document: self.document,
			node,
			node_position: 1,
			is_last_node: false
		}
	}

	pub fn new_evaluation_from_with_pos(&'a self, node: &'a Node, position: usize) -> Self {
		Self {
			document: self.document,
			node,
			node_position: position,
			is_last_node: false
		}
	}
}


#[derive(Debug)]
pub struct FoundNode {
	pub node: Node,
	pub position: usize
}


#[derive(Debug)]
pub struct NodeSearch {
	states: Vec<NodeSearchState>,

	/// Stores Node and position of Node.
	cached_node_info: Option<FoundNode>
}

impl NodeSearch {
	pub fn new() -> Self {
		Self {
			states: Vec::new(),
			cached_node_info: None
		}
	}

	pub fn is_finished(&self) -> bool {
		self.cached_node_info.is_none() || self.states.is_empty()
	}

	pub fn new_with_state(context: AxisName, node: Node, eval: &Evaluation, node_test: &dyn NodeTest) -> Self {
		let mut this = Self {
			states: vec![
				NodeSearchState::new(context, node.clone())
			],
			cached_node_info: None
		};

		let eval = eval.new_evaluation_from(&node);

		// Place Node into next_node.
		let node = this.find_next_node(&eval, node_test);

		this.cached_node_info = node;

		this
	}

	pub fn get_current_state(&self) -> Option<&NodeSearchState> {
		self.states.last()
	}

	/// Iterate until we (hopefully) find a Node.
	fn find_next_node(&mut self, eval: &Evaluation, node_test: &dyn NodeTest) -> Option<FoundNode> {
		while let Some(mut state) = self.states.pop() {
			// Find Nodes in state.
			let (node, states) = state.find_next_node(eval, node_test);

			// Store current state size.
			let prev_state_size = self.states.len();

			// Place any new states into the List.
			if let Some(mut states) = states {
				self.states.append(&mut states);
			}

			state.found_count += 1;

			let node_pos = state.found_count;

			if node.is_some() {
				// Place state back into array. It could have more Nodes in it.
				self.states.insert(prev_state_size, state);
			}

			if let Some(node) = node {
				// Cache found nodes?
				return Some(FoundNode { node, position: node_pos });
			}
		}

		None
	}

	pub fn find_and_cache_next_node(&mut self, super_eval: &Evaluation, node_test: &dyn NodeTest) -> Option<FoundNode> {
		if self.is_finished() {
			return self.cached_node_info.take();
		}

		let state_node = self.get_current_state()?.node.clone();

		let child_eval = super_eval.new_evaluation_from(&state_node);

		// Get next node, replace next node with current cached node.
		let next_node = self.find_next_node(&child_eval, node_test);

		std::mem::replace(&mut self.cached_node_info, next_node)
	}
}

impl Default for NodeSearch {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug)]
pub struct NodeSearchState {
	axis_name: AxisName,

	pub(crate) node: Node,

	offset: usize,

	pub found_count: usize,

	cached_nodes: Option<Vec<Node>> // TODO: Remove.
}

impl NodeSearchState {
	pub fn new(axis_name: AxisName, node: Node) -> Self {
		Self {
			axis_name,
			node,
			offset: 0,
			found_count: 0,
			cached_nodes: None
		}
	}

	pub fn find_next_node(&mut self, eval: &Evaluation, node_test: &dyn NodeTest) -> (Option<Node>, Option<Vec<NodeSearchState>>) {
		match &self.axis_name {
			AxisName::Ancestor => {
				if let Some(parent) = self.node.parent() {
					let eval = eval.new_evaluation_from(&parent);

					if let Some(node) = node_test.test(&eval) {
						let states = if node.is_root() {
							None
						} else {
							Some(vec![Self::new(AxisName::Ancestor, node.clone())])
						};

						return (Some(parent), states);
					}
				}
			}

			AxisName::AncestorOrSelf => {
				return (
					None,
					Some(vec![
						Self::new(AxisName::Ancestor, self.node.clone()),
						Self::new(AxisName::SelfAxis, self.node.clone())
					])
				);
			}

			AxisName::Attribute => {
				if let Node::Element(node) = &self.node {
					if let Some(mut attrs) = value::Attribute::from_node(node) {
						while self.offset < attrs.len() {
							let node_attr = Node::Attribute(attrs.remove(self.offset));

							self.offset += 1;

							if node_test.test(&eval.new_evaluation_from(&node_attr)).is_some() {
								return (Some(node_attr), None);
							}
						}
					}
				}
			}

			AxisName::Child => {
				if let Some(child) = self.node.get_child(self.offset) {
					self.offset += 1;

					let new_context = eval.new_evaluation_from_with_pos(&child, self.offset);

					if node_test.test(&new_context).is_some() {
						return (Some(child), None);
					}
				}
			}

			AxisName::Descendant => {
				if let Some(child) = self.node.get_child(self.offset) {
					self.offset += 1;

					let new_context = eval.new_evaluation_from_with_pos(&child, self.offset);

					if node_test.test(&new_context).is_some() {
						return (Some(child.clone()), Some(vec![NodeSearchState::new(AxisName::Descendant, child)]));
					}
				}
			}

			AxisName::DescendantOrSelf => {
				return (
					None,
					Some(vec![
						Self::new(AxisName::Descendant, self.node.clone()),
						Self::new(AxisName::SelfAxis, self.node.clone())
					])
				);
			}

			// excluding any descendants and excluding attribute nodes and namespace nodes
			AxisName::Following => {
				// Get or Create a cache of Nodes.
				let nodes = if let Some(cache) = self.cached_nodes.as_mut() {
					cache
				} else {
					// Cache Nodes and reverse the array so we can .pop() from start to end.
					let mut nodes = value::following_nodes_from_parent(&self.node);
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
				if let Some(parent) = self.node.parent() {
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
					let mut nodes = value::following_nodes_from_parent(&self.node);
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
				if let Some(node) = self.node.parent() {
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
					let mut nodes = value::preceding_nodes_from_parent(&self.node);
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
				if let Some(parent) = self.node.parent() {
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
					let mut nodes = value::preceding_nodes_from_parent(&self.node);
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
					return (Some(node.clone()), None);
				}
			}
		}

		(None, None)
	}
}