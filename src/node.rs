use element::VElement;
use text::VText;
use types::CowString;

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
