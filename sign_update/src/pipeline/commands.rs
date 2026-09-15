use std::{
    io::{self, BufRead, BufReader, Read},
    process::{Child, Command, Stdio},
    sync::mpsc::Sender,
    thread,
};

use crate::events::AppEvent;

pub fn run_command_streaming(
    mut command: Command,
    tx: &Sender<AppEvent>,
    error_message: &'static str,
) -> io::Result<()> {
    command
        .env("FORCE_COLOR", "1")
        .env("CLICOLOR_FORCE", "1")
        .env("TERM", "xterm-256color")
        .env("COLORTERM", "truecolor")
        .env("CARGO_TERM_COLOR", "always")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child: Child = command.spawn()?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let out_handle = spawn_output_reader(stdout, tx.clone());
    let err_handle = spawn_output_reader(stderr, tx.clone());

    let status = child.wait()?;
    let _ = out_handle.join();
    let _ = err_handle.join();

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, error_message))
    }
}

fn spawn_output_reader<R: Read + Send + 'static>(
    reader: R,
    tx: Sender<AppEvent>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            let _ = tx.send(AppEvent::Output(line));
        }
    })
}
