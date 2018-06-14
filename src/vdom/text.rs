use std::borrow::Cow;
use vdom::node::VNode;

type CowString = Cow<'static, str>;

#[derive(Debug, PartialEq)]
pub struct VText {
    content: CowString,
}

impl VText {
    /// Create a new VText with specified content.
    ///
    pub fn new<S>(content: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        VText {
            content: content.into(),
        }
    }

    /// Wrap text into VNode.
    ///
    pub fn done(self) -> VNode {
        VNode::Text(self)
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }
}

pub fn text<S>(content: S) -> VText
where
    S: Into<Cow<'static, str>>,
{
    VText::new(content.into())
}
