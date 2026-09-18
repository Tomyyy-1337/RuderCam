use std::{
    io::{BufRead, BufReader, Read},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
    thread,
};

use crate::events::AppEvent;

/// Runs `git add .`, `git commit -m <message>` and `git push` in `repo_dir` on a
/// background thread. Output lines are streamed back as `AppEvent::Output`.
/// Returns a flag that can be set to `true` to cooperatively cancel the
/// remaining git commands.
pub fn push_async(tx: Sender<AppEvent>, message: String, repo_dir: PathBuf) -> Arc<AtomicBool> {
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_thread = cancel.clone();

    thread::spawn(move || {
        let commands: [(&str, Command); 3] = [
            ("git add .", {
                let mut cmd = Command::new("git");
                cmd.arg("add").arg(".");
                cmd
            }),
            ("git commit", {
                let mut cmd = Command::new("git");
                cmd.arg("commit").arg("-m").arg(&message);
                cmd
            }),
            ("git push", {
                let mut cmd = Command::new("git");
                cmd.arg("push");
                cmd
            }),
        ];

        for (label, mut command) in commands {
            if cancel_thread.load(Ordering::Relaxed) {
                let _ = tx.send(AppEvent::Output("Git push cancelled.".to_string()));
                let _ = tx.send(AppEvent::GitPushFinished(Err(
                    "Git push cancelled".to_string()
                )));
                return;
            }

            let _ = tx.send(AppEvent::Output(format!("▶  {label}")));
            command.current_dir(&repo_dir);

            if let Err(e) = run_command_streaming(command, &tx, cancel_thread.clone()) {
                let _ = tx.send(AppEvent::Output(format!("✗  {label} failed: {e}")));
                let _ = tx.send(AppEvent::GitPushFinished(Err(e.to_string())));
                return;
            }
        }

        let _ = tx.send(AppEvent::Output("✓  Git push completed".to_string()));
        let _ = tx.send(AppEvent::GitPushFinished(Ok(())));
    });

    cancel
}

fn run_command_streaming(
    mut command: Command,
    tx: &Sender<AppEvent>,
    cancel: Arc<AtomicBool>,
) -> std::io::Result<()> {
    command
        .env("FORCE_COLOR", "1")
        .env("CLICOLOR_FORCE", "1")
        .env("TERM", "xterm-256color")
        .env("COLORTERM", "truecolor")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child: Child = command.spawn()?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let out_handle = spawn_output_reader(stdout, tx.clone(), cancel.clone());
    let err_handle = spawn_output_reader(stderr, tx.clone(), cancel);

    let status = child.wait()?;
    let _ = out_handle.join();
    let _ = err_handle.join();

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "git command failed",
        ))
    }
}

fn spawn_output_reader<R: Read + Send + 'static>(
    reader: R,
    tx: Sender<AppEvent>,
    cancel: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            let _ = tx.send(AppEvent::Output(line));
        }
    })
}

/// Returns the path to the treiber repository directory. This is the parent of
/// the `sign_update` crate directory, which is resolved relative to the current
/// working directory.
pub fn repo_dir() -> PathBuf {
    std::env::current_dir()
        .ok()
        .and_then(|cwd| cwd.parent().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(".."))
}
