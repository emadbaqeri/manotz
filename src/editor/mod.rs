use crate::{
    buffer::{Buffer, GapBuffer},
    command::{
        Edit, Transaction, action_delete, motion_down, motion_left, motion_right, motion_up,
    },
    history::{History, MergeKey},
    input::{Action, Mode},
    render::{Viewport, byte_to_line_col},
    selection::{Selection, SelectionSet},
    text::{grapheme_len, grapheme_to_byte_offset},
    vault::VaultIndex,
};

pub struct EditorState {
    pub buffer: GapBuffer,
    pub viewport: Viewport,
    pub selections: SelectionSet,
    pub mode: Mode,
    pub history: History,
    pub register: Option<String>,
    pub is_dirty: bool,
    pub file_path: Option<std::path::PathBuf>,
    pub vault: Option<VaultIndex>,
}

impl EditorState {
    pub fn new(text: &str, rows: usize, cols: usize) -> Self {
        let buffer = GapBuffer::new(text);
        let viewport = Viewport::new(0, 0, rows, cols);
        let selections = SelectionSet::single(Selection::cursor(0));

        EditorState {
            buffer,
            viewport,
            selections,
            mode: Mode::Normal,
            history: History::new(),
            register: None,
            file_path: None,
            is_dirty: false,
            vault: None,
        }
    }

