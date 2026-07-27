use std::time::{Duration, Instant};

use crate::{buffer::GapBuffer, command::Transaction, selection::SelectionSet};

const MERGE_WINDOW: Duration = Duration::from_millis(200);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MergeKey {
    Insert,
    Other,
}

pub struct History {
    current: usize,
    nodes: Vec<Node>,
}

#[derive(Default)]
struct Node {
    parent: Option<usize>,
    children: Vec<usize>,
    transaction: Option<Transaction>,
    prior_selections: Option<SelectionSet>,
    merge_key: Option<MergeKey>,
    recorded_at: Option<Instant>,
}

impl Default for History {
    fn default() -> Self {
        History {
            current: 0,
            nodes: vec![Node::default()],
        }
    }
}

impl History {
    pub fn new() -> History {
        History::default()
    }

    pub fn record(
        &mut self,
        transaction: Transaction,
        buffer: &mut GapBuffer,
        prior_selections: SelectionSet,
        merge_key: MergeKey,
    ) {
        transaction.apply(buffer);

        if merge_key == MergeKey::Insert
            && let Some(merged) = self.coalesced_insert(&transaction)
        {
            let idx = self.current;
            self.nodes[idx].transaction = Some(merged);
            self.nodes[idx].recorded_at = Some(Instant::now());
            return;
        }

        let child = Node {
            parent: Some(self.current),
            children: vec![],
            transaction: Some(transaction),
            prior_selections: Some(prior_selections),
            merge_key: Some(merge_key),
            recorded_at: Some(Instant::now()),
        };
        let child_index = self.nodes.len();
        self.nodes.push(child);
        self.nodes[self.current].children.push(child_index);
        self.current = child_index;
    }

    fn coalesced_insert(&self, transaction: &Transaction) -> Option<Transaction> {
        let node = &self.nodes[self.current];
        if node.merge_key != Some(MergeKey::Insert) {
            return None;
        }
        let recorded_at = node.recorded_at?;
        if Instant::now().duration_since(recorded_at) > MERGE_WINDOW {
            return None;
        }
        let existing = node.transaction.as_ref()?;
        existing.coalesce_insert(transaction)
    }

    pub fn undo(&mut self, buffer: &mut GapBuffer) -> Option<SelectionSet> {
        let idx = self.current;
        let tx = self.nodes[idx].transaction.as_ref()?;
        tx.unapply(buffer);

        let restored = self.nodes[idx].prior_selections.clone();
        self.current = self.nodes[idx].parent.expect("undo at root");
        restored
    }

    pub fn redo(&mut self, buffer: &mut GapBuffer) -> Option<SelectionSet> {
        let idx = self.current;
        let child_idx = *self.nodes[idx].children.last()?;
        self.current = child_idx;

        let tx = self.nodes[child_idx].transaction.as_ref()?;
        tx.apply(buffer);
        Some(tx.new_selections().clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        buffer::{Buffer, GapBuffer},
        command::{Edit, Transaction},
        selection::{Selection, SelectionSet},
    };

    use super::*;

    #[test]
    fn undo_restores_buffer_after_one_edit() {
        let mut buffer = GapBuffer::new("hello");
        let mut history = History::new();
        let prior = SelectionSet::single(Selection::cursor(0));

        history.record(
            Transaction::new(
                vec![Edit::new(0, 5, "hello", "goodbye")],
                SelectionSet::single(Selection::cursor(0)),
            ),
            &mut buffer,
            prior,
            MergeKey::Other,
        );
        assert_eq!(buffer.slice(0, buffer.len()), "goodbye");

        history.undo(&mut buffer);
        assert_eq!(buffer.slice(0, buffer.len()), "hello");
    }

    #[test]
    fn undo_restores_selections_after_one_edit() {
        let mut buffer = GapBuffer::new("hello");
        let mut history = History::new();
        let prior = SelectionSet::single(Selection::cursor(0));
        let after = SelectionSet::single(Selection::cursor(7));

        history.record(
            Transaction::new(vec![Edit::new(0, 5, "hello", "goodbye")], after),
            &mut buffer,
            prior.clone(),
            MergeKey::Other,
        );

        let restored = history.undo(&mut buffer);
        assert_eq!(buffer.slice(0, buffer.len()), "hello");
        assert_eq!(restored, Some(prior));
    }

    #[test]
    fn redo_reapplies_after_undo() {
        let mut buffer = GapBuffer::new("hello world");
        let mut history = History::new();
        let prior = SelectionSet::single(Selection::cursor(0));
        let after = SelectionSet::single(Selection::cursor(7));

        history.record(
            Transaction::new(vec![Edit::new(0, 5, "hello", "goodbye")], after.clone()),
            &mut buffer,
            prior,
            MergeKey::Other,
        );
        history.undo(&mut buffer);

        let restored = history.redo(&mut buffer);
        assert_eq!(buffer.slice(0, buffer.len()), "goodbye world");
        assert_eq!(restored, Some(after));
    }

    #[test]
    fn record_after_undo_creates_new_branch() {
        let mut buffer = GapBuffer::new("hello");
        let mut history = History::new();
        let prior = SelectionSet::single(Selection::cursor(0));

        // branch A: hello → goodbye
        history.record(
            Transaction::new(
                vec![Edit::new(0, 5, "hello", "goodbye")],
                SelectionSet::single(Selection::cursor(0)),
            ),
            &mut buffer,
            prior.clone(),
            MergeKey::Other,
        );
        history.undo(&mut buffer); // back to "hello", current = root

        // branch B: hello → hey
        history.record(
            Transaction::new(
                vec![Edit::new(0, 5, "hello", "hey")],
                SelectionSet::single(Selection::cursor(0)),
            ),
            &mut buffer,
            prior,
            MergeKey::Other,
        );
        assert_eq!(buffer.slice(0, buffer.len()), "hey");

        history.undo(&mut buffer);
        history.redo(&mut buffer);
        assert_eq!(buffer.slice(0, buffer.len()), "hey"); // newest child, not "goodbye"
    }

    #[test]
    fn consecutive_inserts_undo_as_one_step() {
        let mut history = History::new();
        let mut buffer = GapBuffer::new("");
        let prior = SelectionSet::single(Selection::cursor(0));

        history.record(
            Transaction::new(
                vec![Edit::new(0, 0, "", "a")],
                SelectionSet::single(Selection::cursor(1)),
            ),
            &mut buffer,
            prior.clone(),
            MergeKey::Insert,
        );
        history.record(
            Transaction::new(
                vec![Edit::new(1, 1, "", "b")],
                SelectionSet::single(Selection::cursor(2)),
            ),
            &mut buffer,
            SelectionSet::single(Selection::cursor(1)),
            MergeKey::Insert,
        );
        history.record(
            Transaction::new(
                vec![Edit::new(2, 2, "", "c")],
                SelectionSet::single(Selection::cursor(3)),
            ),
            &mut buffer,
            SelectionSet::single(Selection::cursor(2)),
            MergeKey::Insert,
        );

        assert_eq!(buffer.slice(0, buffer.len()), "abc");

        history.undo(&mut buffer);
        assert_eq!(buffer.slice(0, buffer.len()), "");
    }
}
