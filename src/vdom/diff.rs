use std::collections::{HashMap, HashSet};
use utils::op_queue::OpQueue;
use vdom::element::VElement;
use vdom::node::VNode;

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

#[derive(Debug, PartialEq, Clone)]
pub enum NodeOp<'new> {
    Skip(usize),
    Remove(usize),
    Insert(usize, &'new VNode),
    Update(AttrDiff, ChildDiff<'new>),
    Replace(&'new VNode),
}

// Find the optimal way to transform `old` VNode into `new` VNode.
//
// Returns a tree of `NodeOp`s describing the transformation.
//
// # Steps
//
// 1. If VNodes are of different types, return `Replace(new)`.
// 2. If VNodes are both `Element`:
//     2.a If tags are different, return `Replace(new)`.
//     2.b Diff attributes
//     2.c Diff children
//     2.d If attributes and/or children are different,
//         return `Update(attr, children)`.
//         If not - return `Skip(1)`.
//
// 3. Otherwise, return `Replace(new)`.
//
pub fn diff<'new>(old: &VNode, new: &'new VNode) -> NodeOp<'new> {
    use vdom::node::VNode::Element;

    match (old, new) {
        (Element(old_element), Element(new_element)) => {
            if old_element.get_tag() != new_element.get_tag() {
                NodeOp::Replace(new)
            } else {
                let attr_diff = diff_attributes(old_element, new_element);
                let children_diff = diff_children(old_element, new_element);

                match (attr_diff, children_diff) {
                    (None, None) => NodeOp::Skip(1),
                    (attr, children) => NodeOp::Update(attr, children),
                }
            }
        }
        _ => NodeOp::Replace(new),
    }
}

fn diff_attributes(old: &VElement, new: &VElement) -> AttrDiff {
    let mut diff: Vec<AttrOp> = Vec::new();

    // Diff classes
    //

    let old_classes = old.get_classes();
    let new_classes = new.get_classes();

    let remove_classes = old_classes
        .difference(new_classes)
        .map(|class| AttrOp::RemoveClass(class.to_string()));

    let insert_classes = new_classes
        .difference(old_classes)
        .map(|class| AttrOp::InsertClass(class.to_string()));

    diff.extend(remove_classes);
    diff.extend(insert_classes);

    // Diff other attributes
    //

    let old_attributes = old.get_attributes();
    let new_attributes = new.get_attributes();

    let mut keys: HashSet<&String> = old_attributes.keys().collect();
    keys.extend(new_attributes.keys());

    for key in keys {
        match (old_attributes.get(key), new_attributes.get(key)) {
            (Some(_), None) => diff.push(AttrOp::Remove(key.to_string())),
            (None, Some(value)) => diff.push(AttrOp::Insert(key.to_string(), value.to_string())),
            (Some(old_value), Some(new_value)) => {
                if old_value != new_value {
                    diff.push(AttrOp::Update(key.to_string(), new_value.to_string()))
                }
            }
            (None, None) => {}
        }
    }

    if diff.len() > 0 {
        Some(diff)
    } else {
        None
    }
}

