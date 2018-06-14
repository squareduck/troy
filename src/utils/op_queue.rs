use vdom::diff::NodeOp;

/// NodeOp Queue with size optimizations.
///
/// Reduces all sequences of `Skip` and `Remove` operations into single
/// operation with summed count.
///
/// This implementation mimicks Vec `push()` behavior, but it clones `LastOp`
/// value on each push.
///
/// TODO: Find a better way to implement this.
///
pub struct OpQueue<'new> {
    last: LastOp,
    queue: Vec<NodeOp<'new>>,
}

#[derive(Clone)]
enum LastOp {
    None,
    Skip(usize),
    Remove(usize),
}

impl<'new> OpQueue<'new> {
    pub fn new() -> Self {
        OpQueue {
            last: LastOp::None,
            queue: Vec::new(),
        }
    }

    pub fn push(&mut self, op: NodeOp<'new>) {
        use vdom::diff::NodeOp::*;

        match (self.last.clone(), op) {
            // When LastOp is None
            (LastOp::None, Skip(op_count)) => {
                self.last = LastOp::Skip(op_count);
            }
            (LastOp::None, Remove(op_count)) => {
                self.last = LastOp::Remove(op_count);
            }
            (LastOp::None, op) => {
                self.queue.push(op);
            }
            // When LastOp is Skip
            (LastOp::Skip(last_count), Skip(op_count)) => {
                self.last = LastOp::Skip(op_count + last_count);
            }
            (LastOp::Skip(last_count), Remove(op_count)) => {
                self.queue.push(Skip(last_count));
                self.last = LastOp::Remove(op_count);
            }
            (LastOp::Skip(last_count), op) => {
                self.queue.push(Skip(last_count));
                self.queue.push(op);
                self.last = LastOp::None;
            }
            // When LastOp is Remove
            (LastOp::Remove(last_count), Remove(op_count)) => {
                self.last = LastOp::Remove(op_count + last_count);
            }
            (LastOp::Remove(last_count), Skip(op_count)) => {
                self.queue.push(Remove(last_count));
                self.last = LastOp::Skip(op_count);
            }
            (LastOp::Remove(last_count), op) => {
                self.queue.push(Remove(last_count));
                self.queue.push(op);
                self.last = LastOp::None;
            }
        };
    }

    pub fn remove_single_skip(mut self) -> Self {
        match (self.queue.len(), &self.last) {
            (0, LastOp::Skip(_)) => {
                self.last = LastOp::None;
            }
            _ => {}
        }

        self
    }

    pub fn done(mut self) -> Vec<NodeOp<'new>> {
        use vdom::diff::NodeOp::*;

        match self.last {
            LastOp::Skip(last_count) => {
                self.queue.push(Skip(last_count));
            }
            LastOp::Remove(last_count) => {
                self.queue.push(Remove(last_count));
            }
            LastOp::None => {}
        }

        self.queue
    }

    pub fn done_reverse(self) -> Vec<NodeOp<'new>> {
        let mut queue = self.done();
        queue[..].reverse();
        queue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vdom::diff::NodeOp::*;
    use vdom::element::div;

    #[test]
    fn adding_skips() {
        let mut queue = OpQueue::new();

        queue.push(Skip(1));
        queue.push(Skip(2));
        queue.push(Skip(1));

        let result = queue.done();

        assert_eq!(result, vec![Skip(4)]);
    }

    #[test]
    fn adding_removes() {
        let mut queue = OpQueue::new();

        queue.push(Remove(3));
        queue.push(Remove(1));
        queue.push(Remove(5));

        let result = queue.done();

        assert_eq!(result, vec![Remove(9)]);
    }

    #[test]
    fn adding_mixed_ops() {
        let node = div().done();
        let mut queue = OpQueue::new();

        queue.push(Skip(1));
        queue.push(Skip(1));
        queue.push(Skip(1));
        queue.push(Remove(2));
        queue.push(Replace(&node));
        queue.push(Replace(&node));
        queue.push(Skip(2));
        queue.push(Skip(5));
        queue.push(Remove(1));
        queue.push(Replace(&node));
        queue.push(Remove(4));
        queue.push(Skip(4));

        let result = queue.done();

        assert_eq!(
            result,
            vec![
                Skip(3),
                Remove(2),
                Replace(&node),
                Replace(&node),
                Skip(7),
                Remove(1),
                Replace(&node),
                Remove(4),
                Skip(4),
            ]
        );
    }

    #[test]
    fn removing_single_skip() {
        let mut queue = OpQueue::new();
        queue.push(Skip(5));
        queue.push(Skip(2));
        assert_eq!(queue.remove_single_skip().done(), vec![]);

        let mut queue = OpQueue::new();
        queue.push(Skip(5));
        queue.push(Remove(4));
        assert_eq!(queue.remove_single_skip().done(), vec![Skip(5), Remove(4)]);
    }
}
