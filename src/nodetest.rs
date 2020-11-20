// https://www.w3.org/TR/1999/REC-xpath-19991116/#node-tests


use std::fmt;

use markup5ever::{QualName, Namespace as Ns, LocalName};

use crate::{Evaluation, Nodeset, Node as DomNode};

pub trait NodeTest: fmt::Debug {
    fn test(&self, context: &Evaluation, result: &mut Nodeset);
}


// TODO: Convert to markup5ever::QualName
#[derive(Debug, Clone, PartialEq)]
pub struct NameTest { // '*' | NCName ':' '*' | QName
	pub prefix: Option<String>,
	pub local_part: String
}

impl NameTest {
	fn is_match(&self, _context: &Evaluation, qname: &QualName) -> bool {
		let has_wildcard = self.local_part == "*";

		// TODO: Compare prefix

		if has_wildcard {
			true
		} else {
			self.local_part.as_str() == &qname.local
		}
	}
}


// 5.3 Attribute Nodes
// Each element node has an associated set of attribute nodes;
// the element is the parent of each of these attribute nodes;
// however, an attribute node is not a child of its parent element.
//     NOTE: This is different from the DOM, which does not treat the element bearing an attribute as the parent of the attribute (see [DOM]).

// Elements never share attribute nodes:
//  if one element node is not the same node as another element node,
//  then none of the attribute nodes of the one element node will be the same node as the attribute nodes of another element node.
//     NOTE: The = operator tests whether two nodes have the same value, not whether they are the same node.
// 	         Thus attributes of two different elements may compare as equal using =, even though they are not the same node.

// A defaulted attribute is treated the same as a specified attribute.
// If an attribute was declared for the element type in the DTD,
// but the default was declared as #IMPLIED,
// and the attribute was not specified on the element,
// then the element's attribute set does not contain a node for the attribute.

// Some attributes, such as xml:lang and xml:space,
// have the semantics that they apply to all elements that are descendants of the element bearing the attribute,
// unless overridden with an instance of the same attribute on another descendant element.
// However, this does not affect where attribute nodes appear in the tree:
//  an element has attribute nodes only for attributes that were explicitly specified in the start-tag or empty-element tag of that element or that were explicitly declared in the DTD with a default value.

// An attribute node has an expanded-name and a string-value.
// The expanded-name is computed by expanding the QName specified in the tag in the XML document in accordance with the XML Namespaces Recommendation [XML Names].
// The namespace URI of the attribute's name will be null if the QName of the attribute does not have a prefix.
//     NOTE: In the notation of Appendix A.3 of [XML Names],
//           the local part of the expanded-name corresponds to the name attribute of the ExpAName element;
//           the namespace URI of the expanded-name corresponds to the ns attribute of the ExpAName element,
//           and is null if the ns attribute of the ExpAName element is omitted.

// An attribute node has a string-value.
// The string-value is the normalized value as specified by the XML Recommendation [XML].
// An attribute whose normalized value is a zero-length string is not treated specially:
//  it results in an attribute node whose string-value is a zero-length string.
//     NOTE: It is possible for default attributes to be declared in an external DTD or an external parameter entity.
//           The XML Recommendation does not require an XML processor to read an external DTD or an external parameter unless it is validating.
//           A stylesheet or other facility that assumes that the XPath tree contains default attribute values declared in an external DTD or parameter entity may not work with some non-validating XML processors.

// There are no attribute nodes corresponding to attributes that declare namespaces (see [XML Names]).
#[derive(Debug)]
pub struct Attribute {
    name_test: NameTest,
}

impl Attribute {
    pub fn new(name: NameTest) -> Attribute {
        Attribute { name_test: name }
    }
}

impl NodeTest for Attribute {
    fn test(&self, context: &Evaluation, result: &mut Nodeset) {
        if context.node.is_attribute() {
            let attr = context.node.attribute();

            if self.name_test.is_match(context, &attr.attr.name) {
                result.add_node(context.node.clone());
            }
        }
    }
}



// 5.4 Namespace Nodes

// Each element has an associated set of namespace nodes,
// one for each distinct namespace prefix that is in scope for the element
// (including the xml prefix, which is implicitly declared by the XML Namespaces Recommendation [XML Names])
// and one for the default namespace if one is in scope for the element.
// The element is the parent of each of these namespace nodes;
// however, a namespace node is not a child of its parent element.
// Elements never share namespace nodes:
//     if one element node is not the same node as another element node,
//     then none of the namespace nodes of the one element node will be the same node as the namespace nodes of another element node.
// This means that an element will have a namespace node:
//     for every attribute on the element whose name starts with xmlns:;
//     for every attribute on an ancestor element whose name starts xmlns:
//         unless the element itself or a nearer ancestor redeclares the prefix;
//     for an xmlns attribute, if the element or some ancestor has an xmlns attribute,
//     and the value of the xmlns attribute for the nearest such element is non-empty
//         NOTE: An attribute xmlns="" "undeclares" the default namespace (see [XML Names]).

// A namespace node has an expanded-name: the local part is the namespace prefix (this is empty if the namespace node is for the default namespace); the namespace URI is always null.

// The string-value of a namespace node is the namespace URI that is being bound to the namespace prefix; if it is relative, it must be resolved just like a namespace URI in an expanded-name.
#[derive(Debug)]
pub struct Namespace {
    name_test: NameTest,
}

impl Namespace {
    pub fn new(name_test: NameTest) -> Namespace {
        Namespace { name_test }
    }
}

impl NodeTest for Namespace {
    fn test(&self, context: &Evaluation, result: &mut Nodeset) {
        if context.node.is_namespace() && self.name_test.is_match(context, &QualName::new(None, Ns::from(""), LocalName::from(context.node.prefix()))) {
            result.add_node(context.node.clone());
        }
    }
}

#[derive(Debug)]
pub struct Element {
    name_test: NameTest,
}

impl Element {
    pub fn new(name_test: NameTest) -> Element {
        Element { name_test }
    }
}

impl NodeTest for Element {
    fn test(&self, context: &Evaluation, result: &mut Nodeset) {
        if context.node.is_element() && self.name_test.is_match(context, &context.node.name()) {
            result.add_node(context.node.clone());
        }
    }
}

#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct Node;

impl NodeTest for Node {
    fn test(&self, context: &Evaluation, result: &mut Nodeset) {
        result.add_node(context.node.clone());
    }
}

#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct Text;

impl NodeTest for Text {
    fn test(&self, context: &Evaluation, result: &mut Nodeset) {
        if let DomNode::Text(_) = context.node {
            result.add_node(context.node.clone());
        }
    }
}

#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct Comment;

impl NodeTest for Comment {
    fn test(&self, context: &Evaluation, result: &mut Nodeset) {
        if let DomNode::Comment(_) = context.node {
            result.add_node(context.node.clone());
        }
    }
}

#[derive(Debug)]
pub struct ProcessingInstruction {
    target: Option<String>,
}

impl ProcessingInstruction {
    pub fn new(target: Option<String>) -> ProcessingInstruction {
        ProcessingInstruction { target }
    }
}

impl NodeTest for ProcessingInstruction {
    fn test(&self, context: &Evaluation, result: &mut Nodeset) {
        if context.node.is_processing_instruction() {
            match self.target {
                Some(ref name) if name == &context.node.target() => result.add_node(context.node.clone()),
                Some(_) => {}
                None => result.add_node(context.node.clone()),
            }
        }
    }
}