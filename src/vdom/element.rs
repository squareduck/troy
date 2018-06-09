use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use vdom::node::VNode;
use vdom::text::VText;

type Classes = HashSet<String>;
type Attributes = HashMap<String, String>;
type Key = Option<Cow<'static, str>>;

#[derive(Debug, PartialEq)]
pub struct VElement {
    tag: Cow<'static, str>,
    key: Key,
    attributes: Attributes,
    classes: Classes,
    children: Vec<VNode>,
}

impl VElement {
    /// Create a new VElement with specified tag.
    ///
    pub fn new<S>(tag: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        VElement {
            tag: tag.into(),
            key: None,
            attributes: Attributes::new(),
            classes: Classes::new(),
            children: Vec::new(),
        }
    }

    //
    // # Getters
    //

    pub fn get_tag(&self) -> &str {
        &self.tag
    }

    pub fn get_key(&self) -> Option<&str> {
        match self.key {
            Some(ref key) => Some(key),
            None => None,
        }
    }

    pub fn get_attributes(&self) -> &Attributes {
        &self.attributes
    }

    pub fn get_classes(&self) -> &Classes {
        &self.classes
    }

    pub fn get_children(&self) -> &Vec<VNode> {
        &self.children
    }

    //
    // # Builder
    //

    /// Set a key for VElement.
    ///
    pub fn key<S>(mut self, key: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.key = Some(key.into());
        self
    }

    /// Add attribute to VElement.
    ///
    pub fn attr<S>(mut self, name: S, value: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.attributes
            .insert(name.into().into_owned(), value.into().into_owned());
        self
    }

    /// Add class to VElement.
    ///
    pub fn class<S>(mut self, name: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.classes.insert(name.into().into_owned());
        self
    }

    /// Add VElement as a child.
    ///
    pub fn child(mut self, element: VElement) -> Self {
        self.children.push(element.done());
        self
    }

    /// Add Text node as a child.
    ///
    pub fn text<S>(mut self, text: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.children.push(VNode::Text(VText::new(text.into())));
        self
    }

    /// Finish building the VElement and wrap it into VNode.
    ///
    pub fn done(self) -> VNode {
        VNode::Element(self)
    }
}

//
// # Common HTML elements
//

pub fn div() -> VElement {
    VElement::new("div")
}

pub fn p() -> VElement {
    VElement::new("p")
}
