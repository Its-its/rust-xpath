// Used to tell us how to iterate through the Nodes.

use std::cell::RefCell;

use crate::{
	value,
	Document, Node, AxisName, NodeTest, expressions::Step, Result
};

#[derive(Debug)]
pub struct Evaluation<'a> {
	pub document: &'a Document,
	pub node: &'a Node,

	pub starting_eval_node: &'a Node,

	pub node_position: usize,

	pub is_last_node: bool
}



impl<'a> Evaluation<'a> {
	pub fn new(node: &'a Node, document: &'a Document) -> Evaluation<'a> {
		Evaluation {
			document,
			node,
			starting_eval_node: node,
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
			starting_eval_node: self.starting_eval_node,
			node_position: 1,
			is_last_node: false
		}
	}

	pub fn new_evaluation_from_with_pos(&'a self, node: &'a Node, position: usize) -> Self {
		Self {
			document: self.document,
			node,
			starting_eval_node: self.starting_eval_node,
			node_position: position,
			is_last_node: false
		}
	}
}


#[derive(Debug)]
pub struct FoundNode {
	pub node: Node,
	pub position: usize,
	pub step_index: usize,
}


#[derive(Debug)]
pub struct NodeSearch {
	state: NodeSearchState,

	search_steps: Vec<NodeSearchState>,

	cached: Option<(MoreNodes<Node>, Option<Vec<Self>>)>,
}

impl NodeSearch {
	pub fn new_from_state(state: NodeSearchState) -> Self {
		Self {
			state,
			search_steps: Vec::new(),
			cached: None,
		}
	}

	pub fn new(context: AxisName, node: Node, step_index: usize) -> Self {
		Self::new_from_state(
			NodeSearchState::new(step_index, context, node),
		)
	}


	pub fn find_and_cache_next_node(
		&mut self,
		super_eval: &Evaluation,
		global_steps: &[RefCell<Step>]
	) -> Result<(MoreNodes<Node>, Option<Vec<Self>>)> {
		let state_node = self.state.node.clone();

		let child_eval = super_eval.new_evaluation_from(&state_node);

		// Initiate cache. Used for "is last node" check
		if self.cached.is_none() {
			self.cached = Some(self.find_next_node(&child_eval, global_steps)?);
		}

		let next = self.find_next_node(&child_eval, global_steps)?;

		Ok(self.cached.replace(next).unwrap())
	}

	pub fn is_finished(&self) -> bool {
		self.cached.as_ref().map(|v| v.0.is_no()).unwrap_or(false)
	}

	/// Iterate until we (hopefully) find a Node.
	fn find_next_node(&mut self, eval: &Evaluation, global_steps: &[RefCell<Step>]) -> Result<(MoreNodes<Node>, Option<Vec<Self>>)> {
		let base_state = &mut self.state;

		let base_index = base_state.step_index;

		let (node, states) = base_state.find_next_node(
			eval,
			global_steps[base_index].borrow().node_test.as_ref()
		);

		let global_states = states.map(|v|
			v.into_iter()
			.map(Self::new_from_state)
			.collect()
		);

		match node {
			MoreNodes::Found(node) => {
				let next_step_index = base_index + 1;

				if global_steps.len() == next_step_index {
					base_state.found_count += 1;

					return Ok((
						// node,
						global_steps[base_index].borrow_mut().evaluate(
							eval,
							FoundNode { node, position: base_state.found_count, step_index: base_index, },
						)?,
						global_states
					));
				}
				// Run only once if __base_state hasn't ran before__ AND __base_state axis is descendant__.
				else if base_state.axis_name == AxisName::Descendant && global_steps[next_step_index].borrow().axis == AxisName::Child {
					if base_state.found_count == 1 {
						let curr_state = NodeSearchState::new(next_step_index, global_steps[next_step_index].borrow().axis, base_state.node.clone());

						// Insert into this group.
						self.search_steps.push(curr_state);
					}
				} else {
					let curr_state = NodeSearchState::new(next_step_index, global_steps[next_step_index].borrow().axis, node);

					// Insert into this group.
					self.search_steps.push(curr_state);
				}

				//
				if base_state.axis_name == AxisName::Descendant && global_steps[next_step_index].borrow().axis == AxisName::Child {
					let (noodles, new_states) = self.search_inner_until_not_found(
						eval,
						global_steps
					)?;

					return Ok((noodles, join_states(global_states, new_states)));
				} else {
					return Ok((MoreNodes::Possible, global_states));
				}
			}

			MoreNodes::Possible if self.search_steps.is_empty() => {
				return Ok((MoreNodes::Possible, global_states))
			},


			MoreNodes::No if self.search_steps.is_empty() => {
				return Ok((MoreNodes::No, global_states))
			},

			MoreNodes::No => (),

			v => unimplemented!("{v:?}"),
		}


		if let Some(curr_state) = self.search_steps.pop() {
			if let Some(resp) = self.search_inner_step(curr_state, eval, global_steps)? {
				return Ok((resp.0, join_states(global_states, resp.1)));
			} else {
				return Ok((MoreNodes::Possible, global_states));
			}
			// Else, retry
		}

		Ok((MoreNodes::Possible, global_states))
	}