fn diff_children<'new>(old: &VElement, new: &'new VElement) -> ChildDiff<'new> {
    let old_children = old.get_children();
    let new_children = new.get_children();

    // Find common prefix
    //

    let old_children_len = old_children.len();
    let new_children_len = new_children.len();

    // If no children return None right away
    if old_children_len == 0 && new_children_len == 0 {
        return None;
    }

    let min_len = old_children_len.min(new_children_len);
    let mut prefix_len = 0;

    // Find prefix length
    for index in 0..min_len {
        // Unkeyed elements are treated as having the same key.
        let old_key = old_children[index].get_key();
        let new_key = new_children[index].get_key();
        if old_children[index].get_key() == new_children[index].get_key() {
            prefix_len += 1;
        } else {
            break;
        }
    }

    // Generate prefix ops
    let mut prefix_queue = OpQueue::new();
    for index in 0..prefix_len {
        prefix_queue.push(diff(&old_children[index], &new_children[index]));
    }
    let prefix_ops = prefix_queue.done();

    // Find suffix length
    let possible_suffix_len = min_len - prefix_len;
    let mut suffix_len = 0;
    for index in 0..possible_suffix_len {
        if old_children[old_children_len - index - 1].get_key()
            == new_children[new_children_len - index - 1].get_key()
        {
            suffix_len += 1;
        }
    }

    // Generate suffix ops
    // TODO: Get rid of reverse()
    let mut suffix_queue = OpQueue::new();
    for index in 0..suffix_len {
        suffix_queue.push(diff(&old_children[index], &new_children[index]));
    }
    let suffix_ops = suffix_queue.done_reverse();

    // Find middle length
    let old_middle_len = old_children_len - prefix_len - suffix_len;
    let new_middle_len = new_children_len - prefix_len - suffix_len;

    // Check if one or both of the middles is empty
    // If it is - we can finish here by populating middle ops with
    // Removes or Inserts

    let mut middle_queue = OpQueue::new();

    match (old_middle_len, new_middle_len) {
        // Both middles are empty, do nothing
        (0, 0) => {}
        // Old middle is not empty, remove old nodes
        (_, 0) => middle_queue.push(NodeOp::Remove(old_middle_len)),
        // New middle is not empty, insert new nodes
        (0, _) => {
            let mut index = 0;
            for child in new_children[prefix_len..(prefix_len + new_middle_len)].iter() {
                middle_queue.push(NodeOp::Insert(prefix_len + index, child));
                index += 1;
            }
        }
        // Both middles are not empty, find operation sequece for them
        _ => {
            let old_middle_children: Vec<&VNode> = old_children
                [prefix_len..(prefix_len + old_middle_len)]
                .iter()
                .collect();
            let new_middle_children: Vec<&VNode> = new_children
                [prefix_len..(prefix_len + new_middle_len)]
                .iter()
                .collect();
            diff_middle_children(
                &mut middle_queue,
                prefix_len,
                old_middle_children,
                new_middle_children,
            );
        }
    }
    let middle_ops = middle_queue.done();

    // Decide if children are different
    fn skip_or_empty(ops: &Vec<NodeOp>) -> bool {
        match ops.as_slice() {
            [NodeOp::Skip(_)] => true,
            [] => true,
            _ => false,
        }
    }

    if prefix_ops.len() + suffix_ops.len() + middle_ops.len() > 0 {
        let prefix_skip = skip_or_empty(&prefix_ops);
        let middle_skip = skip_or_empty(&middle_ops);
        let suffix_skip = skip_or_empty(&suffix_ops);

        if prefix_skip && middle_skip && suffix_skip {
            None
        } else {
            let mut ops: Vec<NodeOp> = Vec::new();
            ops.extend(prefix_ops);
            ops.extend(middle_ops);
            ops.extend(suffix_ops);
            Some(ops)
        }
    } else {
        None
    }
}

