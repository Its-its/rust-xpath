use std::cell::Cell;
use std::fmt;
use std::rc::Rc;

use html5ever::serialize;
use markup5ever::{Attribute as DomAttribute, QualName};
use markup5ever_rcdom::{
    Handle as NodeHandle, NodeData, SerializableHandle, WeakHandle as WeakNodeHandle,
};

use crate::factory::ProduceIter;
use crate::result::{Result, ValueError};
use crate::{Document, Error};

#[derive(Debug, Clone)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(String),
    Node(Node),
}

impl Value {
    pub fn is_something(&self) -> bool {
        match self {
            Value::Boolean(v) => *v,
            Value::Number(v) => !v.is_nan(),
            Value::String(v) => !v.is_empty(),
            Value::Node(_) => true,
        }
    }

    pub fn as_node(&self) -> Result<&Node> {
        match self {
            Self::Node(s) => Ok(s),
            _ => Err(ValueError::Nodeset.into()),
        }
    }

    pub fn is_node(&self) -> bool {
        matches!(self, Self::Node(_))
    }

    pub fn into_node(self) -> Result<Node> {
        match self {
            Self::Node(s) => Ok(s),
            _ => Err(ValueError::Nodeset.into()),
        }
    }

    pub fn boolean(&self) -> Result<bool> {
        match self {
            &Self::Boolean(v) => Ok(v),
            Self::Number(v) if *v == 0.0 => Ok(false),
            Self::Number(v) if *v == 1.0 => Ok(true),
            _ => Err(ValueError::Boolean.into()),
        }
    }

    pub fn number(&self) -> Result<f64> {
        match *self {
            Self::Boolean(v) => Ok(if v { 1.0 } else { 0.0 }),
            Self::Number(v) => Ok(v),
            _ => Err(ValueError::Number.into()),
        }
    }

    pub fn as_string(&self) -> Result<&String> {
        match self {
            Value::String(v) => Ok(v),
            _ => Err(ValueError::String.into()),
        }
    }

    pub fn string(self) -> Result<String> {
        match self {
            Value::String(v) => Ok(v),
            _ => Err(ValueError::String.into()),
        }
    }

    /// Change non-string `Value` to a `String`
    pub fn convert_to_string(self) -> Result<String> {
        Ok(match self {
            Value::Boolean(_) => String::new(),
            Value::Number(v) => v.to_string(),
            Value::String(v) => v,
            Value::Node(v) => v.get_string_value()?,
        })
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Self::Number(v1), Self::Number(v2)) => v1 == v2,
            (Self::Boolean(v1), Self::Boolean(v2)) => v1 == v2,
            (Self::String(v1), Self::String(v2)) => v1 == v2,
            (Self::Node(set1), Self::Node(set2)) => set1 == set2,

            // Node == String
            (Self::Node(node), Self::String(value)) | (Self::String(value), Self::Node(node)) => {
                // TODO: No.
                if &format!("{:?}", node) == value {
                    true
                } else {
                    match node {
                        Node::Attribute(attr) => attr.value() == value,

                        Node::Text(handle) => {
                            let upgrade = handle.upgrade().unwrap();
                            if let NodeData::Text { contents } = &upgrade.data {
                                contents
                                    .try_borrow()
                                    .map(|v| v.as_ref() == value)
                                    .unwrap_or_default()
                            } else {
                                false
                            }
                        }

                        _ => false,
                    }
                }
            }

            _ => false,
        }
    }
}

impl From<bool> for Value {
    fn from(val: bool) -> Self {
        Value::Boolean(val)
    }
}

impl From<f64> for Value {
    fn from(val: f64) -> Self {
        Value::Number(val)
    }
}

impl From<String> for Value {
    fn from(val: String) -> Self {
        Value::String(val)
    }
}

impl From<Node> for Value {
    fn from(val: Node) -> Self {
        Value::Node(val)
    }
}

#[derive(Clone)]
pub struct Attribute {
    pub parent: WeakNodeHandle,
    pub attr: DomAttribute,
}

