//! Nodes traversal iterator.

use html5ever::rcdom::Handle;

/// Depth-first tree traversal iterator.
pub struct Traverse {
    /// Ancestors nodes and child node indices.
    // `Vec<(parent, current_child_index)>`.
    ancestors: Vec<(Handle, usize)>,
    /// Next node.
    next: Option<Handle>,
}

impl Traverse {
    /// Creates a new `Traverse` iterator.
    pub fn new(handle: Handle) -> Self {
        Self {
            ancestors: Vec::new(),
            next: Some(handle),
        }
    }

    /// Calculates next of next, and updates the iterator state.
    fn iter_update(&mut self, current: &Handle) {
        // Check if the current node has children.
        let first_child = current.children.borrow().first().cloned();
        if let Some(first_child) = first_child {
            self.ancestors.push((current.clone(), 0));
            self.next = Some(first_child);
            return;
        }
        'next_sibling: loop {
            // Check if the next sibling exists.
            if let Some((parent, child_index)) = self.ancestors.last_mut() {
                *child_index += 1;
                if parent.children.borrow().len() == *child_index {
                    // The current node is the last sibling.
                    // Check for the parent.
                    self.ancestors.pop();
                    continue 'next_sibling;
                } else {
                    // Next sibling found.
                    self.next = Some(parent.children.borrow()[*child_index].clone());
                    return;
                }
            } else {
                // No parent. No more nodes to traverse.
                //self.next = None;  // `self.next` is already `None` here.
                return;
            }
        }
    }
}

impl Iterator for Traverse {
    type Item = Handle;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next.take()?;
        self.iter_update(&current);
        Some(current)
    }
}
