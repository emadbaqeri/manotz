use crate::{buffer::Buffer, markdown::{Highlight, style_for}, selection::SelectionSet, text::grapheme_width};
use unicode_segmentation::UnicodeSegmentation;

pub mod adapter;

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum Colour {
    Rgb(u8, u8, u8),
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Style {
    pub bold: bool,
    pub fg: Option<Colour>,
    pub bg: Option<Colour>,
}

impl Style {
    pub fn new(bold: bool, fg: Option<Colour>, bg: Option<Colour>) -> Self {
        Style { bold, fg, bg }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    style: Style,
    grapheme: String,
}

#[derive(Clone)]
pub struct Grid {
    width: usize,
    cells: Vec<Cell>,
}

pub struct Viewport {
    top: usize,
    left: usize,
    rows: usize,
    cols: usize,
}

impl Viewport {
    pub fn new(top: usize, left: usize, rows: usize, cols: usize) -> Viewport {
        if rows == 0 {
            panic!("Invalid `rows` Value! `rows` has to be at least `1`");
        }

        if cols == 0 {
            panic!("Invalid `cols` Value! `cols` has to be at least `1`");
        }

        Viewport {
            top,
            left,
            rows,
            cols,
        }
    }

    pub fn top(&self) -> usize {
        self.top
    }

    pub fn left(&self) -> usize {
        self.left
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Grid {
        if width == 0 {
            panic!("Invalid `width` Value! `width` has to be at least `1`");
        }

        if height == 0 {
            panic!("Invalid `height` Value! `height` has to be at least `1`");
        }
        Grid {
            width,
            cells: vec![Cell::blank(); width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.cells.len() / self.width
    }

    pub fn cell(&self, row: usize, col: usize) -> &Cell {
        let index = self.cell_index(row, col);
        &self.cells[index]
    }

    pub fn set_cell(&mut self, row: usize, col: usize, cell: Cell) {
        let index = self.cell_index(row, col);
        self.cells[index] = cell;
    }

    pub fn set_style(&mut self, row: usize, col: usize, style: Style) {
        let index = self.cell_index(row, col);
        self.cells[index].style = style;
    }

    fn cell_index(&self, row: usize, col: usize) -> usize {
        row * self.width + col
    }
}

impl Cell {
    pub fn new(grapheme: &str) -> Cell {
        Cell {
            style: Style::default(),
            grapheme: grapheme.to_string(),
        }
    }

    pub fn grapheme(&self) -> &str {
        &self.grapheme
    }

    pub fn blank() -> Cell {
        Cell::new(" ")
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    pub fn is_bold(&self) -> bool {
        self.style.bold
    }

    pub fn foreground(&self) -> Option<Colour> {
        self.style.fg
    }

    pub fn background(&self) -> Option<Colour> {
        self.style.bg
    }
}

pub fn render(buff: &impl Buffer, selections: &SelectionSet, viewport: &Viewport, highlights: &[Highlight]) -> Grid {
    let text = buff.slice(0, buff.len());
    let lines = text.lines().collect::<Vec<&str>>();
    let mut grid = Grid::new(viewport.cols(), viewport.rows());

    // Draw text grapheme
    for r in 0..viewport.rows() {
        let doc_line = viewport.top() + r;
        if doc_line >= lines.len() {
            continue;
        }
        let line = lines[doc_line];
        let graphemes = line.graphemes(true).collect::<Vec<&str>>();

        let mut document_col = 0;
        for grapheme in &graphemes {
            let width = grapheme_width(grapheme);
            if document_col >= viewport.left() {
                let viewport_col = document_col - viewport.left();
                if viewport_col >= viewport.cols() {
                    break;
                }
                grid.set_cell(r, viewport_col, Cell::new(grapheme));
            }
            document_col += width
        }
    }

    for h in highlights {
        let style = style_for(h.kind.clone());
        for byte in h.start..h.end {
            if let Some((line,col)) = byte_to_line_col(text, byte) && let (Some(r), Some(c)) = (line.checked_sub(viewport.top()), col.checked_sub(viewport.left())) && r < viewport.rows() && c < viewport.cols() {
                grid.set_style(r, c, style.clone());
            }

        }

    }

    let selection_style = Style {
        bg: Some(Colour::Rgb(40, 40, 80)),
        fg: Some(Colour::Rgb(230, 230, 255)),
        ..Style::default()
    };

    let cursor_style = Style {
        bg: Some(Colour::Rgb(80, 80, 80)),
        fg: Some(Colour::Rgb(160, 160, 160)),
        ..Style::default()
    };

    for selection in selections {
        let start = selection.anchor().min(selection.head());
        let end = selection.anchor().max(selection.head());
        for byte in start..end {
            if let Some((line, col)) = byte_to_line_col(text, byte)
                && let (Some(r), Some(c)) = (
                    line.checked_sub(viewport.top()),
                    col.checked_sub(viewport.left()),
                )
                && r < viewport.rows()
                && c < viewport.cols()
            {
                grid.set_style(r, c, selection_style.clone());
            }
        }

        if let Some((line, col)) = byte_to_line_col(text, selection.head())
            && let (Some(r), Some(c)) = (
                line.checked_sub(viewport.top()),
                col.checked_sub(viewport.left()),
            )
            && r < viewport.rows()
            && c < viewport.cols()
        {
            grid.set_style(r, c, cursor_style.clone());
        }
    }

    grid
}

pub fn byte_to_line_col(text: &str, offset: usize) -> Option<(usize, usize)> {
    if offset > text.len() {
        return None;
    }

    let mut col = 0;
    let mut line = 0;

    for (_, ch) in text
        .char_indices()
        .take_while(|(byte_pos, _)| *byte_pos < offset)
    {
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    Some((line, col))
}

pub fn line_col_to_byte(text: &str, target_line: usize, target_col: usize) -> Option<usize> {
    if text.is_empty() && (target_line, target_col) == (0, 0) {
        return Some(0);
    }

    let mut current_col = 0;
    let mut current_line = 0;

    for (byte_index, ch) in text.char_indices() {
        if current_line == target_line && current_col == target_col {
            return Some(byte_index);
        }

        if ch == '\n' {
            if current_line == target_line {
                return Some(byte_index);
            }
            current_line += 1;
            current_col = 0;
        } else {
            current_col += 1;
        }
    }
    if current_line == target_line && current_col >= target_col {
        return Some(text.len());
    }

    None
}

pub fn diff<'a>(prev: &Grid, curr: &'a Grid) -> Vec<(usize, usize, &'a Cell)> {
    let mut changes = Vec::new();
    let total = prev.width() * prev.height();

    for index in 0..total {
        let row = index / prev.width();
        let col = index % prev.width();

        if prev.cell(row, col) != curr.cell(row, col) {
            changes.push((row, col, curr.cell(row, col)));
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use crate::{
        buffer::GapBuffer,
        selection::{Selection, SelectionSet},
    };

    use super::*;

    #[test]
    fn render_cell_contains_grapheme() {
        assert_eq!(Cell::new("H").grapheme(), "H");
    }

    #[test]
    fn render_cell_blank_is_space() {
        assert_eq!(Cell::blank().grapheme(), " ");
    }

    #[test]
    fn render_grid_blank() {
        let g = Grid::new(4, 2);
        assert_eq!(g.width(), 4);
        assert_eq!(g.height(), 2);
        assert_eq!(g.cell(0, 0).grapheme(), " ");
    }

    #[test]
    fn render_grid_set_cell() {
        let mut g = Grid::new(2, 2);
        g.set_cell(0, 1, Cell::new("X"));
        assert_eq!(g.cell(0, 1).grapheme(), "X");
        assert_eq!(g.cell(0, 0).grapheme(), " ");
    }

    #[test]
    #[should_panic]
    fn render_grid_rejects_zero_width() {
        Grid::new(0, 5);
    }

    #[test]
    fn render_viewport_blank() {
        let vp = Viewport::new(2, 3, 10, 15);
        assert_eq!(vp.top(), 2);
        assert_eq!(vp.left(), 3);
        assert_eq!(vp.rows(), 10);
        assert_eq!(vp.cols(), 15);
    }

    #[test]
    #[should_panic]
    fn render_viewport_fails_with_no_cols() {
        Viewport::new(1, 1, 1, 0);
    }

    #[test]
    #[should_panic]
    fn render_viewport_fails_with_no_rows() {
        Viewport::new(1, 1, 0, 1);
    }

    #[test]
    fn render_line_pads_blanks() {
        let buf = GapBuffer::new("Hi");
        let selections = SelectionSet::single(Selection::cursor(0));
        let viewport = Viewport::new(0, 0, 1, 3);
        let grid = render(&buf, &selections, &viewport, &[]);
        assert_eq!(grid.width(), 3);
        assert_eq!(grid.height(), 1);
        assert_eq!(grid.cell(0, 1).grapheme(), "i");
        assert_eq!(grid.cell(0, 2).grapheme(), " ");
    }

    #[test]
    fn render_scrolls_vertically() {
        let buffer = GapBuffer::new("AB\nCD\nEF");
        let selections = SelectionSet::single(Selection::cursor(0));
        let viewport = Viewport::new(1, 0, 2, 2);
        let grid = render(&buffer, &selections, &viewport, &[]);

        assert_eq!(grid.width(), 2);
        assert_eq!(grid.height(), 2);
        assert_eq!(grid.cell(0, 0).grapheme(), "C");
        assert_eq!(grid.cell(0, 1).grapheme(), "D");
        assert_eq!(grid.cell(1, 0).grapheme(), "E");
        assert_eq!(grid.cell(1, 1).grapheme(), "F");
    }

    #[test]
    fn render_scrolls_horizontally() {
        let buffer = GapBuffer::new("ABCDE");
        let selections = SelectionSet::single(Selection::cursor(0));
        let viewport = Viewport::new(0, 2, 1, 2);
        let grid = render(&buffer, &selections, &viewport, &[]);

        assert_eq!(grid.width(), 2);
        assert_eq!(grid.height(), 1);
        assert_eq!(grid.cell(0, 0).grapheme(), "C");
        assert_eq!(grid.cell(0, 1).grapheme(), "D");
    }

    #[test]
    fn render_short_doc_pads_blank_rows() {
        let buffer = GapBuffer::new("AB\nCD");
        let selections = SelectionSet::single(Selection::cursor(0));
        let viewport = Viewport::new(0, 0, 4, 3);
        let grid = render(&buffer, &selections, &viewport, &[]);
        assert_eq!(grid.cell(2, 0).grapheme(), " ");
        assert_eq!(grid.cell(3, 0).grapheme(), " ");
    }

    #[test]
    fn byte_to_line_col_at_start() {
        assert_eq!(byte_to_line_col("Hi", 0), Some((0, 0)));
    }

    #[test]
    fn byte_to_line_col_after_newline() {
        assert_eq!(byte_to_line_col("AB\nCD", 3), Some((1, 0)))
    }

    #[test]
    fn line_col_to_byte_start_of_second_line() {
        assert_eq!(line_col_to_byte("AB\nCD", 1, 0), Some(3));
    }

    #[test]
    fn grid_set_style() {
        let buffer = GapBuffer::new("Hello");
        let selections = SelectionSet::single(Selection::cursor(0));
        let viewport = Viewport::new(0, 0, 3, 3);
        let mut grid = render(&buffer, &selections, &viewport, &[]);

        let style = Style {
            bold: true,
            fg: Some(Colour::Rgb(1, 1, 1)),
            bg: Some(Colour::Rgb(2, 2, 2)),
        };

        grid.set_style(0, 0, style);

        assert!(grid.cell(0, 0).style.bold);
        assert_eq!(grid.cell(0, 0).style.fg, Some(Colour::Rgb(1, 1, 1)));
        assert_eq!(grid.cell(0, 0).style.bg, Some(Colour::Rgb(2, 2, 2)));
    }

    #[test]
    fn render_shows_cursor() {
        let buffer = GapBuffer::new("Hi");
        let selections = SelectionSet::single(Selection::cursor(0));
        let viewport = Viewport::new(0, 0, 1, 3);
        let grid = render(&buffer, &selections, &viewport, &[]);

        assert!(!grid.cell(0, 0).is_bold());
        assert_eq!(grid.cell(0, 0).background(), Some(Colour::Rgb(80, 80, 80)));
        assert_eq!(grid.cell(0, 1).background(), None);
    }

    #[test]
    fn render_hides_cursor_scrolled_left() {
        let buffer = GapBuffer::new("ABCDE");
        let selections = SelectionSet::single(Selection::cursor(0));
        let viewport = Viewport::new(0, 2, 5, 5);
        let grid = render(&buffer, &selections, &viewport, &[]);

        assert_eq!(grid.cell(0, 0).background(), None);
    }

    #[test]
    fn render_hides_cursor_scrolled_up() {
        let buffer = GapBuffer::new("AB\nCD");
        let selections = SelectionSet::single(Selection::cursor(0));
        let viewport = Viewport::new(1, 0, 5, 5);
        let grid = render(&buffer, &selections, &viewport, &[]);

        assert_eq!(grid.cell(0, 0).background(), None);
    }

    #[test]
    fn render_cursor_on_second_line() {
        let buffer = GapBuffer::new("AB\nCD");
        let selections = SelectionSet::single(Selection::cursor(3));
        let viewport = Viewport::new(0, 0, 5, 5);
        let grid = render(&buffer, &selections, &viewport, &[]);

        assert_eq!(grid.cell(1, 0).background(), Some(Colour::Rgb(80, 80, 80)));
    }

    #[test]
    fn render_wide_char_takes_two_cols() {
        let buffer = GapBuffer::new("中A");
        let selections = SelectionSet::single(Selection::cursor(0));
        let viewport = Viewport::new(0, 0, 5, 5);
        let grid = render(&buffer, &selections, &viewport, &[]);

        assert_eq!(grid.cell(0, 0).grapheme(), "中");
        assert_eq!(grid.cell(0, 1).grapheme(), " ");
        assert_eq!(grid.cell(0, 2).grapheme(), "A");
    }

    #[test]
    fn diff_identical_grid_returns_empty() {
        let a = Grid::new(3, 2);
        let b = Grid::new(3, 2);

        let changes = diff(&a, &b);

        assert!(changes.is_empty())
    }

    #[test]
    fn diff_returns_changed_cell() {
        let a = Grid::new(3, 2);
        let mut b = Grid::new(3, 2);

        b.set_cell(1, 0, Cell::new("X"));

        let changes = diff(&a, &b);

        assert!(changes.len() == 1);
        assert!(changes[0].0 == 1);
        assert!(changes[0].1 == 0);
        assert_eq!(changes[0].2.grapheme(), "X");
    }

    #[test]
    fn render_selection_range_styles_range_cells() {
        let buffer = GapBuffer::new("ABCDE");
        // Selection from index 1 ('B') to 3 ('D')
        let selections = SelectionSet::single(Selection::new(1, 3));
        let viewport = Viewport::new(0, 0, 1, 5);
        let grid = render(&buffer, &selections, &viewport, &[]);

        // cell 0 ('A') has default bg
        assert_eq!(grid.cell(0, 0).background(), None);

        // cells 1 and 2 ('B', 'C') have selection range bg and fg
        assert_eq!(grid.cell(0, 1).background(), Some(Colour::Rgb(40, 40, 80)));
        assert_eq!(
            grid.cell(0, 1).foreground(),
            Some(Colour::Rgb(230, 230, 255))
        );
        assert_eq!(grid.cell(0, 2).background(), Some(Colour::Rgb(40, 40, 80)));
        assert_eq!(
            grid.cell(0, 2).foreground(),
            Some(Colour::Rgb(230, 230, 255))
        );

        // cell 3 ('D', cursor head) has cursor bg
        assert_eq!(grid.cell(0, 3).background(), Some(Colour::Rgb(80, 80, 80)));
    }

    #[test]
    fn render_applies_markdown_heading_style() {
        use crate::markdown::{highlight, style_for, HighlightKind};

        let buffer = GapBuffer::new("# Heading");
        let selections = SelectionSet::single(Selection::cursor(0));
        let viewport = Viewport::new(0, 0, 1, 10);
        let highlights = highlight("# Heading");

        let grid = render(&buffer, &selections, &viewport, &highlights);

        let expected_style = style_for(HighlightKind::Heading);

        assert_eq!(grid.cell(0, 1).style(), &expected_style);
    }
}