    pub fn update(self, action: Action) -> Self {
        match action {
            Action::Paste => {
                let text = match &self.register {
                    Some(text) if !text.is_empty() => text.clone(),
                    _ => return self,
                };

                let head = self.selections.primary().head();
                let mut buffer = self.buffer;
                let mut history = self.history;
                let prior_selections = self.selections.clone();

                let new_head = head + text.len();
                let new_selections = SelectionSet::single(Selection::cursor(new_head));

                let tx = Transaction::new(
                    vec![Edit::new(head, head, "", &text)],
                    new_selections.clone(),
                );

                history.record(tx, &mut buffer, prior_selections, MergeKey::Other);

                let buf_text = buffer.slice(0, buffer.len());
                let (line, col) = byte_to_line_col(buf_text, new_head).unwrap_or((0, 0));

                let right_edge = self.viewport.left() + self.viewport.cols();
                let new_left = if col >= right_edge {
                    col - self.viewport.cols() + 1
                } else {
                    self.viewport.left()
                };

                let bottom_edge = self.viewport.top() + self.viewport.rows();
                let new_top = if line >= bottom_edge {
                    line - self.viewport.rows() + 1
                } else {
                    self.viewport.top()
                };

                let new_viewport = Viewport::new(
                    new_top,
                    new_left,
                    self.viewport.rows(),
                    self.viewport.cols(),
                );

                EditorState {
                    buffer,
                    viewport: new_viewport,
                    selections: new_selections,
                    mode: Mode::Normal,
                    history,
                    register: self.register,
                    file_path: self.file_path,
                    is_dirty: true,
                    vault: self.vault,
                }
            }
            Action::Yank => {
                let primary = self.selections.primary();
                let (start, end) = if primary.is_empty() {
                    (primary.head(), (primary.head() + 1).min(self.buffer.len()))
                } else {
                    (
                        primary.anchor().min(primary.head()),
                        primary.anchor().max(primary.head()),
                    )
                };

                let yanked = self.buffer.slice(start, end).to_string();
                let new_selections = SelectionSet::single(Selection::cursor(end));
                EditorState {
                    buffer: self.buffer,
                    viewport: self.viewport,
                    selections: new_selections,
                    mode: Mode::Normal,
                    history: self.history,
                    register: Some(yanked),
                    file_path: self.file_path,
                    is_dirty: self.is_dirty,
                    vault: self.vault,
                }
            }
            Action::Change => {
                let mut buffer = self.buffer;
                let mut history = self.history;
                let prior_selections = self.selections.clone();

                let tx = action_delete(&prior_selections, &buffer);
                let new_selections = tx.new_selections().clone();
                history.record(tx, &mut buffer, prior_selections, MergeKey::Other);

                let head = new_selections.primary().head();
                let text = buffer.slice(0, buffer.len());
                let (line, col) = byte_to_line_col(text, head).unwrap_or((0, 0));

                let new_left = if col < self.viewport.left() {
                    col
                } else {
                    self.viewport.left()
                };

                let new_top = if line < self.viewport.top() {
                    line
                } else {
                    self.viewport.top()
                };

                let new_viewport = Viewport::new(
                    new_top,
                    new_left,
                    self.viewport.rows(),
                    self.viewport.cols(),
                );

                EditorState {
                    buffer,
                    viewport: new_viewport,
                    selections: new_selections,
                    mode: Mode::Insert,
                    history,
                    register: self.register,
                    file_path: self.file_path,
                    is_dirty: true,
                    vault: self.vault,
                }
            }
            Action::Delete => {
                let mut buffer = self.buffer;
                let mut history = self.history;
                let prior_selections = self.selections.clone();

                let tx = action_delete(&prior_selections, &buffer);
                let new_selections = tx.new_selections().clone();
                history.record(tx, &mut buffer, prior_selections, MergeKey::Other);

                let head = new_selections.primary().head();
                let text = buffer.slice(0, buffer.len());
                let (line, col) = byte_to_line_col(text, head).unwrap_or((0, 0));

                let new_left = if col < self.viewport.left() {
                    col
                } else {
                    self.viewport.left()
                };

                let new_top = if line < self.viewport.top() {
                    line
                } else {
                    self.viewport.top()
                };

                let new_viewport = Viewport::new(
                    new_top,
                    new_left,
                    self.viewport.rows(),
                    self.viewport.cols(),
                );

                EditorState {
                    buffer,
                    viewport: new_viewport,
                    selections: new_selections,
                    mode: Mode::Normal,
                    history,
                    register: self.register,
                    file_path: self.file_path,
                    is_dirty: true,
                    vault: self.vault,
                }
            }
            Action::Redo => {
                let mut buffer = self.buffer;
                let mut history = self.history;

                match history.redo(&mut buffer) {
                    Some(selections) => {
                        let head = selections.primary().head();
                        let text = buffer.slice(0, buffer.len());
                        let (line, col) = byte_to_line_col(text, head).unwrap();

                        let right_edge = self.viewport.left() + self.viewport.cols();

                        let new_left = if col < self.viewport.left() {
                            col
                        } else if col >= right_edge {
                            col - self.viewport.cols() + 1
                        } else {
                            self.viewport.left()
                        };

                        let bottom_edge = self.viewport.top() + self.viewport.rows();

                        let new_top = if line < self.viewport.top() {
                            line
                        } else if line >= bottom_edge {
                            line - self.viewport.rows() + 1
                        } else {
                            self.viewport.top()
                        };

                        let new_viewport = Viewport::new(
                            new_top,
                            new_left,
                            self.viewport.rows(),
                            self.viewport.cols(),
                        );

                        EditorState {
                            buffer,
                            viewport: new_viewport,
                            selections,
                            mode: self.mode,
                            history,
                            register: self.register,
                            file_path: self.file_path,
                            is_dirty: true,
                            vault: self.vault,
                        }
                    }
                    None => EditorState {
                        buffer,
                        viewport: self.viewport,
                        selections: self.selections,
                        mode: self.mode,
                        history,
                        register: None,
                        file_path: self.file_path,
                        is_dirty: true,
                        vault: self.vault,
                    },
                }
            }
            Action::Undo => {
                let mut buffer = self.buffer;
                let mut history = self.history;

                match history.undo(&mut buffer) {
                    Some(selections) => {
                        let head = selections.primary().head();
                        let text = buffer.slice(0, buffer.len());
                        let (line, col) = byte_to_line_col(text, head).unwrap();

                        let right_edge = self.viewport.left() + self.viewport.cols();
                        let new_left = if col < self.viewport.left() {
                            col
                        } else if col >= right_edge {
                            col - self.viewport.cols() + 1
                        } else {
                            self.viewport.left()
                        };

                        let bottom_edge = self.viewport.top() + self.viewport.rows();
                        let new_top = if line < self.viewport.top() {
                            line
                        } else if line >= bottom_edge {
                            line - self.viewport.rows() + 1
                        } else {
                            self.viewport.top()
                        };

                        let new_viewport = Viewport::new(
                            new_top,
                            new_left,
                            self.viewport.rows(),
                            self.viewport.cols(),
                        );

                        EditorState {
                            buffer,
                            viewport: new_viewport,
                            selections,
                            mode: self.mode,
                            history,
                            register: self.register,
                            file_path: self.file_path,
                            is_dirty: true,
                            vault: self.vault,
                        }
                    }
                    None => EditorState {
                        buffer,
                        viewport: self.viewport,
                        selections: self.selections,
                        mode: self.mode,
                        history,
                        register: None,
                        file_path: self.file_path,
                        is_dirty: self.is_dirty,
                        vault: self.vault,
                    },
                }
            }
            Action::Backspace => {
                let head = self.selections.primary().head();

                if head == 0 {
                    return self;
                }

                let mut buffer = self.buffer;
                let mut history = self.history;
                let prior_selections = self.selections.clone();

                let prefix = buffer.slice(0, head);
                let start = grapheme_to_byte_offset(prefix, grapheme_len(prefix) - 1);
                let deleted = buffer.slice(start, head);

                let new_selections = SelectionSet::single(Selection::cursor(start));
                let tx = Transaction::new(
                    vec![Edit::new(start, head, deleted, "")],
                    new_selections.clone(),
                );
                history.record(tx, &mut buffer, prior_selections, MergeKey::Other);

                let text = buffer.slice(0, buffer.len());
                let (line, col) = byte_to_line_col(text, start).unwrap();
                let new_left = if col < self.viewport.left() {
                    col
                } else {
                    self.viewport.left()
                };
                let new_top = if line < self.viewport.top() {
                    line
                } else {
                    self.viewport.top()
                };
                let new_viewport = Viewport::new(
                    new_top,
                    new_left,
                    self.viewport.rows(),
                    self.viewport.cols(),
                );

                EditorState {
                    buffer,
                    viewport: new_viewport,
                    selections: new_selections,
                    mode: self.mode,
                    history,
                    register: self.register,
                    file_path: self.file_path,
                    is_dirty: true,
                    vault: self.vault,
                }
            }
            Action::InsertChar(ch) => {
                let head = self.selections.primary().head();
                let mut buffer = self.buffer;
                let mut history = self.history;
                let prior_selections = self.selections.clone();

                let new_head = head + ch.len_utf8();
                let new_selections = SelectionSet::single(Selection::cursor(new_head));

                let tx = Transaction::new(
                    vec![Edit::new(head, head, "", &ch.to_string())],
                    new_selections.clone(),
                );
                history.record(tx, &mut buffer, prior_selections, MergeKey::Insert);

                let text = buffer.slice(0, buffer.len());
                let (line, col) = byte_to_line_col(text, new_head).unwrap();
                let right_edge = self.viewport.left() + self.viewport.cols();
                let new_left = if col >= right_edge {
                    col - self.viewport.cols() + 1
                } else {
                    self.viewport.left()
                };
                let bottom_edge = self.viewport.top() + self.viewport.rows();
                let new_top = if line >= bottom_edge {
                    line - self.viewport.rows() + 1
                } else {
                    self.viewport.top()
                };
                let new_viewport = Viewport::new(
                    new_top,
                    new_left,
                    self.viewport.rows(),
                    self.viewport.cols(),
                );

                EditorState {
                    buffer,
                    viewport: new_viewport,
                    selections: new_selections,
                    mode: self.mode,
                    history,
                    register: self.register,
                    file_path: self.file_path,
                    is_dirty: true,
                    vault: self.vault,
                }
            }
            Action::MoveLeft => {
                let new_selections = motion_left(&self.selections, self.mode);
                let head = new_selections.primary().head();
                let text = self.buffer.slice(0, self.buffer.len());
                let (_, col) = byte_to_line_col(text, head).unwrap();
                let new_left = if col < self.viewport.left() {
                    col
                } else {
                    self.viewport.left()
                };

                let new_viewport = Viewport::new(
                    self.viewport.top(),
                    new_left,
                    self.viewport.rows(),
                    self.viewport.cols(),
                );

                EditorState {
                    buffer: self.buffer,
                    viewport: new_viewport,
                    selections: new_selections,
                    mode: self.mode,
                    history: self.history,
                    register: self.register,
                    file_path: self.file_path,
                    is_dirty: self.is_dirty,
                    vault: self.vault,
                }
            }
            Action::MoveRight => {
                let new_selections = motion_right(&self.selections, self.buffer.len(), self.mode);
                let head = new_selections.primary().head();
                let text = self.buffer.slice(0, self.buffer.len());
                let (_, col) = byte_to_line_col(text, head).unwrap();
                let right_edge = self.viewport.left() + self.viewport.cols();

                let new_left = if col >= right_edge {
                    col - self.viewport.cols() + 1
                } else {
                    self.viewport.left()
                };

                let new_viewport = Viewport::new(
                    self.viewport.top(),
                    new_left,
                    self.viewport.rows(),
                    self.viewport.cols(),
                );

                EditorState {
                    buffer: self.buffer,
                    viewport: new_viewport,
                    selections: new_selections,
                    mode: self.mode,
                    history: self.history,
                    register: self.register,
                    file_path: self.file_path,
                    is_dirty: self.is_dirty,
                    vault: self.vault,
                }
            }
            Action::MoveUp => {
                let text = self.buffer.slice(0, self.buffer.len());
                let new_selections = motion_up(&self.selections, text, self.mode);
                let head = new_selections.primary().head();
                let (line, _) = byte_to_line_col(text, head).unwrap();
                let new_top = if line < self.viewport.top() {
                    line
                } else {
                    self.viewport.top()
                };

                let new_viewport = Viewport::new(
                    new_top,
                    self.viewport.left(),
                    self.viewport.rows(),
                    self.viewport.cols(),
                );

                EditorState {
                    buffer: self.buffer,
                    viewport: new_viewport,
                    selections: new_selections,
                    mode: self.mode,
                    history: self.history,
                    register: self.register,
                    file_path: self.file_path,
                    is_dirty: self.is_dirty,
                    vault: self.vault,
                }
            }
            Action::MoveDown => {
                let text = self.buffer.slice(0, self.buffer.len());
                let new_selections = motion_down(&self.selections, text, self.mode);
                let head = new_selections.primary().head();
                let (line, _) = byte_to_line_col(text, head).unwrap();
                let bottom_edge = self.viewport.top() + self.viewport.rows();
                let new_top = if line >= bottom_edge {
                    line - self.viewport.rows() + 1
                } else {
                    self.viewport.top()
                };

                let new_viewport = Viewport::new(
                    new_top,
                    self.viewport.left(),
                    self.viewport.rows(),
                    self.viewport.cols(),
                );

                EditorState {
                    buffer: self.buffer,
                    viewport: new_viewport,
                    selections: new_selections,
                    mode: self.mode,
                    history: self.history,
                    register: self.register,
                    file_path: self.file_path,
                    is_dirty: self.is_dirty,
                    vault: self.vault,
                }
            }
            Action::EnterInsert => EditorState {
                buffer: self.buffer,
                viewport: self.viewport,
                selections: self.selections,
                mode: Mode::Insert,
                history: self.history,
                register: self.register,
                file_path: self.file_path,
                is_dirty: self.is_dirty,
                vault: self.vault,
            },
            Action::EnterNormal => EditorState {
                buffer: self.buffer,
                viewport: self.viewport,
                selections: self.selections,
                mode: Mode::Normal,
                history: self.history,
                register: self.register,
                file_path: self.file_path,
                is_dirty: self.is_dirty,
                vault: self.vault,
            },
            Action::EnterSelect => EditorState {
                buffer: self.buffer,
                viewport: self.viewport,
                selections: self.selections,
                mode: Mode::Select,
                history: self.history,
                register: self.register,
                file_path: self.file_path,
                is_dirty: self.is_dirty,
                vault: self.vault,
            },
            Action::Quit => self,
        }
    }

