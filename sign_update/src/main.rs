mod upload;
mod pipeline;
mod ui;

use std::{io, sync::mpsc, thread};

use pipeline::{run_pipeline, AppEvent};
use ui::{init_terminal, restore_terminal, run_ui, RunOutcome};

fn main() -> io::Result<()> {
    let mut terminal = init_terminal()?;

    loop {
        let (tx, rx) = mpsc::channel::<AppEvent>();

        thread::spawn(move || {
            run_pipeline(tx);
        });

        if run_ui(&mut terminal, rx)? == RunOutcome::Quit {
            break;
        }
    }

    restore_terminal(&mut terminal)?;

    Ok(())
}


