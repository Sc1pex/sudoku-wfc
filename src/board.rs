use crossterm::style::Stylize;
use std::{fmt::Display, ops::Index};

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum Cell {
    #[default]
    Empty,
    Good(u8),
    Bad(u8),
    // bit mask of possibilites 1-9
    Maybe(u16),
    Collapsed(u8),
}

impl Cell {
    pub fn is_maybe(&self) -> bool {
        matches!(self, Cell::Maybe(_))
    }

    pub fn maybe_get_num(&self, num: u8) -> bool {
        if let Cell::Maybe(m) = self {
            (m & (1 << num as u16)) != 0
        } else {
            false
        }
    }
}

impl Cell {
    fn value(&self) -> Option<u8> {
        match self {
            Cell::Good(v) => Some(*v),
            Cell::Bad(v) => Some(*v),
            Cell::Collapsed(v) => Some(*v),
            _ => None,
        }
    }

    fn make_bad(&mut self) {
        if let Cell::Good(v) = self {
            *self = Cell::Bad(*v);
        }
    }

    fn make_good(&mut self) {
        if let Cell::Bad(v) = self {
            *self = Cell::Good(*v);
        }
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cell::Good(v) => write!(f, "{}", v.to_string().blue().bold()),
            Cell::Bad(v) => write!(f, "{}", v.to_string().red().bold()),
            Cell::Collapsed(v) => write!(f, "{}", v.to_string().green().bold()),
            Cell::Empty => write!(f, " "),
            Cell::Maybe(_) => unimplemented!(),
        }
    }
}

#[derive(Default)]
pub struct Board {
    cells: [[Cell; 9]; 9],
}

impl Board {
    pub fn set_cell(&mut self, idx: (usize, usize), value: Option<u8>) {
        self.cells[idx.0][idx.1] = match value {
            Some(v) => Cell::Good(v),
            None => Cell::Empty,
        };
        self.calc_cell_states();
    }

    pub fn can_solve(&self) -> bool {
        for i in 0..9 {
            for j in 0..9 {
                if matches!(self.cells[i][j], Cell::Bad(_)) {
                    return false;
                }
            }
        }
        true
    }

    pub fn clear_maybe(&mut self) {
        for i in 0..9 {
            for j in 0..9 {
                if matches!(self[(i, j)], Cell::Maybe(_) | Cell::Collapsed(_)) {
                    self.cells[i][j] = Cell::Empty;
                }
            }
        }
    }

    pub fn clear_all(&mut self) {
        for i in 0..9 {
            for j in 0..9 {
                self.cells[i][j] = Cell::Empty;
            }
        }
    }
}

impl Board {
    fn calc_cell_states(&mut self) {
        for i in 0..9 {
            for j in 0..9 {
                self.cells[i][j].make_good();
            }
        }

        for i in 0..9 {
            if !self.is_row_ok(i) {
                for j in 0..9 {
                    self.cells[i][j].make_bad();
                }
            }
            if !self.is_col_ok(i) {
                for j in 0..9 {
                    self.cells[j][i].make_bad();
                }
            }
            let square_off = ((i / 3) * 3, (i % 3) * 3);
            if !self.is_square_ok(square_off) {
                for j in 0..9 {
                    let (i, j) = (j / 3 + square_off.0, j % 3 + square_off.1);
                    self.cells[i][j].make_bad();
                }
            }
        }
    }

    fn is_row_ok(&self, row: usize) -> bool {
        self.check_idxes(self.row(row))
    }

    fn is_col_ok(&self, col: usize) -> bool {
        self.check_idxes(self.col(col))
    }

    fn is_square_ok(&self, square_offset: (usize, usize)) -> bool {
        self.check_idxes(self.square(square_offset))
    }

    fn check_idxes<'a>(&'a self, idxes: impl Iterator<Item = &'a Cell>) -> bool {
        idxes
            .fold([0; 9], |mut acc, c| {
                if let Some(v) = c.value() {
                    acc[v as usize - 1] += 1;
                }
                acc
            })
            .into_iter()
            .all(|x| x < 2)
    }

    fn row<'a>(&'a self, row: usize) -> impl Iterator<Item = &'a Cell> {
        (0..9).map(move |i| (row, i)).map(|i| &self[i])
    }

    fn col<'a>(&'a self, col: usize) -> impl Iterator<Item = &'a Cell> {
        (0..9).map(move |i| (i, col)).map(|i| &self[i])
    }

    fn square<'a>(&'a self, square_offset: (usize, usize)) -> impl Iterator<Item = &'a Cell> {
        (0..9)
            .map(move |i| (i / 3 + square_offset.0, i % 3 + square_offset.1))
            .map(|i| &self[i])
    }
}

impl Index<(usize, usize)> for Board {
    type Output = Cell;

    fn index(&self, i: (usize, usize)) -> &Self::Output {
        &self.cells[i.0][i.1]
    }
}
