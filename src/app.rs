use crate::{board::Board, ui::Ui, wfc::Wfc};
use crossterm::event::{KeyCode, KeyModifiers};
use state::{input::InputState, State};
use std::{
    io,
    sync::mpsc::{self, channel, Sender},
    time::Duration,
};

mod state;

enum Event {
    Term(crossterm::event::Event),
    Tick,
}
enum TickCtl {
    Start,
    Stop,
}

struct AppData {
    board: Board,
    ui: Ui,
    wfc: Wfc,

    tickctl_tx: Sender<TickCtl>,
}

impl AppData {
    fn new() -> Self {
        let mut s = Self {
            board: Board::default(),
            ui: Ui::new().unwrap(),
            wfc: Wfc::default(),

            tickctl_tx: channel().0,
        };

        s.toggle_help_ui();
        s
    }

    fn from_data(data: String) -> Self {
        let mut s = Self {
            board: data.parse().unwrap(),
            ui: Ui::new().unwrap(),
            wfc: Wfc::default(),

            tickctl_tx: channel().0,
        };

        s.toggle_help_ui();
        s
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

        if self.ui.has((0, 40)) {
            self.ui.remove_msg((0, 40));
        } else {
            self.ui.add_msg((0, 40), help);
        }
    }
}

pub struct App {
    state: Box<dyn State>,
    data: AppData,

    exit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            data: AppData::new(),
            state: Box::new(InputState::default()),

            exit: false,
        }
    }

    pub fn from_data(data: String) -> Self {
        Self {
            data: AppData::from_data(data),
            state: Box::new(InputState::default()),

            exit: false,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let (event_tx, event_rx) = mpsc::channel();

        let term_tx = event_tx.clone();
        let _c = std::thread::spawn(move || crossterm_el(term_tx));

        let (tickctl_tx, tickctl_rx) = mpsc::channel::<TickCtl>();
        let _t = std::thread::spawn(move || ticker(tickctl_rx, event_tx));
        self.data.tickctl_tx = tickctl_tx;

        self.data.ui.draw(&self.data.board)?;
        self.state.draw(&mut self.data);

        while let Ok(e) = event_rx.recv() {
            match e {
                Event::Term(e) => self.handle_term_event(e),
                Event::Tick => {
                    let new_state = self.state.handle_tick_event(&mut self.data);
                    if let Some(state) = new_state {
                        self.state = state
                    }
                }
            };

            if self.exit {
                break;
            }

            self.data.ui.draw(&self.data.board)?;
            self.state.draw(&mut self.data);
        }

        Ok(())
    }
}

impl App {
    fn handle_term_event(&mut self, e: crossterm::event::Event) {
        match e {
            crossterm::event::Event::Key(k) => match k.code {
                // Quit
                KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
                KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.exit = true
                }
                KeyCode::Char('d') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.exit = true
                }

                // Input
                KeyCode::Char('c') => {
                    self.state = Box::new(InputState::default());
                    self.data.board.clear_maybe();
                }
                KeyCode::Char('C') => {
                    self.state = Box::new(InputState::default());
                    self.data.board.clear_all();
                }
                KeyCode::Char('?') => {
                    self.data.toggle_help_ui();
                }

                _ => {
                    let new_state = self.state.handle_key_event(&mut self.data, k);
                    if let Some(state) = new_state {
                        self.state = state;
                    }
                }
            },
            _ => (),
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
                tick.send(Event::Tick).unwrap();

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
