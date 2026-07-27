#[derive(PartialEq, Debug, Clone)]
pub struct Selection {
    head: usize,   // where the cursor is (byte offset)
    anchor: usize, // where the selection started (byte offset)
}
// direction is implicit: head < anchor means backward selection
// inclusive: selecting one character is a one-width range (e.g. anchor=0, head=0)

// when an implementation method gets `self, &self, &mut self` as arg its an instance method
// when it does not it becomes and associated function

impl Selection {
    pub fn overlaps(&self, other: &Selection) -> bool {
        let a_start = self.anchor().min(self.head());
        let a_end = self.anchor().max(self.head());
        let b_start = other.anchor().min(other.head());
        let b_end = other.anchor().max(other.head());

        a_start <= b_end && b_start <= a_end
    }

    pub fn merge(&self, other: &Selection) -> Selection {
        Selection::new(
            self.anchor().min(other.anchor()),
            self.head().max(other.head()),
        )
    }

    pub fn new(anchor: usize, head: usize) -> Selection {
        Selection { anchor, head }
    }

    pub fn head(&self) -> usize {
        self.head
    }

    pub fn anchor(&self) -> usize {
        self.anchor
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    pub fn is_forward(&self) -> bool {
        self.anchor <= self.head
    }

    pub fn is_backward(&self) -> bool {
        self.anchor >= self.head
    }

    pub fn move_head(&mut self, offset: usize) {
        self.head = offset;
        self.anchor = offset;
    }

    pub fn extend(&mut self, offset: usize) {
        self.head = offset
    }

    pub fn flip(&mut self) {
        std::mem::swap(&mut self.head, &mut self.anchor);
    }

    pub fn cursor(offset: usize) -> Selection {
        Selection {
            head: offset,
            anchor: offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_at_start() {
        let sel = Selection::cursor(0);

        assert!(sel.is_empty());
        assert_eq!(sel.head(), 0);
        assert_eq!(sel.anchor(), 0);
    }

    #[test]
    fn selection_with_range() {
        let sel = Selection::new(3, 7);

        assert!(!sel.is_empty());
        assert_eq!(sel.head(), 7);
        assert_eq!(sel.anchor(), 3);
    }

    #[test]
    fn selection_direction_forward() {
        let sel = Selection::new(2, 5);
        assert!(sel.is_forward());
    }

    #[test]
    fn selection_direction_backward() {
        let sel = Selection::new(5, 2);
        assert!(sel.is_backward());
    }

    #[test]
    fn selection_move_head() {
        let mut sel = Selection::cursor(5);
        sel.move_head(10);
        assert_eq!(sel.head(), 10);
        assert_eq!(sel.anchor(), 10);
        assert!(sel.is_empty());
    }

    #[test]
    fn selection_extend() {
        let mut sel = Selection::cursor(5);
        sel.extend(10);
        assert_eq!(sel.head(), 10);
        assert_eq!(sel.anchor(), 5);
        assert!(!sel.is_empty());
    }

    #[test]
    fn selection_flip() {
        let mut sel = Selection::new(3, 7);
        sel.flip();
        assert_eq!(sel.head(), 3);
        assert_eq!(sel.anchor(), 7);
    }
}
