use super::*;

pub struct SolveState {
    tickctl_tx: Sender<TickCtl>,
}

impl SolveState {
    pub fn new(data: &mut AppData) -> Self {
        data.board.init_maybe();
        data.wfc.init(data.board);
        data.tickctl_tx.send(TickCtl::Start).unwrap();

        Self {
            tickctl_tx: data.tickctl_tx.clone(),
        }
    }
}

impl Drop for SolveState {
    fn drop(&mut self) {
        self.tickctl_tx.send(TickCtl::Stop).unwrap()
    }
}

impl State for SolveState {
    fn handle_tick_event(&mut self, data: &mut AppData) -> Option<Box<dyn State>> {
        let (b, done) = data.wfc.step();
        data.board = b;

        if done {
            data.ui.add_msg((0, 38), || print!("Solved!"));
            return Some(Box::new(InputState::default()));
        }
        None
    }
}
