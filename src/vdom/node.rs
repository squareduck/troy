use vdom::element::VElement;
use vdom::text::VText;
use vdom::types::CowString;

#[derive(Debug, PartialEq)]
pub enum VNode {
    Element(VElement),
    Text(VText),
}

impl VNode {
    pub fn key(&self) -> Option<&CowString> {
        match self {
            VNode::Element(element) => element.get_key(),
            _ => None,
        }
    }
}
