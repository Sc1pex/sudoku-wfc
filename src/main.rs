use app::App;
use clap::Parser;
use std::io;

mod app;
mod board;
mod ui;
mod wfc;

/// Sudoku solver using the wave function collapse algorithm
#[derive(Parser)]
struct Args {
    /// Optional path to file of initial values
    #[arg(short)]
    file: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let mut app = if let Some(file) = args.file {
        let data = std::fs::read_to_string(file).expect("Couldn't read file");
        App::from_data(data)
    } else {
        App::new()
    };
    app.run()
}
