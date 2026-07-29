use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, PartialEq)]
pub enum Action {
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveUp,
    InsertChar(char),
    EnterInsert,
    EnterNormal,
    Backspace,
    Undo,
    Redo,
    Quit, // 'q' or Ctrl+C
    EnterSelect,
    Delete,
    Change,
    Yank,
    Paste,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Mode {
    Normal,
    Insert,
    Select,
}

impl Action {
    pub fn map_key(key: KeyEvent, mode: Mode) -> Option<Action> {
        match mode {
            Mode::Normal => match key.code {
                KeyCode::Left => Some(Action::MoveLeft),
                KeyCode::Right => Some(Action::MoveRight),
                KeyCode::Up => Some(Action::MoveUp),
                KeyCode::Down => Some(Action::MoveDown),
                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Char('i') => Some(Action::EnterInsert),
                KeyCode::Char('u') => Some(Action::Undo),
                KeyCode::Char('U') => Some(Action::Redo),
                KeyCode::Char('v') => Some(Action::EnterSelect),
                KeyCode::Char('y') => Some(Action::Yank),
                KeyCode::Char('d') => Some(Action::Delete),
                KeyCode::Char('c') => Some(Action::Change),
                KeyCode::Char('p') => Some(Action::Paste),
                _ => None,
            },
            Mode::Insert => match key.code {
                KeyCode::Esc => Some(Action::EnterNormal),
                KeyCode::Left => Some(Action::MoveLeft),
                KeyCode::Right => Some(Action::MoveRight),
                KeyCode::Up => Some(Action::MoveUp),
                KeyCode::Down => Some(Action::MoveDown),
                KeyCode::Char(ch) => Some(Action::InsertChar(ch)),
                KeyCode::Backspace => Some(Action::Backspace),
                KeyCode::Enter => Some(Action::InsertChar('\n')),
                _ => None,
            },
            Mode::Select => match key.code {
                KeyCode::Esc | KeyCode::Char('v') => Some(Action::EnterNormal),
                KeyCode::Left => Some(Action::MoveLeft),
                KeyCode::Right => Some(Action::MoveRight),
                KeyCode::Up => Some(Action::MoveUp),
                KeyCode::Down => Some(Action::MoveDown),
                KeyCode::Char('y') => Some(Action::Yank),
                KeyCode::Char('d') => Some(Action::Delete),
                KeyCode::Char('c') => Some(Action::Change),
                KeyCode::Char('p') => Some(Action::Paste),

                _ => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Action;
    use crate::{buffer::Buffer, editor::EditorState, input::Mode};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    const NO_MODS: KeyModifiers = KeyModifiers::NONE;

    #[test]
    fn map_key_left_arrow() {
        let left_key = KeyEvent::new(KeyCode::Left, NO_MODS);
        let action_result = Action::map_key(left_key, Mode::Normal);
        assert_eq!(action_result, Some(Action::MoveLeft));
    }

    #[test]
    fn map_key_right_arrow() {
        let right_key = KeyEvent::new(KeyCode::Right, NO_MODS);
        let action_result = Action::map_key(right_key, Mode::Normal);
        assert_eq!(action_result, Some(Action::MoveRight));
    }

    #[test]
    fn map_key_quit() {
        let q_key = KeyEvent::new(KeyCode::Char('q'), NO_MODS);
        let action_result = Action::map_key(q_key, Mode::Normal);
        assert_eq!(action_result, Some(Action::Quit));
    }

    #[test]
    fn map_key_down_arrow() {
        let down_key = KeyEvent::new(KeyCode::Down, NO_MODS);
        let action_result = Action::map_key(down_key, Mode::Normal);
        assert_eq!(action_result, Some(Action::MoveDown));
    }

    #[test]
    fn map_key_up_arrow() {
        let up_key = KeyEvent::new(KeyCode::Up, NO_MODS);
        let action_result = Action::map_key(up_key, Mode::Normal);
        assert_eq!(action_result, Some(Action::MoveUp));
    }

    #[test]
    fn update_enter_insert_sets_insert_mode() {
        let state = EditorState::new("ABC", 5, 5);
        let next = state.update(Action::EnterInsert);
        assert_eq!(next.mode, Mode::Insert);
        assert_eq!(next.buffer.slice(0, next.buffer.len()), "ABC");
        assert_eq!(next.selections.primary().head(), 0);
    }

    #[test]
    fn update_enter_normal_sets_normal_mode() {
        let state = EditorState::new("ABC", 5, 5);
        let state = state.update(Action::EnterInsert);
        let next = state.update(Action::EnterNormal);

        assert_eq!(next.mode, Mode::Normal);
        assert_eq!(next.buffer.slice(0, next.buffer.len()), "ABC");
        assert_eq!(next.selections.primary().head(), 0);
    }

    #[test]
    fn map_key_i_enters_insert_in_normal_mode() {
        let i_key = KeyEvent::new(KeyCode::Char('i'), NO_MODS);
        let action_result = Action::map_key(i_key, Mode::Normal);
        assert_eq!(action_result, Some(Action::EnterInsert));
        assert_ne!(action_result, Some(Action::EnterNormal));
    }

    #[test]
    fn map_key_i_only_enters_insert_in_normal_mode() {
        let i_key = KeyEvent::new(KeyCode::Char('i'), NO_MODS);
        assert_eq!(
            Action::map_key(i_key, Mode::Normal,),
            Some(Action::EnterInsert)
        );
    }

    #[test]
    fn map_key_esc_enters_normal_in_insert_mode() {
        let esc = KeyEvent::new(KeyCode::Esc, NO_MODS);
        assert_eq!(
            Action::map_key(esc, Mode::Insert),
            Some(Action::EnterNormal)
        );
        assert_eq!(Action::map_key(esc, Mode::Normal), None);
    }

    #[test]
    fn map_key_char_inserts_in_insert_mode() {
        let key = KeyEvent::new(KeyCode::Char('X'), NO_MODS);
        assert_eq!(
            Action::map_key(key, Mode::Insert),
            Some(Action::InsertChar('X'))
        );
        assert_eq!(Action::map_key(key, Mode::Normal), None);
    }

    #[test]
    fn map_key_backspace_in_insert_mode() {
        let key = KeyEvent::new(KeyCode::Backspace, NO_MODS);
        assert_eq!(Action::map_key(key, Mode::Insert), Some(Action::Backspace));
        assert_eq!(Action::map_key(key, Mode::Normal), None);
    }

    #[test]
    fn map_key_enter_inserts_newline_in_insert_mode() {
        let key = KeyEvent::new(KeyCode::Enter, NO_MODS);
        assert_eq!(
            Action::map_key(key, Mode::Insert),
            Some(Action::InsertChar('\n'))
        );
        assert_eq!(Action::map_key(key, Mode::Normal), None);
    }

    #[test]
    fn map_key_arrows_work_in_insert_mode() {
        assert_eq!(
            Action::map_key(KeyEvent::new(KeyCode::Left, NO_MODS), Mode::Insert),
            Some(Action::MoveLeft)
        );
        assert_eq!(
            Action::map_key(KeyEvent::new(KeyCode::Right, NO_MODS), Mode::Insert),
            Some(Action::MoveRight)
        );
        assert_eq!(
            Action::map_key(KeyEvent::new(KeyCode::Up, NO_MODS), Mode::Insert),
            Some(Action::MoveUp)
        );
        assert_eq!(
            Action::map_key(KeyEvent::new(KeyCode::Down, NO_MODS), Mode::Insert),
            Some(Action::MoveDown)
        );
    }

    #[test]
    fn map_key_u_is_undo_in_normal() {
        let key = KeyEvent::new(KeyCode::Char('u'), NO_MODS);
        assert_eq!(Action::map_key(key, Mode::Normal), Some(Action::Undo));
    }

    #[test]
    fn map_key_capital_u_is_redo_in_normal() {
        let key = KeyEvent::new(KeyCode::Char('U'), NO_MODS);
        assert_eq!(Action::map_key(key, Mode::Normal), Some(Action::Redo));
    }

    #[test]
    fn map_key_capital_u_inserts_in_insert_mode() {
        let key = KeyEvent::new(KeyCode::Char('U'), NO_MODS);
        assert_eq!(
            Action::map_key(key, Mode::Insert),
            Some(Action::InsertChar('U'))
        );
    }

    #[test]
    fn map_key_v_enters_select_mode() {
        let v_key = KeyEvent::new(KeyCode::Char('v'), NO_MODS);
        assert_eq!(
            Action::map_key(v_key, Mode::Normal),
            Some(Action::EnterSelect)
        );
    }

    #[test]
    fn map_key_y_is_yank_in_normal_and_select() {
        let key = KeyEvent::new(KeyCode::Char('y'), NO_MODS);
        assert_eq!(Action::map_key(key, Mode::Normal), Some(Action::Yank));
        assert_eq!(Action::map_key(key, Mode::Select), Some(Action::Yank));
    }

    #[test]
    fn map_key_d_is_delete_in_normal_and_select() {
        let key = KeyEvent::new(KeyCode::Char('d'), NO_MODS);
        assert_eq!(Action::map_key(key, Mode::Normal), Some(Action::Delete));
        assert_eq!(Action::map_key(key, Mode::Select), Some(Action::Delete));
    }

    #[test]
    fn map_key_c_is_change_in_normal_and_select() {
        let key = KeyEvent::new(KeyCode::Char('c'), NO_MODS);
        assert_eq!(Action::map_key(key, Mode::Normal), Some(Action::Change));
        assert_eq!(Action::map_key(key, Mode::Select), Some(Action::Change));
    }
}