    pub fn open_file(
        file_path: &std::path::Path,
        rows: usize,
        cols: usize,
    ) -> std::io::Result<EditorState> {
        let content = std::fs::read_to_string(file_path)?;
        let mut state = EditorState::new(&content, rows, cols);
        state.file_path = Some(file_path.to_path_buf());
        state.is_dirty = false;
        Ok(state)
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        match &self.file_path {
            Some(path) => {
                let content = self.buffer.slice(0, self.buffer.len());
                std::fs::write(path, content)?;
                self.is_dirty = false;

                Ok(())
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file path associated with this buffer",
            )),
        }
    }

    pub fn open_or_create(
        file_path: &std::path::Path,
        rows: usize,
        cols: usize,
    ) -> std::io::Result<EditorState> {
        if file_path.exists() {
            Self::open_file(file_path, rows, cols)
        } else {
            let mut state = Self::new("", rows, cols);
            state.file_path = Some(file_path.to_path_buf());
            state.is_dirty = false;
            Ok(state)
        }
    }

    pub fn restore_cursor(&mut self, offset: usize) {
        let len = self.buffer.len();
        let safe = offset.min(len);
        self.selections = SelectionSet::single(Selection::cursor(safe));
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn update_move_left_at_start_clamps() {
        let (rows, cols) = (5, 5);
        let state = EditorState::new("ABC", rows, cols);
        let next = state.update(Action::MoveLeft);

        assert_eq!(next.selections.primary().head(), 0);
    }

    #[test]
    fn update_move_left_decrements() {
        let (rows, cols) = (5, 5);
        let state = EditorState::new("ABC", rows, cols);
        let next = state.update(Action::MoveLeft);

        assert_eq!(next.selections.primary().head(), 0);
    }

    #[test]
    fn update_move_right_increments() {
        let (rows, cols) = (5, 5);
        let state = EditorState::new("ABC", rows, cols);
        let next = state.update(Action::MoveRight);

        assert_eq!(next.selections.primary().head(), 1);
    }

    #[test]
    fn update_insert_at_end_appends_after_last_char() {
        let mut state = EditorState::new("AB", 5, 5);
        state = state.update(Action::MoveRight); // on 'B'
        state = state.update(Action::MoveRight); // past end
        state = state.update(Action::EnterInsert);
        let next = state.update(Action::InsertChar('X'));

        assert_eq!(next.buffer.slice(0, next.buffer.len()), "ABX");
        assert_eq!(next.selections.primary().head(), 3);
    }

    #[test]
    fn update_quit_returns_same_state() {
        let (rows, cols) = (5, 5);
        let state = EditorState::new("ABC", rows, cols);
        let next = state.update(Action::Quit);

        assert_eq!(next.selections.primary().head(), 0);
    }

    #[test]
    fn update_move_right_scrolls_viewport_when_past_right_edge() {
        let (rows, cols) = (3, 5);
        let mut state = EditorState::new("AAAAABBBBBCCCCCDDDDD", rows, cols);
        for _ in 0..5 {
            state = state.update(Action::MoveRight);
        }
        assert!(state.viewport.left() > 0);
    }

    #[test]
    fn update_move_left_scrolls_viewport_when_past_left_edge() {
        let (rows, cols) = (3, 5);
        let mut state = EditorState::new("AAAAABBBBBCCCCCDDDDD", rows, cols);
        for _ in 0..5 {
            state = state.update(Action::MoveRight);
        }
        assert!(state.viewport.left() > 0);
        for _ in 0..5 {
            state = state.update(Action::MoveLeft);
        }
        assert_eq!(state.viewport.left(), 0);
    }

    #[test]
    fn update_move_down_moves_to_next_line() {
        let text = "AB\nCD";
        let (rows, cols) = (5, 5);
        let state = EditorState::new(text, rows, cols);
        let next = state.update(Action::MoveDown);
        assert_eq!(next.selections.primary().head(), 3);
    }

    #[test]
    fn update_move_up_moves_to_previous_line() {
        let text = "AB\nCD";
        let (rows, cols) = (5, 5);
        let state = EditorState::new(text, rows, cols);

        let state = state.update(Action::MoveDown);
        let next = state.update(Action::MoveUp);
        assert_eq!(next.selections.primary().head(), 0);
    }

    #[test]
    fn update_move_down_scrolls_viewport_when_past_bottom_edge() {
        let (rows, cols) = (2, 5);
        let text = "AA\nBB\nCC\nDD";
        let state = EditorState::new(text, rows, cols);
        let state = state.update(Action::MoveDown);
        let state = state.update(Action::MoveDown);

        assert!(state.viewport.top() > 0);
    }

    #[test]
    fn update_move_up_scrolls_viewport_when_past_top_edge() {
        let (rows, cols) = (2, 5);
        let text = "AA\nBB\nCC\nDD";
        let state = EditorState::new(text, rows, cols);

        let state = state.update(Action::MoveDown);
        let state = state.update(Action::MoveDown);
        let state = state.update(Action::MoveUp);
        let state = state.update(Action::MoveUp);

        assert_eq!(state.viewport.top(), 0);
    }

    #[test]
    fn update_insert_char_at_cursor() {
        let state = EditorState::new("ABC", 5, 5);
        let next = state.update(Action::InsertChar('X'));
        assert_eq!(next.buffer.slice(0, next.buffer.len()), "XABC");
        assert_eq!(next.selections.primary().head(), 1);
    }

    #[test]
    fn update_insert_char_preserves_insert_mode() {
        let state = EditorState::new("ABC", 5, 5);
        let state = state.update(Action::EnterInsert);
        let next = state.update(Action::InsertChar('X'));
        assert_eq!(next.mode, Mode::Insert);
        assert_eq!(next.buffer.slice(0, next.buffer.len()), "XABC");
        assert_eq!(next.selections.primary().head(), 1);
    }

    #[test]
    fn update_move_right_preserves_insert_mode() {
        let state = EditorState::new("ABC", 5, 5);
        let state = state.update(Action::EnterInsert);
        let next = state.update(Action::MoveRight);
        assert_eq!(next.mode, Mode::Insert);
        assert_eq!(next.selections.primary().head(), 1);
    }

    #[test]
    fn update_backspace_at_start_is_noop() {
        let state = EditorState::new("ABC", 5, 5);
        let state = state.update(Action::EnterInsert);
        let next = state.update(Action::Backspace);
        assert_eq!(next.buffer.slice(0, next.buffer.len()), "ABC");
        assert_eq!(next.selections.primary().head(), 0);
        assert_eq!(next.mode, Mode::Insert);
    }

    #[test]
    fn update_insert_newline_splits_line() {
        let state = EditorState::new("AB", 5, 5);
        let state = state.update(Action::MoveRight);
        let state = state.update(Action::EnterInsert);
        let next = state.update(Action::InsertChar('\n'));

        assert_eq!(next.buffer.slice(0, next.buffer.len()), "A\nB");
        assert_eq!(next.selections.primary().head(), 2);
        assert_eq!(next.mode, Mode::Insert);
    }

    #[test]
    fn update_insert_char_scrolls_viewport_when_past_right_edge() {
        let mut state = EditorState::new("", 3, 5);
        state = state.update(Action::EnterInsert);
        for _ in 0..5 {
            state = state.update(Action::InsertChar('A'));
        }

        assert_eq!(state.viewport.left(), 1);
    }

    #[test]
    fn update_backspace_scrolls_viewport_when_past_left_edge() {
        let mut state = EditorState::new("", 3, 5);
        state = state.update(Action::EnterInsert);
        for _ in 0..5 {
            state = state.update(Action::InsertChar('A'));
        }
        assert!(state.viewport.left() > 0);

        for _ in 0..5 {
            state = state.update(Action::Backspace);
        }
        assert_eq!(state.viewport.left(), 0);
    }

    #[test]
    fn update_insert_newline_scrolls_viewport_when_past_bottom_edge() {
        let mut state = EditorState::new("", 2, 5);
        state = state.update(Action::EnterInsert);
        state = state.update(Action::InsertChar('\n'));
        state = state.update(Action::InsertChar('\n'));
        assert!(state.viewport.top() > 0);
    }

    #[test]
    fn update_undo_after_insert_restores_buffer_and_head() {
        let state = EditorState::new("AB", 5, 5);
        let state = state.update(Action::InsertChar('X'));
        let next = state.update(Action::Undo);

        assert_eq!(next.buffer.slice(0, next.buffer.len()), "AB");
        assert_eq!(next.selections.primary().head(), 0);
    }

    #[test]
    fn update_undo_after_backspace_restores_buffer_and_head() {
        let state = EditorState::new("AB", 5, 5);
        let state = state.update(Action::MoveRight);
        let state = state.update(Action::EnterInsert);
        let state = state.update(Action::Backspace);
        let next = state.update(Action::Undo);

        assert_eq!(next.buffer.slice(0, next.buffer.len()), "AB");
        assert_eq!(next.selections.primary().head(), 1);
    }

    #[test]
    fn update_consecutive_inserts_undo_as_one_step() {
        let mut state = EditorState::new("", 3, 5);
        state = state.update(Action::EnterInsert);
        state = state.update(Action::InsertChar('a'));
        state = state.update(Action::InsertChar('b'));
        state = state.update(Action::InsertChar('c'));
        assert_eq!(state.buffer.slice(0, state.buffer.len()), "abc");

        let next = state.update(Action::Undo);
        assert_eq!(next.buffer.slice(0, next.buffer.len()), "");
        assert_eq!(next.selections.primary().head(), 0);
    }

    #[test]
    fn update_undo_with_empty_history_is_noop() {
        let state = EditorState::new("AB", 5, 5);
        let next = state.update(Action::Undo);

        assert_eq!(next.buffer.slice(0, next.buffer.len()), "AB");
        assert_eq!(next.selections.primary().head(), 0);
    }

    #[test]
    fn redo_restores_buffer_and_head_after_undo() {
        let state = EditorState::new("AB", 5, 5);
        let state = state.update(Action::InsertChar('X'));
        let state = state.update(Action::Undo);
        let next = state.update(Action::Redo);
        assert_eq!(next.buffer.slice(0, next.buffer.len()), "XAB");
        assert_eq!(next.selections.primary().head(), 1);
    }

    #[test]
    fn update_enter_select_sets_select_mode() {
        let state = EditorState::new("ABC", 5, 5);
        let next = state.update(Action::EnterSelect);
        assert_eq!(next.mode, Mode::Select);
    }

    #[test]
    fn update_move_right_in_select_mode_extends_selection() {
        let state = EditorState::new("ABCDE", 5, 5);
        let state = state.update(Action::EnterSelect);
        let next = state.update(Action::MoveRight);

        let primary = next.selections.primary();
        assert_eq!(primary.anchor(), 0);
        assert_eq!(primary.head(), 1);
        assert_eq!(next.mode, Mode::Select);
    }

    #[test]
    fn update_delete_in_select_mode_deletes_range_and_returns_to_normal() {
        let state = EditorState::new("hello world", 5, 5);
        let state = state.update(Action::EnterSelect);
        let state = state.update(Action::MoveRight);
        let state = state.update(Action::MoveRight);
        let next = state.update(Action::Delete);

        assert_eq!(next.buffer.slice(0, next.buffer.len()), "llo world");
        assert_eq!(next.mode, Mode::Normal);
        assert_eq!(next.selections.primary().head(), 0);
    }

    #[test]
    fn update_change_in_select_mod_deletes_range_and_enters_insert() {
        let state = EditorState::new("hello world", 5, 5);
        let state = state.update(Action::EnterSelect);
        let state = state.update(Action::MoveRight);
        let state = state.update(Action::MoveRight);
        let next = state.update(Action::Change);

        assert_eq!(next.buffer.slice(0, next.buffer.len()), "llo world");
        assert_eq!(next.mode, Mode::Insert);
        assert_eq!(next.selections.primary().head(), 0);
    }

    #[test]
    fn update_yank_copies_selected_text_to_register() {
        let state = EditorState::new("hello world", 5, 5);
        let mut state = state.update(Action::EnterSelect);
        for _ in 0..5 {
            state = state.update(Action::MoveRight);
        }

        let next = state.update(Action::Yank);

        assert_eq!(next.register.as_deref(), Some("hello"));
        assert_eq!(next.mode, Mode::Normal);
        assert_eq!(next.buffer.slice(0, next.buffer.len()), "hello world");
    }

    #[test]
    fn update_paste_inserts_register_text_and_records_in_history() {
        let state = EditorState::new(" world", 5, 5);
        let mut state = state;
        state.register = Some("hello".to_string());

        let next = state.update(Action::Paste);

        assert_eq!(next.buffer.slice(0, next.buffer.len()), "hello world");
        assert_eq!(next.selections.primary().head(), 5);
        assert_eq!(next.mode, Mode::Normal);

        let undone = next.update(Action::Undo);
        assert_eq!(undone.buffer.slice(0, undone.buffer.len()), " world");
    }

    #[test]
    fn update_yank_collapses_selection_to_point_cursor() {
        let state = EditorState::new("hello world", 5, 5);
        let mut state = state.update(Action::EnterSelect);
        for _ in 0..5 {
            state = state.update(Action::MoveRight);
        }
        let next = state.update(Action::Yank);

        let primary = next.selections.primary();
        assert!(primary.is_empty());
        assert_eq!(primary.head(), 5);
    }

    #[test]
    fn register_persists_across_motions_and_mode_changes() {
        let state = EditorState::new("hello world", 5, 5);
        let state = state.update(Action::EnterSelect);
        let state = state.update(Action::MoveRight);
        let state = state.update(Action::Yank);
        let next = state.update(Action::MoveRight);

        assert_eq!(next.register.as_deref(), Some("h"));
    }

    #[test]
    fn open_file_loads_content_and_sets_not_dirty() {
        use std::{
            io::Write,
            sync::atomic::{AtomicU64, Ordering},
        };
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir();
        let file_path = dir.join(format!(
            "manotz_test_open_file_{}_{n}.md",
            std::process::id()
        ));
        {
            let mut file = std::fs::File::create(&file_path).unwrap();
            write!(file, "Hello from disk!").unwrap();
        }

        let state = EditorState::open_file(&file_path, 5, 5).unwrap();

        assert_eq!(
            state.buffer.slice(0, state.buffer.len()),
            "Hello from disk!"
        );
        assert_eq!(state.file_path, Some(file_path.clone()));
        assert!(!state.is_dirty);

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn edit_sets_is_dirty_true() {
        use std::{
            io::Write,
            sync::atomic::{AtomicU64, Ordering},
        };
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir();
        let file_path = dir.join(format!(
            "manotz_test_dirty_file_{}_{n}.md",
            std::process::id()
        ));
        {
            let mut file = std::fs::File::create(&file_path).unwrap();
            write!(file, "Hello").unwrap();
        }

        let state = EditorState::open_file(&file_path, 5, 5).unwrap();
        assert!(!state.is_dirty); // initially clean

        let next = state.update(Action::InsertChar('!'));
        assert!(next.is_dirty); // dirty after edit!

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn save_writes_buffer_to_disk_and_resets_is_dirty() {
        use std::{
            io::Write,
            sync::atomic::{AtomicU64, Ordering},
        };
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir();
        let file_path = dir.join(format!(
            "manotz_test_save_file_{}_{n}.md",
            std::process::id()
        ));
        {
            let mut file = std::fs::File::create(&file_path).unwrap();
            write!(file, "Hello").unwrap();
        }

        let state = EditorState::open_file(&file_path, 5, 5).unwrap();
        let mut state = state.update(Action::InsertChar('!'));
        state.save().unwrap();
        assert!(!state.is_dirty);
        let disk_content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(disk_content, "!Hello");

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn open_or_create_nonexistent_file_returns_file_returns_empty_buffer_with_path() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir();
        let file_path = dir.join(format!(
            "manotz_nonexistent_test_{}_{n}.md",
            std::process::id()
        ));

        let _ = std::fs::remove_file(&file_path);

        let state = EditorState::open_or_create(&file_path, 5, 5).unwrap();

        assert_eq!(state.buffer.slice(0, state.buffer.len()), "");
        assert_eq!(state.file_path, Some(file_path.clone()));
        assert!(!state.is_dirty);

        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn restore_cursor_clamps_to_buffer_len() {
        let mut state = EditorState::new("hi", 5, 5);
        state.restore_cursor(999);
        let head = state.selections.primary().head();
        assert_eq!(head, 2);
    }
}
