use node::VNode;
use std::collections::{HashMap, HashSet};
use text::VText;
use types::CowString;

type Classes = HashSet<CowString>;
type Attributes = HashMap<CowString, CowString>;
type Key = Option<CowString>;

#[derive(Debug, PartialEq)]
pub struct VElement {
    tag: CowString,
    void: bool,
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
        S: Into<CowString>,
    {
        VElement {
            tag: tag.into(),
            void: false,
            key: None,
            attributes: Attributes::new(),
            classes: Classes::new(),
            children: Vec::new(),
        }
    }

    /// Create a new void VElement with specified tag.
    /// Void elements don't have a closing tag and can't have children.
    ///
    pub fn new_void<S>(tag: S) -> Self
    where
        S: Into<CowString>,
    {
        VElement {
            tag: tag.into(),
            void: true,
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

    pub fn is_void(&self) -> bool {
        self.void
    }

    pub fn get_key(&self) -> Option<&CowString> {
        match self.key {
            Some(ref key) => Some(&key),
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
        S: Into<CowString>,
    {
        self.key = Some(key.into());
        self
    }

    /// Add attribute to VElement.
    ///
    pub fn attr<S>(mut self, name: S, value: S) -> Self
    where
        S: Into<CowString>,
    {
        self.attributes.insert(name.into(), value.into());
        self
    }

    /// Parse classlist and add each class to VElement.
    ///
    pub fn class_list<S>(mut self, classes: S) -> Self
    where
        S: Into<CowString>,
    {
        for class in classes.into().split_whitespace() {
            self.classes.insert(class.to_string().into());
        }
        self
    }

    /// Add class to VElement.
    ///
    pub fn class<S>(mut self, name: S) -> Self
    where
        S: Into<CowString>,
    {
        self.classes.insert(name.into());
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
        S: Into<CowString>,
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
