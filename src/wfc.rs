use crate::board::Board;
use rand::{seq::SliceRandom, thread_rng};

#[derive(Default)]
pub struct Wfc {
    stack: Vec<Board>,
}

impl Wfc {
    pub fn init(&mut self, b: Board) {
        self.stack = vec![b];
    }

    pub fn step(&mut self) -> (Board, bool) {
        let b = self.stack.last().unwrap().clone();
        let mut posibilities = b
            .get_uncollapsed()
            .into_iter()
            .map(|(i, c)| (i, c.entropy()))
            .collect::<Vec<_>>();
        if posibilities.is_empty() {
            return (b.clone(), true);
        }

        posibilities.sort_by(|(_, e1), (_, e2)| e1.cmp(&e2));
        let cell = posibilities.first().unwrap();
        let els = b[cell.0].maybe_values();

        match els.choose(&mut thread_rng()) {
            Some(&v) => {
                let mut bb = b.clone();

                self.stack.last_mut().unwrap()[cell.0].maybe_unset(v);

                bb.collapse(cell.0, v);
                self.stack.push(bb);
            }
            None => {
                self.stack.pop();
            }
        }

        (self.stack.last().unwrap().clone(), false)
    }
}
