use app::App;
use std::io;

mod app;
mod board;
mod ui;

fn main() -> io::Result<()> {
    let mut app = App::new();
    app.run()
}
