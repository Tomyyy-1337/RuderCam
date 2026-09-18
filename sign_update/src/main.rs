mod events;
mod git;
mod pipeline;
mod ui;
mod upload;
mod text;

use std::{io, sync::mpsc, thread};

use events::AppEvent;
use pipeline::run_pipeline;
use ui::{RunOutcome, init_terminal, restore_terminal, run_app};

fn main() -> io::Result<()> {
    let mut terminal = init_terminal()?;

    loop {
        let (tx, rx) = mpsc::channel::<AppEvent>();

        thread::spawn(move || {
            run_pipeline(tx);
        });

        if run_app(&mut terminal, rx)? == RunOutcome::Quit {
            break;
        }
    }

    restore_terminal(&mut terminal)?;

    Ok(())
}
