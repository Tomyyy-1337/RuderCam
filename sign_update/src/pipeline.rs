mod archive;
mod build;
mod commands;
mod hash;

use std::{io, sync::mpsc::Sender};

use crate::events::{AppEvent, Task};
use archive::create_archive;
use build::run_parallel_build;
use hash::calculate_hash;

const VERSION_PATH: &str = "version.txt";
const BUILD_STATE_DIR: &str = ".build-state";

pub const STATIC_DIR: &str = "../backend/static";
pub const BACKEND_CODE_PATH: &str = "../backend";
pub const BACKEND_BIN_PATH: &str = "./backend_bin";

pub fn read_version_number() -> io::Result<String> {
    Ok(std::fs::read_to_string(VERSION_PATH)?
        .trim()
        .trim_start_matches('v')
        .to_string())
}

pub fn write_version_number(version: &str) -> io::Result<()> {
    std::fs::write(VERSION_PATH, format!("{version}\n"))
}

pub fn run_pipeline(tx: Sender<AppEvent>) {
    let result: io::Result<()> = (|| {
        run_parallel_build(&tx)?;
        let combined_hash = run_step(&tx, Task::Hash, |tx| calculate_hash(tx))?;
        run_step(&tx, Task::Archive, |tx| {
            create_archive(tx, read_version_number()?.as_str(), &combined_hash)
        })?;
        Ok(())
    })();

    if let Err(e) = result {
        let _ = tx.send(AppEvent::Output(format!("Pipeline aborted: {e}")));
    }
}

pub fn run_step<T>(
    tx: &Sender<AppEvent>,
    task: Task,
    f: impl FnOnce(&Sender<AppEvent>) -> io::Result<T>,
) -> io::Result<T> {
    let _ = tx.send(AppEvent::TaskStarted(task));
    match f(tx) {
        Ok(value) => {
            let _ = tx.send(AppEvent::TaskFinished(task, Ok(())));
            Ok(value)
        }
        Err(e) => {
            let _ = tx.send(AppEvent::TaskFinished(task, Err(e.to_string())));
            Err(e)
        }
    }
}
