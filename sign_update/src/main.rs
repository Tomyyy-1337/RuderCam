mod pipeline;
mod ui;

use std::{io, sync::mpsc, thread};

use pipeline::{run_pipeline, AppEvent};
use ui::{run_ui, RunOutcome};

fn main() -> io::Result<()> {
    loop {
        let (tx, rx) = mpsc::channel::<AppEvent>();

        thread::spawn(move || {
            run_pipeline(tx);
        });

        if run_ui(rx)? == RunOutcome::Quit {
            break;
        }
    }

    Ok(())
}