impl Attribute {
    pub fn new(parent: WeakNodeHandle, attr: DomAttribute) -> Self {
        Self { parent, attr }
    }

    pub fn from_node(node: &WeakNodeHandle) -> Option<Vec<Attribute>> {
        if let NodeData::Element { attrs, .. } = &node.upgrade().unwrap().data {
            Some(
                attrs
                    .borrow()
                    .iter()
                    .map(|a| Attribute::new(node.clone(), a.clone()))
                    .collect(),
            )
        } else {
            None
        }
    }

    pub fn name(&self) -> &QualName {
        &self.attr.name
    }

    pub fn name_string(&self) -> String {
        let mut comp = String::new();

        if let Some(prefix) = &self.attr.name.prefix {
            comp.push_str(prefix);
            comp.push(':');
        }

        comp.push_str(&self.attr.name.local);

        comp
    }

    pub fn value(&self) -> &str {
        &self.attr.value
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
        matches!(self, Node::Root(_))
    }

    pub fn is_namespace(&self) -> bool {
        matches!(self, Node::Namespace(_))
    }

    pub fn is_element(&self) -> bool {
        matches!(self, Node::Element(_))
    }

    pub fn is_attribute(&self) -> bool {
        matches!(self, Node::Attribute(_))
    }

    pub fn is_text(&self) -> bool {
        matches!(self, Node::Text(_))
    }

    pub fn is_comment(&self) -> bool {
        matches!(self, Node::Comment(_))
    }

    pub fn is_processing_instruction(&self) -> bool {
        matches!(self, Node::ProcessingInstruction(_))
    }

    pub fn get_string_value(&self) -> Result<String> {
        self.value().and_then(|v| v.string())
    }

    pub fn value(&self) -> Result<Value> {
        match self {
            Node::Attribute(attr) => Ok(Value::String(attr.value().to_string())),

            Node::Text(node) => {
                if let NodeData::Text { contents } = &node.upgrade().unwrap().data {
                    Ok(Value::String(contents.borrow().to_string()))
                } else {
                    Err(Error::NodeDidNotContainText)
                }
            }

            _ => Err(Error::CannotConvertNodeToValue),
        }
    }

    pub fn as_simple_html(&self) -> Option<String> {
        match self {
            Node::Root(_) => None,

            Node::Attribute(attr) => Some(format!("@{}={}", attr.name_string(), attr.value())),

            _ => {
                let mut st = Vec::new();

                let write = std::io::Cursor::new(&mut st);

                serialize::<_, SerializableHandle>(
                    write,
                    &self.inner_weak()?.upgrade()?.into(),
                    html5ever::serialize::SerializeOpts {
                        traversal_scope: markup5ever::serialize::TraversalScope::IncludeNode,
                        ..Default::default()
                    },
                )
                .ok()?;

                Some(String::from_utf8(st).ok()?)
            }
        }
    }

    pub fn attribute(&self) -> Option<&Attribute> {
        match self {
            Node::Attribute(attr) => Some(attr),
            _ => None,
        }
    }

    pub fn parent(&self) -> Option<Node> {
        // TODO: Fix. Example. The Root element would get classified as an Node::Element instead of Node::Root.
        match self {
            Node::Attribute(attr) => attr
                .parent
                .upgrade()
                .and_then(|node| get_opt_node_from_cell(&node.parent).map(Node::Element)),
            Node::DocType(_) | Node::Namespace(_) | Node::Root(_) => None,
            Node::Element(weak) => weak
                .upgrade()
                .and_then(|node| get_opt_node_from_cell(&node.parent).map(Node::Element)),
            Node::Text(weak) => weak
                .upgrade()
                .and_then(|node| get_opt_node_from_cell(&node.parent).map(Node::Text)),
            Node::Comment(weak) => weak
                .upgrade()
                .and_then(|node| get_opt_node_from_cell(&node.parent).map(Node::Comment)),
            Node::ProcessingInstruction(weak) => weak.upgrade().and_then(|node| {
                get_opt_node_from_cell(&node.parent).map(Node::ProcessingInstruction)
            }),
        }
    }

