use std::io::Write;

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::render::{Cell, Colour, Grid, diff};

pub struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        disable_raw_mode().expect("failed to disable raw mode");
    }
}

pub fn enter_raw_mode() -> RawModeGuard {
    let mut stdout = std::io::stdout();
    queue!(stdout, crossterm::cursor::Hide).unwrap();
    enable_raw_mode().expect("failed to enable raw mode");
    RawModeGuard
}

#[derive(Default)]
pub struct Adapter {
    prev: Option<Grid>,
}

impl Adapter {
    pub fn draw(&mut self, curr: &Grid, stdout: &mut impl Write) -> Result<(), std::io::Error> {
        match &self.prev {
            None => {
                // first rame draw - draw ALL cells
                for row in 0..curr.height() {
                    for col in 0..curr.width() {
                        self.write_cell(stdout, row, col, curr.cell(row, col))?
                    }
                }
            }
            Some(prev) => {
                // later frame - diff and draw only changes
                let changes = diff(prev, curr);
                for (row, col, cell) in changes {
                    self.write_cell(stdout, row, col, cell)?
                }
            }
        }
        stdout.flush()?;
        self.prev = Some(curr.clone());

        Ok(())
    }

    fn write_cell(
        &self,
        stdout: &mut impl Write,
        row: usize,
        col: usize,
        cell: &Cell,
    ) -> Result<(), std::io::Error> {
        queue!(stdout, MoveTo(col as u16, row as u16))?;
        queue!(stdout, SetBackgroundColor(Color::Reset))?;
        queue!(stdout, SetForegroundColor(Color::Reset))?;

        if let Some(crate::render::Colour::Rgb(r, g, b)) = cell.background() {
            queue!(stdout, SetBackgroundColor(Color::Rgb { r, g, b }))?;
        };

        if let Some(Colour::Rgb(r, g, b)) = cell.foreground() {
            queue!(stdout, SetForegroundColor(Color::Rgb { r, g, b }))?;
        };

        queue!(stdout, Print(cell.grapheme()))?;
        Ok(())
    }
}