// Search for inserted, replaced and moved nodes between two lists of children.
//
// Inspired by:
// https://github.com/localvoid/ivi/
//
// Nodes without keys are treated as unique and are always replaced.
fn diff_middle_children<'new>(
    queue: &mut OpQueue<'new>,
    offset: usize,
    old_children: Vec<&VNode>,
    new_children: Vec<&'new VNode>,
) {
    // Create positions array with length of new children list. It will hold
    // position of each new child in old children list.
    //
    // By default we assume new child is not in old children list, so we fill
    // the array with None.
    let mut positions: Vec<Option<usize>> = vec![None; new_children.len()];

    // Create a list of predicted inserts
    let mut predicted_inserts: Vec<NodeOp<'new>> = vec![];

    // Create map between new children keys and their position in the list.
    let mut new_children_map: HashMap<&str, usize> = HashMap::new();
    for (index, child) in new_children.iter().enumerate() {
        if let Some(key) = child.get_key() {
            new_children_map.insert(key, index);
        // If key is not set we assume that child is new and predict that it
        // is going to be inserted.
        } else {
            //predicted_inserts.push(NodeOp::Insert(index, child));
        }
    }

    // Create a list of predicted operations for old children.
    // Operation refers to the old child with the same index.
    let mut predicted_ops: Vec<NodeOp<'new>> = vec![NodeOp::Skip(1); old_children.len()];

    // Iterate over old children list and for each child try to find a node
    // with the same key in new_children_map.
    // Remember last seen position in the new children list.
    // If current position is larger than last position - we set `moved`
    // to `true`.
    let mut last_position: Option<usize> = None;
    let mut moved = false;
    let mut removed_keys: usize = 0;
    for (index, child) in old_children.iter().enumerate() {
        if let Some(key) = child.get_key() {
            // If the old child is found in new_children_map we put the old
            // position into positions array.
            if let Some(position_index) = new_children_map.get(key) {
                // Toggle `moved` flag if current position is larger than last.
                if let Some(last_position) = last_position {
                    if last_position > *position_index {
                        moved = true;
                    }
                }
                last_position = Some(*position_index);
                positions[*position_index] = Some(index);
            // If the old child is not found in new_children_map we predict it
            // is going to be removed.
            } else {
                predicted_ops[index] = NodeOp::Remove(1);
                removed_keys += 1;
            }

        // If key is not set we assume the child is going to be removed.
        } else {
            //predicted_ops[index] = NodeOp::Remove(1);
        }
    }

    // If all old children are to be removed, add an insert for each new child.
    if !moved && old_children.len() == removed_keys {
        for (index, child) in new_children.iter().enumerate() {
            predicted_inserts.push(NodeOp::Insert(offset + index, child))
        }
    }

    // TODO: Check moved flag and sizes and decide if we can finish
    // or need to continue

    // Build a queue from predictions
    for op in predicted_ops {
        queue.push(op);
    }

    for op in predicted_inserts {
        queue.push(op);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vdom::element::{div, p};
    use vdom::text::text;

    //
    // Basic node diffing
    //

    #[test]
    fn different_types() {
        let old = div().done();
        let new = text("").done();

        let result = diff(&old, &new);

        assert_eq!(result, NodeOp::Replace(&new));
    }

    #[test]
    fn different_tags() {
        let old = div().done();
        let new = p().done();

        let result = diff(&old, &new);

        assert_eq!(result, NodeOp::Replace(&new));
    }

    #[test]
    fn toggled_class() {
        let old = div().class("user").class("offline").done();
        let new = div().class("user").class("online").done();

        let result = diff(&old, &new);

        assert_eq!(
            result,
            NodeOp::Update(
                Some(vec![
                    AttrOp::RemoveClass("offline".to_string()),
                    AttrOp::InsertClass("online".to_string()),
                ]),
                None
            )
        )
    }

    #[test]
    fn updated_attributes() {
        let old = div().attr("hidden", "").attr("id", "1").done();

        let new = div().attr("data-user", "username").attr("id", "2").done();

        let result = diff(&old, &new);

        // Need to sort because otherwise order is not guaranteed.
        //
        if let NodeOp::Update(Some(mut ops), _) = result {
            ops.sort();
            assert_eq!(
                ops,
                vec![
                    AttrOp::Insert("data-user".to_string(), "username".to_string()),
                    AttrOp::Update("id".to_string(), "2".to_string()),
                    AttrOp::Remove("hidden".to_string()),
                ]
            )
        } else {
            panic!("Got no ops");
        }
    }

    //
    // # Children reconciliation
    //

    #[test]
    fn same_unkeyed_children() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let old = div().key("p1")
            .child(p())
            .child(p())
            .done();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = div().key("p1")
            .child(p())
            .child(p())
            .done();

        let result = diff(&old, &new);

        assert_eq!(result, NodeOp::Skip(1));
    }

    fn different_unkeyed_children() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let old = div().key("p1")
            .child(p())
            .child(div())
            .done();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = div().key("p1")
            .child(div())
            .child(p())
            .done();

        let result = diff(&old, &new);

        assert_eq!(
            result,
            NodeOp::Update(
                None,
                Some(vec![
                    NodeOp::Replace(&div().done()),
                    NodeOp::Replace(&p().done()),
                ])
            )
        );
    }

    #[test]
    fn same_keyed_children() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let old = div().key("p1")
            .child(p().key("c1"))
            .child(p().key("c2"))
            .done();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = div().key("p1")
            .child(p().key("c1"))
            .child(p().key("c2"))
            .done();

        let result = diff(&old, &new);

        assert_eq!(result, NodeOp::Skip(1));
    }

    #[test]
    fn different_keyed_children() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let old = div().key("p1")
            .child(p().key("c1"))
            .child(p().key("c2"))
            .done();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = div().key("p1")
            .child(p().key("c3"))
            .child(p().key("c4"))
            .done();

        let result = diff(&old, &new);

        assert_eq!(
            result,
            NodeOp::Update(
                None,
                Some(vec![
                    NodeOp::Remove(2),
                    NodeOp::Insert(0, &p().key("c3").done()),
                    NodeOp::Insert(1, &p().key("c4").done()),
                ])
            )
        );
    }

    #[test]
    fn different_middle_keyed_children() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let old = div().key("p1")
            .child(p().key("c1"))
            .child(p().key("c2"))
            .child(p().key("c3"))
            .child(p().key("c4"))
            .child(p().key("c5"))
            .done();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = div().key("p1")
            .child(p().key("c1"))
            .child(p().key("c2"))
            .child(p().key("d1"))
            .child(p().key("d2"))
            .child(p().key("c5"))
            .done();

        let result = diff(&old, &new);

        assert_eq!(
            result,
            NodeOp::Update(
                None,
                Some(vec![
                    NodeOp::Skip(2),
                    NodeOp::Remove(2),
                    NodeOp::Insert(2, &p().key("d1").done()),
                    NodeOp::Insert(3, &p().key("d2").done()),
                    NodeOp::Skip(1),
                ])
            )
        );
    }

    #[test]
    fn prepended_keyed_children() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let old = div().key("p1")
            .child(p().key("c1"))
            .child(p().key("c2"))
            .done();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = div().key("p1")
            .child(p().key("c3"))
            .child(p().key("c4"))
            .child(p().key("c5"))
            .child(p().key("c1"))
            .child(p().key("c2"))
            .done();

        let result = diff(&old, &new);

        assert_eq!(
            result,
            NodeOp::Update(
                None,
                Some(vec![
                    NodeOp::Insert(0, &p().key("c3").done()),
                    NodeOp::Insert(1, &p().key("c4").done()),
                    NodeOp::Insert(2, &p().key("c5").done()),
                    NodeOp::Skip(2),
                ])
            )
        );
    }

    #[test]
    fn inserted_keyed_children() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let old = div().key("p1")
            .child(p().key("c1"))
            .child(p().key("c2"))
            .child(p().key("c5"))
            .done();

        let new = div()
            .key("p1")
            .child(p().key("c1"))
            .child(p().key("c2"))
            .child(p().key("c3"))
            .child(p().key("c4"))
            .child(p().key("c5"))
            .done();

        let result = diff(&old, &new);

        assert_eq!(
            result,
            NodeOp::Update(
                None,
                Some(vec![
                    NodeOp::Skip(2),
                    NodeOp::Insert(2, &p().key("c3").done()),
                    NodeOp::Insert(3, &p().key("c4").done()),
                    NodeOp::Skip(1),
                ])
            )
        );
    }

    #[test]
    fn appended_keyed_children() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let old = div().key("p1")
            .child(p().key("c1"))
            .child(p().key("c2"))
            .done();

        let new = div()
            .key("p1")
            .child(p().key("c1"))
            .child(p().key("c2"))
            .child(p().key("c3"))
            .child(p().key("c4"))
            .child(p().key("c5"))
            .done();

        let result = diff(&old, &new);

        assert_eq!(
            result,
            NodeOp::Update(
                None,
                Some(vec![
                    NodeOp::Skip(2),
                    NodeOp::Insert(2, &p().key("c3").done()),
                    NodeOp::Insert(3, &p().key("c4").done()),
                    NodeOp::Insert(4, &p().key("c5").done()),
                ])
            )
        );
    }
}
