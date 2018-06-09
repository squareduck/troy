use vdom::element::VElement;
use vdom::text::VText;

#[derive(Debug, PartialEq)]
pub enum VNode {
    Element(VElement),
    Text(VText),
}

impl VNode {
    pub fn get_key(&self) -> Option<&str> {
        match self {
            VNode::Element(element) => element.get_key(),
            _ => None,
        }
    }
}
