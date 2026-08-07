use std::io;

use crossterm::event::{self, KeyEventKind};
use manotz::{
    editor::EditorState,
    input::Action,
    render::{
        adapter::{Adapter, enter_raw_mode},
        render,
    },
};

fn main() -> io::Result<()> {
    let state_path = std::path::PathBuf::from(".manotz/state.json");
    let mut session = manotz::session::SessionState::load(&state_path);

    let arg_path = std::env::args().nth(1).map(std::path::PathBuf::from);

    let _guard = enter_raw_mode();

    let target_path = arg_path.or(session.last_open_file.clone());

    let mut state = {
        let (cols, rows) = crossterm::terminal::size().unwrap();
        let rows = (rows as usize).max(1);
        let cols = (cols as usize).max(1);

        if let Some(path) = target_path {
            let mut state = EditorState::open_or_create(&path, rows, cols)?;
            if let Some(&saved) = session.cursors.get(&path) {
                state.restore_cursor(saved);
            }
            state
        } else {
            EditorState::new("Hello, manotz!", rows, cols)
        }
    };

    let mut adapter = Adapter::default();
    let mut stdout = std::io::stdout();

    loop {
        let grid = render(&state.buffer, &state.selections, &state.viewport);
        adapter.draw(&grid, &mut stdout)?;
        let event = event::read()?;
        let key = match event {
            crossterm::event::Event::Key(k) => k,
            _ => continue,
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        let action = match Action::map_key(key, state.mode) {
            Some(a) => a,
            None => continue,
        };

        if action == Action::Quit {
            if let Some(path) = &state.file_path {
                let cursor_pos = state.selections.primary().head();
                session.update_cursor(path.clone(), cursor_pos);
            }

            let _ = session.save(&state_path);

            break;
        }

        state = state.update(action);
    }

    Ok(())
}
