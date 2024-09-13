use crate::{board::Board, ui::Ui, wfc::Wfc};
use crossterm::event::{KeyCode, KeyModifiers};
use std::{
    io,
    sync::mpsc::{self, Sender},
    time::Duration,
};

enum Event {
    Term(crossterm::event::Event),
    SolveTick,
}
enum TickCtl {
    Start,
    Stop,
}

pub struct App {
    board: Board,
    ui: Ui,
    wfc: Wfc,

    selected: (usize, usize),

    solving: bool,
}

impl App {
    pub fn new() -> Self {
        let mut s = Self {
            board: Board::default(),
            ui: Ui::new().unwrap(),
            wfc: Wfc::default(),

            selected: (0, 0),
            solving: false,
        };

        s.toggle_help_ui();
        s
    }

    pub fn run(&mut self) -> io::Result<()> {
        let (event_tx, event_rx) = mpsc::channel();

        let term_tx = event_tx.clone();
        let _c = std::thread::spawn(move || crossterm_el(term_tx));

        let (tick_ctl_tx, tick_ctl_rx) = mpsc::channel::<TickCtl>();
        let _t = std::thread::spawn(move || ticker(tick_ctl_rx, event_tx));

        self.ui.draw(&self.board)?;
        self.ui.set_cursor_onboard(Some(self.selected))?;

        while let Ok(e) = event_rx.recv() {
            let exit = match e {
                Event::Term(e) => self.handle_term_event(e, &tick_ctl_tx),
                Event::SolveTick => self.handle_tick_event(&tick_ctl_tx),
            };

            if exit {
                break;
            }

            self.ui.draw(&self.board)?;
            self.ui.set_cursor_onboard(Some(self.selected))?;
        }

        Ok(())
    }
}

impl App {
    fn handle_term_event(
        &mut self,
        e: crossterm::event::Event,
        tick_ctl_tx: &Sender<TickCtl>,
    ) -> bool {
        match e {
            crossterm::event::Event::Key(k) => match k.code {
                // Quit
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => return true,
                KeyCode::Char('d') if k.modifiers.contains(KeyModifiers::CONTROL) => return true,

                // Navigation
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

                // Input
                KeyCode::Char(c) if matches!(c, '1'..='9') && !self.solving => {
                    let digit = c.to_digit(10).unwrap() as u8;
                    self.ui.remove_msg((0, 38));
                    self.board.set_cell(self.selected, Some(digit));
                }
                KeyCode::Backspace | KeyCode::Delete if !self.solving => {
                    self.board.set_cell(self.selected, None)
                }
                KeyCode::Char('s') if !self.solving => {
                    if !self.board.can_solve() {
                        self.ui.add_msg((0, 38), || {
                            print!("Can't start solving. Board is invalid\r\n")
                        });
                    } else {
                        self.board.init_maybe();
                        self.wfc.init(self.board);
                        self.solving = true;
                        tick_ctl_tx.send(TickCtl::Start).unwrap();
                    }
                }
                KeyCode::Char('c') => {
                    tick_ctl_tx.send(TickCtl::Stop).unwrap();
                    self.solving = false;
                    self.board.clear_maybe();
                }
                KeyCode::Char('C') => {
                    tick_ctl_tx.send(TickCtl::Stop).unwrap();
                    self.solving = false;
                    self.board.clear_all();
                }

                _ => (),
            },
            _ => (),
        }

        false
    }

    fn handle_tick_event(&mut self, tick_ctl_tx: &Sender<TickCtl>) -> bool {
        let (b, done) = self.wfc.step();
        self.board = b;

        if done {
            tick_ctl_tx.send(TickCtl::Stop).unwrap();
            self.ui.add_msg((0, 38), || print!("Solved!"));
        }

        false
    }

    fn toggle_help_ui(&mut self) {
        let help = || {
            print!(
                "Keybinds:\r\n  \
            ?         -> toggle this message\r\n  \
            arrows    -> move around the board\r\n  \
            tab       -> go to next space\r\n  \
            1..9      -> set current space\r\n  \
            backspace -> clear current space\r\n  \
            s         -> start solving\r\n  \
            c         -> clear solved spaces\r\n  \
            C         -> clear entire board\r\n  \
            q or esc  -> quit\r\n"
            )
        };

        if self.ui.has((0, 39)) {
            self.ui.remove_msg((0, 39));
        } else {
            self.ui.add_msg((0, 39), help);
        }
    }
}

fn crossterm_el(event_tx: mpsc::Sender<Event>) -> io::Result<()> {
    loop {
        let e = crossterm::event::read()?;
        event_tx.send(Event::Term(e)).unwrap();
    }
}

fn ticker(ctl: mpsc::Receiver<TickCtl>, tick: mpsc::Sender<Event>) {
    loop {
        match ctl.recv() {
            Ok(TickCtl::Start) => loop {
                tick.send(Event::SolveTick).unwrap();

                std::thread::sleep(Duration::from_millis(50));
                if matches!(
                    ctl.try_recv(),
                    Ok(TickCtl::Stop) | Err(mpsc::TryRecvError::Disconnected)
                ) {
                    break;
                }
            },
            Err(mpsc::RecvError) => break,
            _ => (),
        }
    }
}
