use crate::board::Board;
use crossterm::{
    cursor, execute,
    style::Stylize,
    terminal::{self, Clear, ClearType},
};
use std::{
    collections::HashMap,
    io::{self, stdout},
};

pub struct Ui {
    messages: HashMap<(u16, u16), fn()>,
}

const CELL_WIDTH: usize = 7;
const CELL_HEIGHT: usize = 3;

impl Ui {
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), Clear(ClearType::All), cursor::Hide)?;

        Ok(Self {
            messages: HashMap::default(),
        })
    }

    pub fn draw(&self, b: &Board) -> io::Result<()> {
        draw_board(b)?;

        for ((x, y), f) in &self.messages {
            execute!(stdout(), cursor::MoveTo(*x, *y))?;
            f()
        }

        Ok(())
    }

    pub fn set_cursor_onboard(&self, pos: Option<(usize, usize)>) -> io::Result<()> {
        match pos {
            Some((y, x)) => {
                let x = (CELL_WIDTH + 1) * x + CELL_WIDTH / 2 + 1;
                let y = (CELL_HEIGHT + 1) * y + CELL_HEIGHT / 2 + 1;
                execute!(
                    stdout(),
                    cursor::MoveTo(x as u16, y as u16),
                    cursor::SetCursorStyle::BlinkingUnderScore,
                    cursor::Show,
                )
            }
            None => execute!(stdout(), cursor::Hide),
        }
    }

    pub fn has(&self, at: (u16, u16)) -> bool {
        self.messages.contains_key(&at)
    }

    pub fn add_msg(&mut self, at: (u16, u16), f: fn()) {
        self.messages.insert(at, f);
    }

    pub fn remove_msg(&mut self, at: (u16, u16)) {
        self.messages.remove(&at);
    }
}

impl Drop for Ui {
    fn drop(&mut self) {
        execute!(
            stdout(),
            cursor::MoveTo(0, 0),
            cursor::SetCursorStyle::DefaultUserShape,
            cursor::Show
        )
        .unwrap();
        terminal::disable_raw_mode().unwrap();
    }
}

pub fn draw_board(b: &Board) -> io::Result<()> {
    execute!(stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    let bold_vertical = |i| (i + 1) % 3 == 0;
    horiz_border(BorderKind::Down, 9, CELL_WIDTH, bold_vertical, true);
    for i in 0..9 {
        for j in 0..CELL_HEIGHT {
            print!("{}", "║".grey().dim());

            for k in 0..9 {
                let c = &b[(i, k)];
                let sep = if bold_vertical(k) {
                    "║".grey().dim()
                } else {
                    "│".grey().dim()
                };

                if c.is_maybe() {
                    let num = |n: usize| {
                        if c.maybe_get_num(n as u8) {
                            n.to_string()
                        } else {
                            " ".into()
                        }
                    };

                    print!(
                        " {} {} {} {}",
                        num(j * 3 + 1),
                        num(j * 3 + 2),
                        num(j * 3 + 3),
                        sep
                    )
                } else {
                    print!(
                        "   {}   {}",
                        if j == (CELL_HEIGHT / 2) {
                            b[(i, k)].to_string()
                        } else {
                            " ".into()
                        },
                        sep,
                    );
                }
            }
            print!("\r\n");
        }

        if i != 8 {
            horiz_border(
                BorderKind::Both,
                9,
                CELL_WIDTH,
                bold_vertical,
                bold_vertical(i),
            );
        }
    }

    horiz_border(BorderKind::Up, 9, CELL_WIDTH, bold_vertical, true);

    Ok(())
}

enum BorderKind {
    Down,
    Up,
    Both,
}

impl BorderKind {
    fn left_corner(&self, bold_h: bool, bold_v: bool) -> &str {
        match (self, bold_h, bold_v) {
            (BorderKind::Down, false, false) => "┌",
            (BorderKind::Down, false, true) => "╓",
            (BorderKind::Down, true, false) => "╒",
            (BorderKind::Down, true, true) => "╔",
            (BorderKind::Up, false, false) => "└",
            (BorderKind::Up, false, true) => "╙",
            (BorderKind::Up, true, false) => "╘",
            (BorderKind::Up, true, true) => "╚",
            (BorderKind::Both, false, false) => "├",
            (BorderKind::Both, false, true) => "╟",
            (BorderKind::Both, true, false) => "╞",
            (BorderKind::Both, true, true) => "╠",
        }
    }

    fn right_corner(&self, bold_h: bool, bold_v: bool) -> &str {
        match (self, bold_h, bold_v) {
            (BorderKind::Down, false, false) => "┐",
            (BorderKind::Down, false, true) => "╖",
            (BorderKind::Down, true, false) => "╕",
            (BorderKind::Down, true, true) => "╗",
            (BorderKind::Up, false, false) => "┘",
            (BorderKind::Up, false, true) => "╜",
            (BorderKind::Up, true, false) => "╛",
            (BorderKind::Up, true, true) => "╝",
            (BorderKind::Both, false, false) => "┤",
            (BorderKind::Both, false, true) => "╢",
            (BorderKind::Both, true, false) => "╡",
            (BorderKind::Both, true, true) => "╣",
        }
    }

    fn middle(&self, bold_h: bool, bold_v: bool) -> &str {
        match (self, bold_h, bold_v) {
            (BorderKind::Down, false, false) => "┬",
            (BorderKind::Down, false, true) => "╥",
            (BorderKind::Down, true, false) => "╤",
            (BorderKind::Down, true, true) => "╦",
            (BorderKind::Up, false, false) => "┴",
            (BorderKind::Up, false, true) => "╨",
            (BorderKind::Up, true, false) => "╧",
            (BorderKind::Up, true, true) => "╩",
            (BorderKind::Both, false, false) => "┼",
            (BorderKind::Both, false, true) => "╫",
            (BorderKind::Both, true, false) => "╪",
            (BorderKind::Both, true, true) => "╬",
        }
    }
}

fn horiz_border(
    kind: BorderKind,
    cells: usize,
    cell_size: usize,
    bold_vertical: fn(usize) -> bool,
    bold: bool,
) {
    let mid = if bold { "═" } else { "─" };

    print!("{}", kind.left_corner(bold, true).grey().dim());
    for i in 0..cells {
        print!("{}", mid.repeat(cell_size).grey().dim());
        if i != cells - 1 {
            print!("{}", kind.middle(bold, bold_vertical(i)).grey().dim());
        }
    }
    print!("{}\r\n", kind.right_corner(bold, true).grey().dim());
}
