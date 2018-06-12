//! # VNode diffing
//!
//! ## Target output
//!
//! Output of the diff function is a tree structure which describes necessary
//! transformations for each node in the old VNode tree.
//!
//! If VNode has children and they were updated, diff will contain an operation
//! for each old child, and possibly new inserts.
//!
//! Each child must have an operation associated with it by the same index if
//! a node with children is to be updated.
//!
//! Some operations are reducible if they occur in sequence. For example if
//! there are several Skip(1) in a row, we can reduce them to single Skip(3).
//! It still signifies skipping three children, but takes less space.
//!
//! Position parameters (such as in Move or Insert operations) refer to
//! positon in the new children list.
//!
//! For example, for a change such as:
//! ```html
//!
//! Old:
//!
//! <div class="users">
//!     <p key="1" class="online">Ash</p>
//!     <p key="2" class="online">Bob</p>
//!     <p key="3" class="offline">Cid</p>
//!     <p key="4" class="offline">Dan</p>
//!     <p key="5" class="offline">Ela</p>
//! </div>
//!
//! New:
//!
//! <div class="users">
//!     <p key="1" class="online message">Ash</p>
//!     <p key="3" class="online">Cid</p>
//!     <p key="4" class="online">Dan</p>
//!     <p key="2" class="offline">Bobby</p>
//!     <p key="6" class="offline">Fiz</p>
//! </div>
//! ```
//!
//! We should get the following diff:
//! ```text
//! Update(
//!     None,
//!     Some(vec![
//!         Update(Some(vec![InsertClass("message")]), None, None),
//!         Move(3, None, Some(vec![Replace(VText("Bobby"))]), None),
//!         Update(Some(vec![RemoveClass("offline"), InsertClass("online")]), None, None)
//!         Update(Some(vec![RemoveClass("offline"), InsertClass("online")]), None, None)
//!         Remove(1),
//!     ]),
//!     Some(vec![
//!         (4, &p().class("offline").text("Fiz")),
//!     ])
//! )
//!
use std::collections::{HashMap, HashSet};
use utils::lis::lis;
use utils::op_queue::OpQueue;
use vdom::element::VElement;
use vdom::node::VNode;
use vdom::types::CowString;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum AttrOp {
    InsertClass(String),
    RemoveClass(String),
    Insert(String, String),
    Update(String, String),
    Remove(String),
}

pub type AttrDiff = Option<Vec<AttrOp>>;
pub type ChildDiff<'new> = Option<Vec<NodeOp<'new>>>;
pub type ChildInsert<'new> = (usize, &'new VNode);
pub type ChildInserts<'new> = Option<Vec<ChildInsert<'new>>>;

#[derive(Debug, PartialEq, Clone)]
pub enum NodeOp<'new> {
    Skip(usize),
    Remove(usize),
    Move(usize, AttrDiff, ChildDiff<'new>, ChildInserts<'new>),
    Update(AttrDiff, ChildDiff<'new>, ChildInserts<'new>),
    Replace(&'new VNode),
}

pub fn diff<'new>(old: &VNode, new: &'new VNode) -> NodeOp<'new> {
    use self::NodeOp::*;
    use vdom::node::VNode::*;

    match (old, new) {
        (Element(old_element), Element(new_element)) => {
            // Elements with different tags produce Replace.
            if old_element.get_tag() != new_element.get_tag() {
                Replace(&new)
            // Elements with different keys produce Replace.
            } else if old_element.get_key() != new_element.get_key() {
                Replace(&new)
            // Diff attributes and children lists.
            } else {
                let attr_diff = diff_attributes(old_element, new_element);
                // TODO: Diff children.
                let (children_diff, children_inserts) = diff_children(old_element, new_element);
                // Check if either of attr and children diffs returned Some.
                match (attr_diff, children_diff, children_inserts) {
                    // No diffs produce Skip.
                    (None, None, None) => Skip(1),
                    // Any diff produces Update.
                    (attr, children, inserts) => Update(attr, children, inserts),
                }
            }
        }
        // VNodes of different type produce Replace
        _ => Replace(&new),
    }
}

fn diff_attributes(old: &VElement, new: &VElement) -> AttrDiff {
    use self::AttrOp::*;

    // Find removed and inserted classes

    let old_classes = old.get_classes();
    let new_classes = new.get_classes();

    let remove_classes: Vec<AttrOp> = old_classes
        .difference(&new_classes)
        .map(|c| RemoveClass(c.clone().into_owned()))
        .collect();
    let insert_classes: Vec<AttrOp> = new_classes
        .difference(&old_classes)
        .map(|c| InsertClass(c.clone().into_owned()))
        .collect();

    let mut attr_diff: Vec<AttrOp> = vec![];

    attr_diff.extend(remove_classes);
    attr_diff.extend(insert_classes);

    // Find removed, updated, or inserted attributes

    let old_attributes = old.get_attributes();
    let new_attributes = new.get_attributes();

    let mut keys: HashSet<&CowString> = old_attributes.keys().collect();
    keys.extend(new_attributes.keys());

    for key in keys {
        match (old_attributes.get(key), new_attributes.get(key)) {
            (Some(_), None) => attr_diff.push(AttrOp::Remove(key.clone().into_owned())),
            (None, Some(value)) => attr_diff.push(AttrOp::Insert(
                key.clone().into_owned(),
                value.clone().into_owned(),
            )),
            (Some(old_value), Some(new_value)) => {
                if old_value != new_value {
                    attr_diff.push(AttrOp::Update(
                        key.clone().into_owned(),
                        new_value.clone().into_owned(),
                    ))
                }
            }
            (None, None) => {}
        }
    }

    // Return result

    if attr_diff.len() > 0 {
        Some(attr_diff)
    } else {
        None
    }
}

