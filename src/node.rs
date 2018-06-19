use element::VElement;
use std::fmt;
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

impl fmt::Display for VNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn fmt_indent(indent_level: usize, node: &VNode, f: &mut fmt::Formatter) -> fmt::Result {
            let indent_string = "    ";
            match node {
                VNode::Element(element) => {
                    // Begin opening tag
                    write!(
                        f,
                        "{}<{}",
                        indent_string.repeat(indent_level),
                        element.get_tag()
                    )?;

                    // Classes
                    let mut classes: Vec<&CowString> = element.get_classes().iter().collect();
                    classes.sort_by(|a, b| a.cmp(b));

                    if classes.len() > 0 {
                        write!(f, " class=\"")?;
                        for (index, class) in classes.iter().enumerate() {
                            if index > 0 {
                                write!(f, " ")?;
                            }
                            write!(f, "{}", class)?;
                        }
                        write!(f, "\"")?;
                    }

                    // Attributes
                    let mut attr_pairs: Vec<(&CowString, &CowString)> =
                        element.get_attributes().iter().collect();
                    attr_pairs.sort_by(|(a, _), (b, _)| a.cmp(b));
                    for (name, value) in attr_pairs {
                        if value.len() > 0 {
                            write!(f, " {}=\"{}\"", name, value)?;
                        } else {
                            write!(f, " {}", name)?;
                        }
                    }

                    // Void elements do not have cloning tag or children.
                    if element.is_void() {
                        write!(f, ">\n")
                    } else {
                        // End opening tag
                        write!(f, ">")?;

                        // Children

                        if element.get_children().len() > 0 {
                            write!(f, "\n")?;
                        }

                        for child in element.get_children() {
                            fmt_indent(indent_level + 1, child, f)?;
                        }

                        // Closing tag
                        write!(
                            f,
                            "{}</{}>\n",
                            indent_string.repeat(indent_level),
                            element.get_tag()
                        )
                    }
                }
                VNode::Text(text) => write!(
                    f,
                    "{}{}\n",
                    indent_string.repeat(indent_level),
                    text.get_content()
                ),
            }
        }

        fmt_indent(0, self, f)
    }
}

#[cfg(test)]
mod tests {
    use tags::*;

    #[test]
    fn node_to_string() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let node = div().class_list("aaa bbb").attr("id", "ccc").attr("hidden", "")
            .child(p().class("one").text("1"))
            .child(p().class("two").text("2"))
            .child(hr())
            .child(p().class("three").text("3"))
            .done();

        let result = format!("\n{}", node.to_string());

        let expected = r#"
<div class="aaa bbb" hidden id="ccc">
    <p class="one">
        1
    </p>
    <p class="two">
        2
    </p>
    <hr>
    <p class="three">
        3
    </p>
</div>
"#;

        assert_eq!(result, expected);
    }
}