	fn search_inner_until_not_found(
		&mut self,
		eval: &Evaluation,
		global_steps: &[RefCell<Step>]
	) -> Result<(MoreNodes<Node>, Option<Vec<Self>>)> {
		let mut states = None;

		while let Some(curr_state) = self.search_steps.pop() {
			let curr_step_len = self.search_steps.len() + 1;

			if let Some((node, states_new)) = self.search_inner_step(curr_state, eval, global_steps)? {
				states = join_states(states, states_new);

				// We found a node
				if node.has_passed_pred() {
					return Ok((node, states));
				}

				// If we added a new step to the list we'll go again.
				if curr_step_len < self.search_steps.len() {
					continue;
				}

				// If we're on the first step and we haven't passed pred. Break.
				if self.search_steps.len() <= 1 {
					break;
				}
			} else {
				break;
			}
		}

		Ok((MoreNodes::Possible, states))
	}


	#[allow(clippy::type_complexity)]
	fn search_inner_step(
		&mut self,
		mut curr_state: NodeSearchState,
		eval: &Evaluation,
		global_steps: &[RefCell<Step>]
	) -> Result<Option<(MoreNodes<Node>, Option<Vec<Self>>)>> {
		let step_index = curr_state.step_index;

		// Find Nodes in state.
		let (node, states) = curr_state.find_next_node(
			eval,
			global_steps[step_index].borrow().node_test.as_ref()
		);

		let states = states.map(|v|
			v.into_iter()
			.map(Self::new_from_state)
			.collect()
		);

		if node.is_found() {
			curr_state.found_count += 1;
		}

		let curr_node_pos = curr_state.found_count;

		if node.has_more() {
			// Place state back into array. It could have more Nodes in it.
			self.search_steps.push(curr_state);
		}

		match node {
			MoreNodes::Found(node) => {
				let passed = global_steps[step_index].borrow_mut().evaluate(
					eval,
					FoundNode { node, position: curr_node_pos, step_index, },
				)?;

				if let MoreNodes::PassedPredicate(node) = passed {
					if global_steps.len() == step_index + 1 {
						return Ok(Some((
							MoreNodes::PassedPredicate(node),
							states
						)));
					} else {
						let next_step_index = step_index + 1;

						// Insert into this group.
						self.search_steps.push(NodeSearchState::new(next_step_index, global_steps[next_step_index].borrow().axis, node));

						return Ok(Some((MoreNodes::Possible, states)));
					}
				}
				// Else, return none
			}

			MoreNodes::Possible => return Ok(Some((MoreNodes::Possible, states))),

			MoreNodes::No if states.is_some() => return Ok(Some((MoreNodes::Possible, states))),
			MoreNodes::No => (), // return none.

			v => unimplemented!("{v:?}"),
		}

		Ok(None)
	}
}

#[derive(Debug)]
pub struct NodeSearchState {
	step_index: usize,

	axis_name: AxisName,

	pub(crate) node: Node,

	offset: usize,

	pub found_count: usize,

	cached_nodes: Option<Vec<Node>> // TODO: Remove.
}

impl NodeSearchState {
	pub fn new(step_index: usize, axis_name: AxisName, node: Node) -> Self {
		Self {
			axis_name,
			node,
			step_index,
			offset: 0,
			found_count: 0,
			cached_nodes: None
		}
	}

