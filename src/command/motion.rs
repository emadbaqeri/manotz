use crate::{
    render::{byte_to_line_col, line_col_to_byte},
    selection::{Selection, SelectionSet},
};

pub fn motion_left(selections: &SelectionSet) -> SelectionSet {
    let mut new_selections = Vec::new();
    for selection in selections {
        let new_head = selection.head().saturating_sub(1);
        new_selections.push(Selection::cursor(new_head));
    }
    SelectionSet::from_vec(new_selections)
}

pub fn motion_right(selections: &SelectionSet, buf_len: usize) -> SelectionSet {
    let mut new_selections = Vec::new();
    for selection in selections {
        // Allow head == buf_len (caret after last char) so insert/backspace work at EOF.
        let new_head = (selection.head() + 1).min(buf_len);
        new_selections.push(Selection::cursor(new_head));
    }
    SelectionSet::from_vec(new_selections)
}

pub fn motion_down(selections: &SelectionSet, text: &str) -> SelectionSet {
    let mut new_selections: Vec<Selection> = Vec::new();

    for selection in selections {
        let head = selection.head();
        let (line, col) = byte_to_line_col(text, head).unwrap();
        let target_line = line + 1;

        match line_col_to_byte(text, target_line, col) {
            None => {
                new_selections.push(Selection::cursor(head));
            }
            Some(byte) => {
                new_selections.push(Selection::cursor(byte));
            }
        }
    }

    SelectionSet::from_vec(new_selections)
}

pub fn motion_up(selections: &SelectionSet, text: &str) -> SelectionSet {
    let mut new_selections: Vec<Selection> = Vec::new();
    for selection in selections {
        let head = selection.head();
        let (line, col) = byte_to_line_col(text, head).unwrap();

        if line == 0 {
            new_selections.push(Selection::cursor(head));
        } else {
            let target_line = line - 1;
            let new_head = line_col_to_byte(text, target_line, col).unwrap();
            new_selections.push(Selection::cursor(new_head));
        }
    }

    SelectionSet::from_vec(new_selections)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motion_move_left() {
        let selection_set = SelectionSet::single(Selection::cursor(5));
        let new_selection_set = motion_left(&selection_set);
        assert_eq!(*new_selection_set.primary(), Selection::cursor(4));
    }

    #[test]
    fn motion_move_left_clamps_at_zero() {
        let selection_set = SelectionSet::single(Selection::cursor(0));
        let new_selection_set = motion_left(&selection_set);
        assert_eq!(*new_selection_set.primary(), Selection::cursor(0));
    }

    #[test]
    fn motion_move_right() {
        let selection_set = SelectionSet::single(Selection::cursor(5));
        let new_selection_set = motion_right(&selection_set, 10);
        assert_eq!(*new_selection_set.primary(), Selection::cursor(6));
    }

    #[test]
    fn motion_move_right_clamps_at_end() {
        let selection_set = SelectionSet::single(Selection::cursor(9));
        let new_selection_set = motion_right(&selection_set, 10);
        assert_eq!(*new_selection_set.primary(), Selection::cursor(10));
    }

    #[test]
    fn motion_move_right_past_end_stays_at_len() {
        let selection_set = SelectionSet::single(Selection::cursor(10));
        let new_selection_set = motion_right(&selection_set, 10);
        assert_eq!(*new_selection_set.primary(), Selection::cursor(10));
    }

    #[test]
    fn motion_move_right_single_char() {
        let selection_set = SelectionSet::single(Selection::cursor(0));
        let new_selection_set = motion_right(&selection_set, 1);
        assert_eq!(*new_selection_set.primary(), Selection::cursor(1));
    }

    #[test]
    fn motion_move_right_empty_buffer_stays_at_zero() {
        let selection_set = SelectionSet::single(Selection::cursor(0));
        let new_selection_set = motion_right(&selection_set, 0);
        assert_eq!(*new_selection_set.primary(), Selection::cursor(0));
    }

    #[test]
    fn motion_down_same_column() {
        let selection_set = SelectionSet::single(Selection::cursor(0));
        let new_selection_set = motion_down(&selection_set, "AB\nCD");
        assert_eq!(new_selection_set.primary().head(), 3);
    }

    #[test]
    fn motion_down_clamps_on_last_line() {
        let selection_set = SelectionSet::single(Selection::cursor(3));
        let new_selection_set = motion_down(&selection_set, "AB\nCD");

        assert_eq!(new_selection_set.primary().head(), 3);
    }

    #[test]
    fn motion_up_same_column() {
        let selection_set = SelectionSet::single(Selection::cursor(3));
        let new_selection_set = motion_up(&selection_set, "AB\nCD");
        assert_eq!(new_selection_set.primary().head(), 0);
    }
}