    pub fn children(&self) -> Vec<Node> {
        match self {
            Node::Root(handle) => {
                let node = handle.as_ref();

                node.children.borrow().iter().map(|c| c.into()).collect()
            }

            Node::Text(handle)
            | Node::Comment(handle)
            | Node::DocType(handle)
            | Node::Element(handle) => {
                let node = handle.upgrade().unwrap();

                let borrow = node.children.borrow();

                borrow.iter().map(|c| c.into()).collect()
            }

            _ => unimplemented!("Node::children(\"{}\")", self.enum_name()),
        }
    }

    pub fn get_child(&self, index: usize) -> Option<Node> {
        match self {
            Node::Root(handle) => {
                let node = handle.as_ref();

                let children = node.children.borrow();

                Some(children.get(index)?.into())
            }

            Node::Text(handle)
            | Node::Comment(handle)
            | Node::DocType(handle)
            | Node::Element(handle) => {
                let node = handle.upgrade()?;

                let children = node.children.borrow();

                Some(children.get(index)?.into())
            }

            _ => unimplemented!("Node::children(\"{}\")", self.enum_name()),
        }
    }

    pub fn name(&self) -> Option<QualName> {
        match self {
            Node::Element(node) => {
                if let NodeData::Element { name, .. } = &node.upgrade()?.data {
                    Some(name.clone())
                } else {
                    None
                }
            }

            Node::Attribute(attr) => {
                if let NodeData::Element { name, .. } = &attr.parent.upgrade()?.data {
                    Some(name.clone())
                } else {
                    None
                }
            }

            _ => None,
        }
    }

    pub fn target(&self) -> Option<String> {
        match self {
            Node::ProcessingInstruction(node) => {
                if let NodeData::ProcessingInstruction { target, .. } = &node.upgrade()?.data {
                    Some(target.to_string())
                } else {
                    None
                }
            }

            _ => None,
        }
    }

    pub fn prefix(&self) -> String {
        unimplemented!("Node::prefix()");
    }

    pub fn inner_weak(&self) -> Option<&WeakNodeHandle> {
        match self {
            Node::Root(..) => None,
            Node::DocType(weak)
            | Node::Namespace(weak)
            | Node::Element(weak)
            | Node::Text(weak)
            | Node::Comment(weak)
            | Node::ProcessingInstruction(weak) => Some(weak),
            Node::Attribute(weak) => Some(&weak.parent),
        }
    }

    pub fn evaluate_from<'a, S: Into<String>>(
        &'a self,
        search: S,
        doc: &'a Document,
    ) -> Result<ProduceIter<'a>> {
        doc.evaluate_from(search, self)
    }
}

impl From<&NodeHandle> for Node {
    fn from(handle: &NodeHandle) -> Self {
        match &handle.data {
            NodeData::Comment { .. } => Node::Comment(Rc::downgrade(handle)),

            NodeData::Document => {
                panic!("Cannot convert borrowed Document to Node.")
            }

            NodeData::Element { .. } => Node::Element(Rc::downgrade(handle)),

            NodeData::ProcessingInstruction { .. } => {
                Node::ProcessingInstruction(Rc::downgrade(handle))
            }

            NodeData::Text { .. } => Node::Text(Rc::downgrade(handle)),

            NodeData::Doctype { .. } => Node::DocType(Rc::downgrade(handle)),
        }
    }
}