	pub fn find_next_node(&mut self, eval: &Evaluation, node_test: &dyn NodeTest) -> (MoreNodes<Node>, Option<Vec<NodeSearchState>>) {
		match &self.axis_name {
			AxisName::Ancestor => {
				if let Some(parent) = self.node.parent() {
					let eval = eval.new_evaluation_from(&parent);

					if node_test.test(&eval).is_some() {
						let states = if parent.is_root() {
							None
						} else {
							Some(vec![Self::new(self.step_index, AxisName::Ancestor, parent.clone())])
						};

						return (MoreNodes::Found(parent), states);
					}
				}
			}

			AxisName::AncestorOrSelf => {
				return (
					MoreNodes::No,
					Some(vec![
						Self::new(self.step_index, AxisName::Ancestor, self.node.clone()),
						Self::new(self.step_index, AxisName::SelfAxis, self.node.clone())
					])
				);
			}

			AxisName::Attribute => {
				if let Node::Element(node) = &self.node {
					// Get or Create a cache of Nodes.
					let nodes = if let Some(cache) = self.cached_nodes.as_mut() {
						cache
					} else if let Some(attrs) = value::Attribute::from_node(node) {
						// Cache Nodes and reverse the array so we can .pop() from start to end.
						self.cached_nodes = Some(attrs.into_iter().rev().map(Node::Attribute).collect());

						self.cached_nodes.as_mut().unwrap()
					} else {
						return (MoreNodes::No, None);
					};

					while let Some(node_attr) = nodes.pop() {
						if node_test.test(&eval.new_evaluation_from(&node_attr)).is_some() {
							return (MoreNodes::Found(node_attr), None);
						}
					}
				}
			}

			AxisName::Child => {
				if let Some(child) = self.node.get_child(self.offset) {
					self.offset += 1;

					let new_context = eval.new_evaluation_from_with_pos(&child, self.offset);

					if node_test.test(&new_context).is_some() {
						return (MoreNodes::Found(child), None);
					} else {
						return (MoreNodes::Possible, None);
					}
				}
			}

			AxisName::Descendant => {
				if let Some(child) = self.node.get_child(self.offset) {
					self.offset += 1;

					let new_context = eval.new_evaluation_from_with_pos(&child, self.offset);

					if node_test.test(&new_context).is_some() {
						return (
							// Return Current Child
							MoreNodes::Found(child.clone()),
							// Append Child to search through
							Some(vec![NodeSearchState::new(self.step_index, AxisName::Descendant, child)])
						);
					}
				}
			}

			AxisName::DescendantOrSelf => {
				return (
					MoreNodes::No,
					Some(vec![
						Self::new(self.step_index, AxisName::Descendant, self.node.clone()),
						Self::new(self.step_index, AxisName::SelfAxis, self.node.clone()),
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
					states.push(Self::new(self.step_index, AxisName::DescendantOrSelf, node));
				}

				// Get the parents children after 'self.node.parent()'
				if let Some(parent) = self.node.parent() {
					states.push(Self::new(self.step_index, AxisName::Following, parent));
				}

				return (MoreNodes::No, Some(states));
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
					return (MoreNodes::Found(node), None);
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
					return (MoreNodes::Found(node), None);
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
					states.push(Self::new(self.step_index, AxisName::DescendantOrSelf, node));
				}

				// Get the parents children before 'self.node.parent()'
				if let Some(parent) = self.node.parent() {
					states.push(Self::new(self.step_index, AxisName::Preceding, parent));
				}

				return (MoreNodes::No, Some(states));
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
					return (MoreNodes::Found(node), None);
				}
			}

			AxisName::SelfAxis => if self.offset == 0 && node_test.test(eval).is_some() {
				self.offset = 1;
				return (MoreNodes::Found(eval.node.clone()), None);
			}
		}

		(MoreNodes::No, None)
	}
}




#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum MoreNodes<V> {
	/// Found something before checks
	Found(V),

	/// Found something which passes checks
	PassedPredicate(V),
	/// Found something which didn't pass checks
	FailedPredicate,

	Possible,

	/// Nothing Found
	No
}

impl<V> MoreNodes<V> {
	/// If we found a node or if there are potentially more nodes
	pub fn has_more(&self) -> bool {
		matches!(self, Self::Found(_) | Self::Possible)
	}

	/// We found a Node
	pub fn is_found(&self) -> bool {
		matches!(self, Self::Found(_))
	}

	/// It's possible we have more Nodes.
	pub fn is_possible(&self) -> bool {
		matches!(self, Self::Possible)
	}

	/// We have no more Nodes.
	pub fn is_no(&self) -> bool {
		matches!(self, Self::No)
	}

	/// Node Passed Predicate
	pub fn has_passed_pred(&self) -> bool {
		matches!(self, Self::PassedPredicate(_))
	}


	pub fn take(&mut self) -> Self {
		std::mem::replace(self, Self::No)
	}

	pub fn as_ref(&self) -> MoreNodes<&V> {
		match self {
			MoreNodes::Found(v) => MoreNodes::Found(v),
			MoreNodes::PassedPredicate(v) => MoreNodes::PassedPredicate(v),
			MoreNodes::FailedPredicate => MoreNodes::FailedPredicate,
			MoreNodes::Possible => MoreNodes::Possible,
			MoreNodes::No => MoreNodes::No,
		}
	}

	pub fn map<U, F>(self, f: F) -> MoreNodes<U>
	where
		F: FnOnce(V) -> U,
	{
		match self {
			MoreNodes::Found(v) => MoreNodes::Found(f(v)),
			MoreNodes::PassedPredicate(v) => MoreNodes::PassedPredicate(f(v)),
			MoreNodes::FailedPredicate => MoreNodes::FailedPredicate,
			MoreNodes::Possible => MoreNodes::Possible,
			MoreNodes::No => MoreNodes::No,
		}
	}


	/// Turn into Option.
	pub fn into_option(self) -> Option<V> {
		match self {
			Self::Found(v)|
			Self::PassedPredicate(v) => Some(v),

			_ => None,
		}
	}

	/// Reference to Optional Value.
	pub fn as_option(&self) -> Option<&V> {
		match self {
			Self::Found(v) |
			Self::PassedPredicate(v) => Some(v),

			_ => None
		}
	}
}


fn join_states<V>(left: Option<Vec<V>>, right: Option<Vec<V>>) -> Option<Vec<V>> {
	match (left, right) {
		(None, None) => None,

		(None, Some(v)) |
		(Some(v), None) => Some(v),

		(Some(mut l), Some(mut r)) => {
			l.append(&mut r);
			Some(l)
		}
	}
}