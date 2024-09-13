use crossterm::style::Stylize;
use std::{
    fmt::Display,
    ops::{Index, IndexMut},
    str::FromStr,
};

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

    pub fn is_empty(&self) -> bool {
        matches!(self, Cell::Empty)
    }

    pub fn maybe_get_num(&self, num: u8) -> bool {
        if let Cell::Maybe(m) = self {
            (m & (1 << num as u16)) != 0
        } else {
            false
        }
    }

    pub fn entropy(&self) -> u8 {
        match self {
            Cell::Maybe(v) => v.count_ones() as u8,
            _ => unimplemented!(),
        }
    }

    pub fn maybe_values(&self) -> Vec<u8> {
        match self {
            Cell::Maybe(_v) => (1..=9).filter(|&i| self.maybe_get_num(i)).collect(),
            _ => unimplemented!(),
        }
    }

    pub fn maybe_unset(&mut self, value: u8) {
        let mask = !(1 << value);
        match self {
            Cell::Maybe(m) => *m = *m & mask,
            _ => (),
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

#[derive(Clone, Copy)]
pub struct Board {
    cells: [Cell; 81],
}

impl Default for Board {
    fn default() -> Self {
        Self {
            cells: [Cell::default(); 81],
        }
    }
}

impl Board {
    pub fn set_cell(&mut self, idx: (usize, usize), value: Option<u8>) {
        self.cells[idx.0 * 9 + idx.1] = match value {
            Some(v) => Cell::Good(v),
            None => Cell::Empty,
        };
        self.calc_cell_states();
    }

    pub fn can_solve(&self) -> bool {
        for i in 0..81 {
            if matches!(self.cells[i], Cell::Bad(_)) {
                return false;
            }
        }
        true
    }

    pub fn clear_maybe(&mut self) {
        for i in 0..81 {
            if matches!(self.cells[i], Cell::Maybe(_) | Cell::Collapsed(_)) {
                self.cells[i] = Cell::Empty;
            }
        }
    }

    pub fn clear_all(&mut self) {
        for i in 0..81 {
            self.cells[i] = Cell::Empty;
        }
    }

    pub fn get_uncollapsed(&self) -> Vec<(usize, Cell)> {
        self.cells
            .clone()
            .into_iter()
            .enumerate()
            .filter(|(_, c)| c.is_maybe())
            .collect()
    }

    pub fn collapse(&mut self, index: usize, value: u8) {
        self.cells[index] = Cell::Collapsed(value);

        let cell = (index / 9, index % 9);
        let square = ((cell.0 / 3) * 3, (cell.1 / 3) * 3);
        for i in 0..9 {
            let idx = cell.0 * 9 + i;
            self.cells[idx].maybe_unset(value);

            let idx = i * 9 + cell.1;
            self.cells[idx].maybe_unset(value);

            let idx = (square.0 + i / 3) * 9 + square.1 + i % 3;
            self.cells[idx].maybe_unset(value);
        }
    }

    pub fn init_maybe(&mut self) {
        for i in 0..81 {
            if !self.cells[i].is_empty() {
                continue;
            }

            let mut value = Cell::Maybe(0b1111111110);

            let cell = (i / 9, i % 9);
            let square = ((cell.0 / 3) * 3, (cell.1 / 3) * 3);
            self.row(cell.0).for_each(|c| {
                if let Some(v) = c.value() {
                    value.maybe_unset(v)
                }
            });
            self.col(cell.1).for_each(|c| {
                if let Some(v) = c.value() {
                    value.maybe_unset(v)
                }
            });
            self.square(square).for_each(|c| {
                if let Some(v) = c.value() {
                    value.maybe_unset(v)
                }
            });

            self.cells[i] = value;
        }
    }
}

impl Board {
    fn calc_cell_states(&mut self) {
        for i in 0..81 {
            self.cells[i].make_good();
        }

        for i in 0..9 {
            if !self.is_row_ok(i) {
                for j in 0..9 {
                    self.cells[i * 9 + j].make_bad();
                }
            }
            if !self.is_col_ok(i) {
                for j in 0..9 {
                    self.cells[j * 9 + i].make_bad();
                }
            }
            let square_off = ((i / 3) * 3, (i % 3) * 3);
            if !self.is_square_ok(square_off) {
                for j in 0..9 {
                    let (i, j) = (j / 3 + square_off.0, j % 3 + square_off.1);
                    self.cells[i * 9 + j].make_bad();
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
        &self.cells[i.0 * 9 + i.1]
    }
}

impl Index<usize> for Board {
    type Output = Cell;

    fn index(&self, i: usize) -> &Self::Output {
        &self.cells[i]
    }
}

impl IndexMut<(usize, usize)> for Board {
    fn index_mut(&mut self, i: (usize, usize)) -> &mut Self::Output {
        &mut self.cells[i.0 * 9 + i.1]
    }
}

impl IndexMut<usize> for Board {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.cells[i]
    }
}

impl FromStr for Board {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut board = Board::default();

        for (i, l) in s.lines().enumerate() {
            for (j, c) in l.chars().enumerate() {
                match c {
                    '1'..='9' => {
                        if i > 8 || j > 8 {
                            return Err(format!(
                                "Expected max 9 characters per line and max 9 lines"
                            ));
                        }
                        board[(i, j)] = Cell::Good(c as u8 - b'0');
                    }
                    ' ' => (),
                    _ => return Err(format!("Unexpected character {}", c)),
                }
            }
        }
        board.calc_cell_states();

        Ok(board)
    }
}
