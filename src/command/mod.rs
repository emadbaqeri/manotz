use crate::{
    buffer::Buffer,
    selection::{Selection, SelectionSet},
};

mod motion;
pub use motion::motion_down;
pub use motion::motion_left;
pub use motion::motion_right;
pub use motion::motion_up;

pub struct Edit {
    start: usize,
    end: usize,
    old_text: String,
    new_text: String,
}

pub struct Transaction {
    edits: Vec<Edit>,
    new_selections: SelectionSet,
}

impl Transaction {
    pub fn new(edits: Vec<Edit>, new_selections: SelectionSet) -> Transaction {
        Transaction {
            edits,
            new_selections,
        }
    }

    pub fn apply(&self, buf: &mut impl Buffer) {
        for edit in self.edits.iter().rev() {
            buf.delete(edit.start, edit.end);
            buf.insert(edit.start, &edit.new_text);
        }
    }

    pub fn unapply(&self, buf: &mut impl Buffer) {
        for edit in self.edits.iter() {
            let (start, _) = edit.range();
            let new_end = start + edit.new_text().len();
            buf.delete(start, new_end);
            buf.insert(start, &edit.old_text());
        }
    }

    pub fn new_selections(&self) -> &SelectionSet {
        &self.new_selections
    }

    /// Merge a pure insert that continues immediately after this transaction's new text.
    /// Returns `None` if the edits can't be coalesced.
    pub fn coalesce_insert(&self, next: &Transaction) -> Option<Transaction> {
        if self.edits.len() != 1 || next.edits.len() != 1 {
            return None;
        }
        let prev = &self.edits[0];
        let ins = &next.edits[0];
        let (ins_start, ins_end) = ins.range();
        if ins_start != ins_end || !ins.old_text().is_empty() {
            return None;
        }
        let (prev_start, prev_end) = prev.range();
        let expected = prev_start + prev.new_text().len();
        if ins_start != expected {
            return None;
        }
        let merged_new = format!("{}{}", prev.new_text(), ins.new_text());
        Some(Transaction::new(
            vec![Edit::new(
                prev_start,
                prev_end,
                &prev.old_text(),
                &merged_new,
            )],
            next.new_selections().clone(),
        ))
    }
}

impl Edit {
    pub fn new(start: usize, end: usize, old_text: &str, new_text: &str) -> Edit {
        Edit {
            start,
            end,
            old_text: old_text.to_owned(),
            new_text: new_text.to_owned(),
        }
    }

    pub fn range(&self) -> (usize, usize) {
        (self.start, self.end)
    }

    pub fn old_text(&self) -> String {
        self.old_text.to_string()
    }

    pub fn new_text(&self) -> String {
        self.new_text.to_string()
    }
}

pub fn action_delete(selections: &SelectionSet, buf: &impl Buffer) -> Transaction {
    let mut edits = vec![];
    let mut cursors = vec![];
    for selection in selections {
        let start = selection.anchor().min(selection.head());
        let end = if selection.is_empty() {
            (start + 1).min(buf.len())
        } else {
            selection.anchor().max(selection.head())
        };
        let old_text = buf.slice(start, end);
        edits.push(Edit::new(start, end, old_text, ""));
        cursors.push(Selection::cursor(start));
    }
    let new_selections = SelectionSet::from_vec(cursors);
    Transaction::new(edits, new_selections)
}

#[cfg(test)]
mod tests {
    use crate::{
        buffer::{Buffer, GapBuffer},
        command::{Edit, Transaction, action_delete},
        selection::{Selection, SelectionSet},
    };

    #[test]
    fn edit_records_change() {
        let edit = Edit::new(0, 5, "hello", "world");
        assert_eq!(edit.range(), (0, 5));
        assert_eq!(edit.old_text(), "hello");
        assert_eq!(edit.new_text(), "world");
    }

    #[test]
    fn transaction_applied_edits() {
        let mut buf = GapBuffer::new("hello world");
        let tx = Transaction::new(
            vec![Edit::new(0, 5, "hello", "goodbye")],
            SelectionSet::single(Selection::cursor(7)),
        );
        tx.apply(&mut buf);
        assert_eq!(buf.slice(0, 13), "goodbye world");
    }

    #[test]
    fn transaction_unapply_restores_buffer() {
        let mut buf = GapBuffer::new("hello world");
        let tx = Transaction::new(
            vec![Edit::new(0, 5, "hello", "goodbye")],
            SelectionSet::single(Selection::cursor(7)),
        );
        tx.apply(&mut buf);
        tx.unapply(&mut buf);
        assert_eq!(buf.slice(0, buf.len()), "hello world");
    }

    #[test]
    fn action_delete_selection() {
        let mut buf = GapBuffer::new("hello world");
        let selections = SelectionSet::single(Selection::new(0, 4));
        let tx = action_delete(&selections, &buf);
        tx.apply(&mut buf);
        assert_eq!(buf.slice(0, buf.len()), "o world");
    }

    #[test]
    fn action_delete_point_cursor() {
        let mut buffer = GapBuffer::new("hello world");
        let selections = SelectionSet::single(Selection::cursor(0));
        let tx = action_delete(&selections, &buffer);
        tx.apply(&mut buffer);
        assert_eq!(buffer.slice(0, buffer.len()), "ello world")
    }

    #[test]
    fn action_delete_range_selection() {
        let mut buffer = GapBuffer::new("hello world");
        let selections = SelectionSet::single(Selection::new(0, 5));
        let tx = action_delete(&selections, &buffer);
        tx.apply(&mut buffer);
        assert_eq!(buffer.slice(0, buffer.len()), " world");
    }

    #[test]
    fn action_delete_at_eof_does_not_panic() {
        let buf = GapBuffer::new("ABC");
        let selections = SelectionSet::single(Selection::cursor(3)); // cursor at EOF
        let tx = action_delete(&selections, &buf);
        assert_eq!(tx.new_selections().primary().head(), 3);
    }
}
