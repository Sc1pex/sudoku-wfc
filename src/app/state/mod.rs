use super::*;
use crossterm::event::KeyEvent;

pub mod input;
pub mod solve;

pub trait State {
    #[allow(unused)]
    fn handle_key_event(&mut self, data: &mut AppData, k: KeyEvent) -> Option<Box<dyn State>> {
        None
    }

    #[allow(unused)]
    fn handle_tick_event(&mut self, data: &mut AppData) -> Option<Box<dyn State>> {
        None
    }

    #[allow(unused)]
    fn draw(&self, data: &mut AppData) {}
}