impl From<NodeHandle> for Node {
    fn from(handle: NodeHandle) -> Self {
        match handle.data {
            NodeData::Comment { .. } => Node::Comment(Rc::downgrade(&handle)),

            NodeData::Document => Node::Root(handle),

            NodeData::Element { .. } => Node::Element(Rc::downgrade(&handle)),

            NodeData::Doctype { .. } => Node::DocType(Rc::downgrade(&handle)),

            NodeData::ProcessingInstruction { .. } => {
                Node::ProcessingInstruction(Rc::downgrade(&handle))
            }

            NodeData::Text { .. } => Node::Text(Rc::downgrade(&handle)),
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        if self.is_root() || other.is_root() {
            return self.is_root() == other.is_root();
        }

        match (self.inner_weak(), other.inner_weak()) {
            (Some(left), Some(right)) => left.ptr_eq(right),
            _ => false,
        }
    }
}

pub fn compare_weak_nodes(left: &WeakNodeHandle, right: &WeakNodeHandle) -> bool {
    let left_upgrade = left.upgrade().unwrap();
    let right_upgrade = right.upgrade().unwrap();

    compare_nodes(&left_upgrade, &right_upgrade)
}

pub fn following_nodes_from_parent(node: &Node) -> Vec<Node> {
    find_nodes_from_parent(node, |child_pos, node_pos| child_pos > node_pos)
}

pub fn preceding_nodes_from_parent(node: &Node) -> Vec<Node> {
    find_nodes_from_parent(node, |child_pos, node_pos| child_pos < node_pos)
}

fn find_nodes_from_parent<F: Fn(usize, usize) -> bool>(node: &Node, f_capture: F) -> Vec<Node> {
    let node = match node.inner_weak().and_then(|v| v.upgrade()) {
        Some(v) => v,
        None => return Vec::new(),
    };

    // Taken from markup5ever_rcdom
    if let Some(weak) = node.parent.take() {
        let parent = weak.upgrade().expect("dangling weak pointer");
        node.parent.set(Some(weak));

        let children = parent.children.borrow();

        let i = match children
            .iter()
            .enumerate()
            .find(|&(_, child)| Rc::ptr_eq(child, &node))
        {
            Some((i, _)) => i,
            None => return Vec::new(),
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
                contents: b_contents,
            },
            NodeData::Text { contents },
        ) => b_contents == contents,

        (
            NodeData::Comment {
                contents: b_contents,
            },
            NodeData::Comment { contents },
        ) => b_contents == contents,

        (
            NodeData::Doctype {
                name: b_name,
                public_id: b_public_id,
                system_id: b_system_id,
            },
            NodeData::Doctype {
                name,
                public_id,
                system_id,
            },
        ) => b_name == name || b_public_id == public_id || b_system_id == system_id,

        (
            NodeData::Element {
                name: b_name,
                attrs: b_attr,
                template_contents: b_template_contents,
                mathml_annotation_xml_integration_point: b_mathml,
            },
            NodeData::Element {
                name,
                attrs,
                template_contents,
                mathml_annotation_xml_integration_point,
            },
        ) => {
            b_name == name
                || b_attr == attrs
                || Some((b_template_contents, template_contents))
                    .filter(|c| c.0.borrow().is_some() || c.1.borrow().is_some())
                    .map(|i| {
                        compare_nodes(
                            i.0.borrow().as_ref().unwrap(),
                            i.1.borrow().as_ref().unwrap(),
                        )
                    })
                    .unwrap_or_default()
                || b_mathml == mathml_annotation_xml_integration_point
        }

        (
            NodeData::ProcessingInstruction {
                target: b_target,
                contents: b_contents,
            },
            NodeData::ProcessingInstruction { target, contents },
        ) => b_target == target || b_contents == contents,

        _ => false,
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
    !l_children
        .iter()
        .zip(r_children.iter())
        .any(|c| !compare_nodes(c.0, c.1))
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
            Node::Root(weak) => f.debug_tuple("Root").field(&weak).finish(),

            Node::Attribute(weak) => f
                .debug_tuple("Attribute")
                .field(&weak.parent.upgrade().unwrap().data)
                .finish(),

            Node::DocType(weak)
            | Node::Element(weak)
            | Node::Namespace(weak)
            | Node::Text(weak)
            | Node::Comment(weak)
            | Node::ProcessingInstruction(weak) => f
                .debug_tuple("Node")
                .field(&weak.upgrade().unwrap().data)
                .finish(),
        }
    }
}

// TODO: Ensure no duplicate nodes
#[derive(Clone, Default)]
pub struct Nodeset {
    pub nodes: Vec<Node>,
}

impl Nodeset {
    pub fn new() -> Self {
        Default::default()
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

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
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
        Self { nodes }
    }
}

impl fmt::Debug for Nodeset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();

        self.nodes.iter().for_each(|node| {
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
