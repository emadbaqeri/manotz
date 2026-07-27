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
    let _guard = enter_raw_mode();
    let mut state = {
        let (cols, rows) = crossterm::terminal::size().unwrap();
        let rows = (rows as usize).max(1);
        let cols = (cols as usize).max(1);
        EditorState::new("Hello, manotz!", rows, cols)
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
            break;
        }

        state = state.update(action);
    }

    Ok(())
}
