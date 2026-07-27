use crate::selection::cursor::Selection;

// stored non-overlapping selections with a primary
#[derive(Debug, PartialEq, Clone)]
pub struct SelectionSet {
    pub primary: usize,
    pub selections: Vec<Selection>,
}

impl<'a> IntoIterator for &'a SelectionSet {
    type Item = &'a Selection;
    type IntoIter = std::slice::Iter<'a, Selection>;

    fn into_iter(self) -> Self::IntoIter {
        self.selections.iter()
    }
}

// sorted by position
// Never overlapping
// Primary is an index

impl SelectionSet {
    pub fn len(&self) -> usize {
        self.selections.len()
    }

    pub fn is_empty(&self) -> bool {
        self.selections.len() == 0
    }

    pub fn primary(&self) -> &Selection {
        &self.selections[self.primary]
    }

    pub fn set_primary(&mut self, index: usize) {
        self.primary = index
    }

    pub fn single(selection: Selection) -> SelectionSet {
        SelectionSet {
            primary: 0,
            selections: vec![selection],
        }
    }

    pub fn add(&mut self, selection: Selection) {
        let mut merged = selection;
        let mut i = 0;
        while i < self.selections.len() {
            if merged.overlaps(&self.selections[i]) {
                merged = merged.merge(&self.selections.remove(i))
            } else {
                i += 1;
            }
        }
        let pos = merged.anchor().min(merged.head());
        let insert_idx = self
            .selections
            .iter()
            .position(|s| {
                let s_pos = s.anchor().min(s.head());
                s_pos > pos
            })
            .unwrap_or(self.selections.len());
        self.selections.insert(insert_idx, merged);
    }

    pub fn from_vec(selections: Vec<Selection>) -> SelectionSet {
        let mut set = SelectionSet {
            primary: 0,
            selections: vec![],
        };
        for s in selections {
            set.add(s);
        }
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_set_single() {
        let set = SelectionSet::single(Selection::cursor(5));
        assert_eq!(*set.primary(), Selection::cursor(5));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn selection_set_add_non_overlapping() {
        let mut set = SelectionSet::single(Selection::cursor(5));
        set.add(Selection::cursor(10));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn selection_set_add_sorted() {
        let mut set = SelectionSet::single(Selection::cursor(10));
        set.add(Selection::cursor(5));
        assert_eq!(set.len(), 2);
        assert_eq!(set.selections[0].head(), 5);
        assert_eq!(set.selections[1].head(), 10);
    }

    #[test]
    fn selection_set_add_overlapping_merges() {
        let mut set = SelectionSet::single(Selection::new(3, 7));
        set.add(Selection::new(5, 10));
        assert_eq!(set.len(), 1);
        assert_eq!(*set.primary(), Selection::new(3, 10));
    }

    #[test]
    fn selection_set_change_primary() {
        let mut set = SelectionSet::single(Selection::cursor(5));
        set.add(Selection::cursor(10));
        set.set_primary(1);
        assert_eq!(*set.primary(), Selection::cursor(10));
    }
}
