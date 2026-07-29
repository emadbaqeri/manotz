use crate::{
    input::Mode,
    render::{byte_to_line_col, line_col_to_byte},
    selection::{Selection, SelectionSet},
};

fn selection_for_mode(anchor: usize, new_head: usize, mode: Mode) -> Selection {
    match mode {
        Mode::Select => Selection::new(anchor, new_head),
        Mode::Normal | Mode::Insert => Selection::cursor(new_head),
    }
}

pub fn motion_left(selections: &SelectionSet, mode: Mode) -> SelectionSet {
    let mut new_selections = Vec::new();
    for selection in selections {
        let new_head = selection.head().saturating_sub(1);
        let new_sel = selection_for_mode(selection.anchor(), new_head, mode);
        new_selections.push(new_sel);
    }
    SelectionSet::from_vec(new_selections)
}

pub fn motion_right(selections: &SelectionSet, buf_len: usize, mode: Mode) -> SelectionSet {
    let mut new_selections = Vec::new();
    for selection in selections {
        let new_head = (selection.head() + 1).min(buf_len);
        let new_sel = selection_for_mode(selection.anchor(), new_head, mode);

        new_selections.push(new_sel);
    }
    SelectionSet::from_vec(new_selections)
}

pub fn motion_down(selections: &SelectionSet, text: &str, mode: Mode) -> SelectionSet {
    let mut new_selections: Vec<Selection> = Vec::new();

    for selection in selections {
        let head = selection.head();
        let (line, col) = byte_to_line_col(text, head).unwrap();
        let target_line = line + 1;

        let new_head = match line_col_to_byte(text, target_line, col) {
            None => head,
            Some(byte) => byte,
        };

        let new_sel = selection_for_mode(selection.anchor(), new_head, mode);

        new_selections.push(new_sel);
    }

    SelectionSet::from_vec(new_selections)
}

pub fn motion_up(selections: &SelectionSet, text: &str, mode: Mode) -> SelectionSet {
    let mut new_selections: Vec<Selection> = Vec::new();
    for selection in selections {
        let head = selection.head();
        let (line, col) = byte_to_line_col(text, head).unwrap();

        let new_head = if line == 0 {
            head
        } else {
            let target_line = line - 1;
            line_col_to_byte(text, target_line, col).unwrap()
        };
        let new_sel = selection_for_mode(selection.anchor(), new_head, mode);

        new_selections.push(new_sel);
    }

    SelectionSet::from_vec(new_selections)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motion_move_left() {
        let selection_set = SelectionSet::single(Selection::cursor(5));
        let new_selection_set = motion_left(&selection_set, Mode::Normal);
        assert_eq!(*new_selection_set.primary(), Selection::cursor(4));
    }

    #[test]
    fn motion_move_left_clamps_at_zero() {
        let selection_set = SelectionSet::single(Selection::cursor(0));
        let new_selection_set = motion_left(&selection_set, Mode::Select);
        assert_eq!(*new_selection_set.primary(), Selection::cursor(0));
    }

    #[test]
    fn motion_move_right() {
        let selection_set = SelectionSet::single(Selection::cursor(5));
        let new_selection_set = motion_right(&selection_set, 10, Mode::Normal);
        assert_eq!(*new_selection_set.primary(), Selection::cursor(6));
    }

    #[test]
    fn motion_move_right_clamps_at_end() {
        let selection_set = SelectionSet::single(Selection::cursor(9));
        let new_selection_set = motion_right(&selection_set, 10, Mode::Normal);
        assert_eq!(*new_selection_set.primary(), Selection::cursor(10));
    }

    #[test]
    fn motion_move_right_past_end_stays_at_len() {
        let selection_set = SelectionSet::single(Selection::cursor(10));
        let new_selection_set = motion_right(&selection_set, 10, Mode::Normal);
        assert_eq!(*new_selection_set.primary(), Selection::cursor(10));
    }

    #[test]
    fn motion_move_right_single_char() {
        let selection_set = SelectionSet::single(Selection::cursor(0));
        let new_selection_set = motion_right(&selection_set, 1, Mode::Normal);
        assert_eq!(*new_selection_set.primary(), Selection::cursor(1));
    }

    #[test]
    fn motion_move_right_empty_buffer_stays_at_zero() {
        let selection_set = SelectionSet::single(Selection::cursor(0));
        let new_selection_set = motion_right(&selection_set, 0, Mode::Normal);
        assert_eq!(*new_selection_set.primary(), Selection::cursor(0));
    }

    #[test]
    fn motion_down_same_column() {
        let selection_set = SelectionSet::single(Selection::cursor(0));
        let new_selection_set = motion_down(&selection_set, "AB\nCD", Mode::Normal);
        assert_eq!(new_selection_set.primary().head(), 3);
    }

    #[test]
    fn motion_down_clamps_on_last_line() {
        let selection_set = SelectionSet::single(Selection::cursor(3));
        let new_selection_set = motion_down(&selection_set, "AB\nCD", Mode::Normal);

        assert_eq!(new_selection_set.primary().head(), 3);
    }

    #[test]
    fn motion_up_same_column() {
        let selection_set = SelectionSet::single(Selection::cursor(3));
        let new_selection_set = motion_up(&selection_set, "AB\nCD", Mode::Normal);
        assert_eq!(new_selection_set.primary().head(), 0);
    }
}
