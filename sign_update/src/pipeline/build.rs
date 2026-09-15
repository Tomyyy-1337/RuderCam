use std::{
    io,
    path::{Path, PathBuf},
    process::Command,
    sync::mpsc::{self, Sender},
    thread,
};

use crate::events::{AppEvent, Task};

use super::{
    BACKEND_BIN_PATH, BACKEND_CODE_PATH, BUILD_STATE_DIR, STATIC_DIR,
    commands::run_command_streaming,
    hash::{hash_directory, hash_path, hash_paths},
};

pub fn run_parallel_build(tx: &Sender<AppEvent>) -> io::Result<()> {
    let _ = tx.send(AppEvent::ParallelStarted);

    let frontend_handle = spawn_parallel_task(Task::Frontend, tx.clone(), build_frontend);
    let backend_handle = spawn_parallel_task(Task::Backend, tx.clone(), build_backend);

    let (frontend_result, frontend_lines) = frontend_handle
        .join()
        .expect("frontend build thread panicked");
    let (backend_result, backend_lines) = backend_handle
        .join()
        .expect("backend build thread panicked");

    let _ = tx.send(AppEvent::ParallelFinished);

    let _ = tx.send(AppEvent::TaskStarted(Task::Frontend));
    for line in frontend_lines {
        let _ = tx.send(AppEvent::Output(line));
    }
    let _ = tx.send(AppEvent::TaskFinished(
        Task::Frontend,
        frontend_result
            .as_ref()
            .map(|_| ())
            .map_err(|e| e.to_string()),
    ));

    let _ = tx.send(AppEvent::TaskStarted(Task::Backend));
    for line in backend_lines {
        let _ = tx.send(AppEvent::Output(line));
    }
    let _ = tx.send(AppEvent::TaskFinished(
        Task::Backend,
        backend_result
            .as_ref()
            .map(|_| ())
            .map_err(|e| e.to_string()),
    ));

    frontend_result?;
    backend_result?;
    Ok(())
}

fn spawn_parallel_task(
    task: Task,
    tx: Sender<AppEvent>,
    f: impl FnOnce(&Sender<AppEvent>) -> io::Result<()> + Send + 'static,
) -> thread::JoinHandle<(io::Result<()>, Vec<String>)> {
    thread::spawn(move || {
        let (local_tx, local_rx) = mpsc::channel::<AppEvent>();
        let output_tx = tx.clone();

        let collector = thread::spawn(move || {
            let mut lines = Vec::new();
            while let Ok(AppEvent::Output(line)) = local_rx.recv() {
                let _ = output_tx.send(AppEvent::ParallelOutput(task, line.clone()));
                lines.push(line);
            }
            lines
        });

        let result = f(&local_tx);
        drop(local_tx);
        let lines = collector.join().expect("output collector thread panicked");
        let event_result = result.as_ref().map(|_| ()).map_err(|e| e.to_string());
        let _ = tx.send(AppEvent::ParallelTaskFinished(task, event_result));

        (result, lines)
    })
}

fn build_frontend(tx: &Sender<AppEvent>) -> io::Result<()> {
    let input_hash = hash_directory(Path::new("../frontend"), &["node_modules", "dist"])?;
    if build_is_current("frontend", input_hash, Path::new(STATIC_DIR))? {
        let _ = tx.send(AppEvent::Output(
            "Frontend unchanged; skipping build".into(),
        ));
        return Ok(());
    }

    let mut command = Command::new("npm.cmd");
    command.arg("run").arg("check").current_dir("../frontend");
    run_command_streaming(command, tx, "frontend check failed")?;

    let mut command = Command::new("npm.cmd");
    command.arg("run").arg("build").current_dir("../frontend");
    run_command_streaming(command, tx, "frontend build failed")?;
    save_build_state("frontend", input_hash, Path::new(STATIC_DIR))
}

fn build_backend(tx: &Sender<AppEvent>) -> io::Result<()> {
    let input_hash = hash_paths(&[
        PathBuf::from("../backend/Cargo.toml"),
        PathBuf::from("../backend/Cargo.lock"),
        PathBuf::from("../backend/Dockerfile"),
        PathBuf::from("../backend/src"),
    ])?;
    let output_path = Path::new(BACKEND_CODE_PATH).join(BACKEND_BIN_PATH);
    if build_is_current("backend", input_hash, &output_path)? {
        let _ = tx.send(AppEvent::Output("Backend unchanged; skipping build".into()));
        return Ok(());
    }

    let backend_dir = Path::new(BACKEND_CODE_PATH);

    let mut build = Command::new("docker");
    build
        .env("DOCKER_BUILDKIT", "1")
        .arg("build")
        .arg("-t")
        .arg("pi-backend")
        .arg(".")
        .current_dir(backend_dir);
    run_command_streaming(build, tx, "docker build failed")?;

    let mut create = Command::new("docker");
    create
        .arg("create")
        .arg("--name")
        .arg("temp")
        .arg("pi-backend");
    run_command_streaming(create, tx, "docker create failed")?;

    let mut cp = Command::new("docker");
    cp.arg("cp")
        .arg("temp:/backend")
        .arg(BACKEND_BIN_PATH)
        .current_dir(backend_dir);
    run_command_streaming(cp, tx, "docker cp failed")?;

    let mut rm = Command::new("docker");
    rm.arg("rm").arg("temp");
    run_command_streaming(rm, tx, "docker rm failed")?;
    save_build_state("backend", input_hash, &output_path)
}

fn build_is_current(name: &str, input_hash: [u8; 32], output_path: &Path) -> io::Result<bool> {
    let output_hash = match hash_path(output_path)? {
        Some(hash) => hash,
        None => return Ok(false),
    };
    let state = match std::fs::read(Path::new(BUILD_STATE_DIR).join(format!("{name}.sha256"))) {
        Ok(state) if state.len() == 64 => state,
        Ok(_) | Err(_) => return Ok(false),
    };

    Ok(state[..32] == input_hash && state[32..] == output_hash)
}

fn save_build_state(name: &str, input_hash: [u8; 32], output_path: &Path) -> io::Result<()> {
    let output_hash = hash_path(output_path)?.ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "generated build output is missing")
    })?;
    std::fs::create_dir_all(BUILD_STATE_DIR)?;
    let mut state = Vec::with_capacity(64);
    state.extend_from_slice(&input_hash);
    state.extend_from_slice(&output_hash);
    std::fs::write(
        Path::new(BUILD_STATE_DIR).join(format!("{name}.sha256")),
        state,
    )
}
