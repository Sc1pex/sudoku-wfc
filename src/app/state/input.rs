use solve::SolveState;

use super::*;

#[derive(Default)]
pub struct InputState {
    selected: (usize, usize),
}

impl InputState {
    fn set_cursor(&self, ui: &mut Ui) {
        ui.set_cursor_onboard(Some(self.selected)).unwrap();
    }
}

impl State for InputState {
    fn handle_key_event(&mut self, data: &mut AppData, k: KeyEvent) -> Option<Box<dyn State>> {
        match k.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected.0 -= 1;
                if self.selected.0 == usize::MAX {
                    self.selected.0 = 8;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected.0 += 1;
                if self.selected.0 == 9 {
                    self.selected.0 = 0;
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.selected.1 -= 1;
                if self.selected.1 == usize::MAX {
                    self.selected.1 = 8;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.selected.1 += 1;
                if self.selected.1 == 9 {
                    self.selected.1 = 0;
                }
            }
            KeyCode::Tab => {
                self.selected.1 = self.selected.1 + 1;
                if self.selected.1 == 9 {
                    self.selected.1 = 0;
                    self.selected.0 = self.selected.0 + 1;
                    if self.selected.0 == 9 {
                        self.selected.0 = 0;
                    }
                }
            }

            KeyCode::Char(c) if matches!(c, '1'..='9') => {
                let digit = c.to_digit(10).unwrap() as u8;
                data.board.set_cell(self.selected, Some(digit));
            }
            KeyCode::Backspace | KeyCode::Delete => data.board.set_cell(self.selected, None),

            KeyCode::Char('s') => {
                if !data.board.can_solve() {
                    data.ui.add_msg((0, 38), || {
                        print!("Can't start solving. Board is invalid\r\n")
                    });
                } else {
                    return Some(Box::new(SolveState::new(data)));
                }
            }

            _ => (),
        }

        None
    }

    fn draw(&self, data: &mut AppData) {
        self.set_cursor(&mut data.ui);
    }
}