fn diff_children<'new>(
    old: &VElement,
    new: &'new VElement,
) -> (ChildDiff<'new>, ChildInserts<'new>) {
    use self::NodeOp::*;

    let old_children = old.get_children();
    let new_children = new.get_children();

    match (old_children.len(), new_children.len()) {
        // Both children lists are empty, no diff and no inserts.
        (0, 0) => (None, None),
        // Old children list is not empty, add Remove for each old child.
        (old_len, 0) => (Some(vec![Remove(old_len)]), None),
        // New children list is not empty, add Insert for each new child.
        (0, _) => (None, Some(new_children.iter().enumerate().collect())),
        // Both children lists are not empty
        (old_len, new_len) => {
            let mut op_queue = OpQueue::new();
            let mut inserts: Vec<ChildInsert> = Vec::new();

            // Find common prefix length
            let max_prefix_len = old_len.min(new_len);
            let mut prefix_len = 0;
            for i in 0..max_prefix_len {
                // For unkeyed children this is always true
                if old_children[i].key() == new_children[i].key() {
                    prefix_len += 1;
                }
            }

            // Find common suffix length
            let max_suffix_len = max_prefix_len - prefix_len;
            let mut suffix_len = 0;
            for i in (0..max_suffix_len).rev() {
                if old_children[i].key() == new_children[i].key() {
                    suffix_len += 1;
                }
            }
            // Calculate middle length for both lists
            let old_middle_len = old_len - (prefix_len + suffix_len);
            let new_middle_len = new_len - (prefix_len + suffix_len);

            // Push operations for common prefix
            for i in 0..prefix_len {
                op_queue.push(diff(&old_children[i], &new_children[i]));
            }

            // Push operations for middle
            match (old_middle_len, new_middle_len) {
                // Both middles are empty, do nothing
                (0, 0) => {}
                // New middle is empty, add Remove for each old middle child
                (old_middle_len, 0) => op_queue.push(Remove(old_middle_len)),
                // Old middle is empty, add Insert for each new middle child
                (0, new_middle_len) => {
                    for i in prefix_len..(prefix_len + new_middle_len) {
                        inserts.push((i, &new_children[i]));
                    }
                }
                // TODO: Collecting references to VNodes can be suboptimal if
                // the middle is big.
                //
                // Maybe send refenece to old and new children, and ranges?
                //
                (old_middle_len, new_middle_len) => {
                    let old_middle_children: Vec<&VNode> = old_children
                        [prefix_len..(prefix_len + old_middle_len)]
                        .iter()
                        .collect();

                    let new_middle_children: Vec<&VNode> = new_children
                        [prefix_len..(prefix_len + new_middle_len)]
                        .iter()
                        .collect();

                    diff_middles(
                        &mut op_queue,
                        &mut inserts,
                        old_middle_children,
                        new_middle_children,
                    );
                }
            };

            // Push operations for common suffix
            let old_suffix_start = old_len - suffix_len;
            let new_suffix_start = new_len - suffix_len;

            for i in 0..suffix_len {
                op_queue.push(diff(
                    &old_children[old_suffix_start + i],
                    &new_children[new_suffix_start + i],
                ));
            }

            // Extract operations and generate final results
            let mut ops = op_queue.remove_single_skip().done();

            match (ops.len(), inserts.len()) {
                (0, 0) => (None, None),
                (0, _) => (None, Some(inserts)),
                (_, 0) => (Some(ops), None),
                (_, _) => (Some(ops), Some(inserts)),
            }
        }
    }
}

// TODO: Implement middle children reconciliation
fn diff_middles(
    op_queue: &mut OpQueue,
    inserts: &mut Vec<ChildInsert>,
    old_children: Vec<&VNode>,
    new_children: Vec<&VNode>,
) {
}

#[cfg(test)]
mod tests {
    use super::NodeOp::*;
    use super::*;
    use vdom::element::{div, p};
    use vdom::text::text;

    //
    // # Comparing types and tags
    //

    #[test]
    fn different_vnode_types() {
        let old = div().done();
        let new = text("").done();

        let result = diff(&old, &new);

        assert_eq!(result, Replace(&new));
    }

    #[test]
    fn different_velement_tags() {
        let old = div().done();
        let new = p().done();

        let result = diff(&old, &new);

        assert_eq!(result, Replace(&new));
    }

