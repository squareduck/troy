use std::borrow::Cow;
use vdom::node::VNode;

#[derive(Debug, PartialEq)]
pub struct VText {
    content: Cow<'static, str>,
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
}

pub fn text<S>(content: S) -> VText
where
    S: Into<Cow<'static, str>>,
{
    VText::new(content.into())
}