    #[test]
    fn same_tags() {
        let old = div().done();
        let new = div().done();

        let result = diff(&old, &new);

        assert_eq!(result, Skip(1));
    }

    //
    // # Comparing attributes
    //

    #[test]
    fn same_tags_with_different_keys() {
        let old = div().key("a").done();
        let new = div().key("b").done();

        let result = diff(&old, &new);

        assert_eq!(result, Replace(&new));
    }

    #[test]
    fn same_tags_with_same_classes() {
        let old = div().class_list("aaa bbb").done();
        let new = div().class_list("aaa bbb").done();

        let result = diff(&old, &new);

        assert_eq!(result, Skip(1));
    }

    #[test]
    fn same_tags_with_different_classes() {
        let old = div().class_list("aaa bbb").done();
        let new = div().class_list("bbb ccc").done();

        let result = diff(&old, &new);

        assert_eq!(
            result,
            Update(
                Some(vec![
                    AttrOp::RemoveClass("aaa".to_string()),
                    AttrOp::InsertClass("ccc".to_string()),
                ]),
                None,
                None
            )
        );
    }

    #[test]
    fn same_tags_with_same_attributes() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let old = div()
            .attr("attr_a", "aaa")
            .attr("attr_b", "bbb")
            .done();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = div()
            .attr("attr_a", "aaa")
            .attr("attr_b", "bbb")
            .done();

        let result = diff(&old, &new);

        assert_eq!(result, Skip(1));
    }

    #[test]
    fn same_tags_with_different_attributes() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let old = div()
            .attr("attr_a", "aaa")
            .attr("attr_b", "bbb")
            .attr("attr_c", "ccc")
            .done();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = div()
            .attr("attr_b", "bbb")
            .attr("attr_c", "***")
            .attr("attr_d", "ddd")
            .done();

        let result = diff(&old, &new);

        if let Update(Some(attr_diff), None, None) = result {
            assert_eq!(attr_diff.len(), 3);
            assert!(attr_diff.contains(&AttrOp::Remove("attr_a".to_string())));
            assert!(attr_diff.contains(&AttrOp::Update("attr_c".to_string(), "***".to_string())));
            assert!(attr_diff.contains(&AttrOp::Insert("attr_d".to_string(), "ddd".to_string())));
        } else {
            panic!("No attribute diff.")
        }
    }

    //
    // # Comparing unkeyed children
    //

    #[test]
    fn same_unkeyed_children() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let old = div()
            .child(p())
            .child(p())
            .child(p())
            .done();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = div()
            .child(p())
            .child(p())
            .child(p())
            .done();

        let result = diff(&old, &new);

        assert_eq!(result, Skip(1));
    }

    #[test]
    fn inserted_all_unkeyed_children() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let old = div()
            .done();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = div()
            .child(p())
            .child(p())
            .child(p())
            .done();

        let result = diff(&old, &new);

        assert_eq!(
            result,
            Update(
                None,
                None,
                Some(vec![(0, &p().done()), (1, &p().done()), (2, &p().done())])
            )
        );
    }

    #[test]
    fn removed_all_unkeyed_children() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let old = div()
            .child(p())
            .child(p())
            .child(p())
            .done();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = div()
            .done();

        let result = diff(&old, &new);

        assert_eq!(result, Update(None, Some(vec![Remove(3)]), None));
    }

    #[test]
    fn prepended_unkeyed_children() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let old = div()
            .child(p())
            .done();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = div()
            .child(div())
            .child(div())
            .child(p())
            .done();

        let result = diff(&old, &new);

        assert_eq!(
            result,
            Update(
                None,
                Some(vec![Replace(&div().done())]),
                Some(vec![(1, &div().done()), (2, &p().done())])
            )
        )
    }

    #[test]
    fn inserted_unkeyed_children() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let old = div()
            .child(p())
            .child(p())
            .done();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = div()
            .child(p())
            .child(div())
            .child(p())
            .done();

        let result = diff(&old, &new);

        assert_eq!(
            result,
            Update(
                None,
                Some(vec![Skip(1), Replace(&div().done())]),
                Some(vec![(2, &p().done())])
            )
        )
    }

    #[test]
    fn appended_unkeyed_children() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let old = div()
            .child(p())
            .done();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = div()
            .child(p())
            .child(div())
            .child(div())
            .done();

        let result = diff(&old, &new);

        assert_eq!(
            result,
            Update(
                None,
                None,
                Some(vec![(1, &div().done()), (2, &div().done())])
            )
        )
    }

    #[test]
    fn inserted_and_modified_unkeyed_children() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let old = div()
            .child(div()
                .child(p())
                .child(p())
            )
            .child(div()
                .child(div())
            )
            .done();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = div()
            .child(div()
                .child(p())
                .child(p())
                .child(p())
            )
            .child(p().text("Hello"))
            .child(div())
            .done();

        let result = diff(&old, &new);

        assert_eq!(
            result,
            Update(
                None,
                Some(vec![
                    Update(None, None, Some(vec![(2, &p().done())])),
                    Replace(&p().text("Hello").done()),
                ]),
                Some(vec![(2, &div().done())])
            )
        );
    }
}
