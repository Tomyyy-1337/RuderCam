use std::{
    fs::File, io::{self, BufRead, BufReader}, path::{Path, PathBuf}, process::{Child, Command, Stdio}, sync::mpsc::{self, Sender}, thread::{self},
};

use sha2::{Digest, Sha256};
use tar::{Builder, Header};

const VERSION_PATH: &str = "version.txt";
const BUILD_STATE_DIR: &str = ".build-state";

pub const STATIC_DIR: &str = "../backend/static";
pub const BACKEND_CODE_PATH: &str = "../backend";
pub const BACKEND_BIN_PATH: &str = "./backend_bin";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Task {
    Frontend,
    Backend,
    Hash,
    Archive,
}

impl Task {
    pub const ALL: [Task; 4] = [Task::Frontend, Task::Backend, Task::Hash, Task::Archive];

    pub fn name(self) -> &'static str {
        match self {
            Task::Frontend => "Build frontend",
            Task::Backend => "Build backend",
            Task::Hash => "Create hash",
            Task::Archive => "Create archive",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Done,
    Failed,
}

pub enum AppEvent {
    TaskStarted(Task),
    Output(String),
    TaskFinished(Task, Result<(), String>),
    ParallelStarted,
    ParallelOutput(Task, String),
    ParallelTaskFinished(Task, Result<(), String>),
    ParallelFinished,
}

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
        run_step(&tx, Task::Archive, |tx| create_archive(tx, read_version_number()?.as_str(), &combined_hash))?;
        Ok(())
    })();

    if let Err(e) = result {
        let _ = tx.send(AppEvent::Output(format!("Pipeline aborted: {e}")));
    }
}

fn run_parallel_build(tx: &Sender<AppEvent>) -> io::Result<()> {
    let _ = tx.send(AppEvent::ParallelStarted);

    let frontend_handle = spawn_parallel_task(Task::Frontend, tx.clone(), build_frontend);
    let backend_handle = spawn_parallel_task(Task::Backend, tx.clone(), build_backend);

    let (frontend_result, frontend_lines) = frontend_handle.join().expect("frontend build thread panicked");
    let (backend_result, backend_lines) = backend_handle.join().expect("backend build thread panicked");

    let _ = tx.send(AppEvent::ParallelFinished);
    
    let _ = tx.send(AppEvent::TaskStarted(Task::Frontend));
    for line in frontend_lines {
        let _ = tx.send(AppEvent::Output(line));
    }
    let _ = tx.send(AppEvent::TaskFinished(Task::Frontend, frontend_result.as_ref().map(|_| ()).map_err(|e| e.to_string())));

    let _ = tx.send(AppEvent::TaskStarted(Task::Backend));
    for line in backend_lines {
        let _ = tx.send(AppEvent::Output(line));
    }
    let _ = tx.send(AppEvent::TaskFinished(Task::Backend, backend_result.as_ref().map(|_| ()).map_err(|e| e.to_string())));

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

fn calculate_hash(tx: &Sender<AppEvent>) -> io::Result<Vec<u8>> {
    let _ = tx.send(AppEvent::Output("\x1b[36mCalculating hash of files...\x1b[0m".into()));

    let backend_path = Path::new(BACKEND_CODE_PATH).join(BACKEND_BIN_PATH);

    let backend_hash = Sha256::digest(&std::fs::read(&backend_path)?);
    let _ = tx.send(AppEvent::Output(format!("\x1b[90mHashed\x1b[0m \x1b[33mBackend binary\x1b[0m")));
    let version_hash = Sha256::digest(read_version_number()?.as_bytes());
    let _ = tx.send(AppEvent::Output(format!("\x1b[90mHashed\x1b[0m \x1b[33mversion.txt\x1b[0m")));
    let mut combined_hash = xor_hashes(&backend_hash, &version_hash);

    for file in all_files_in_dir(STATIC_DIR)? {
        let raw_data = std::fs::read(&file)?;
        let file_hash = Sha256::digest(&raw_data);
        combined_hash = xor_hashes(&combined_hash, &file_hash);
        let _ = tx.send(AppEvent::Output(format!("\x1b[90mHashed\x1b[0m \x1b[33m{file}\x1b[0m")));
    }

    Ok(combined_hash)
}

fn xor_hashes(hash1: &[u8], hash2: &[u8]) -> Vec<u8> {
    hash1.iter().zip(hash2.iter()).map(|(a, b)| a ^ b).collect()
}

fn all_files_in_dir(dir: &str) -> io::Result<Vec<String>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            files.extend(all_files_in_dir(path.to_string_lossy().as_ref())?);
        } else {
            files.push(path.to_string_lossy().into_owned());
        }
    }
    Ok(files)
}

fn create_archive(tx: &Sender<AppEvent>, version: &str, combined_hash: &[u8]) -> io::Result<()> {
    let _ = tx.send(AppEvent::Output("\x1b[36mCreating update archive...\x1b[0m".into()));

    let file = File::create("update.tar")?;
    let mut archive = Builder::new(file);

    archive.append_dir_all("static", STATIC_DIR)?;
    archive.append_file(
        "server",
        &mut File::open(Path::new(BACKEND_CODE_PATH).join(BACKEND_BIN_PATH))?,
    )?;

    let mut version_header = Header::new_gnu();
    version_header.set_size(version.as_bytes().len() as u64);
    archive.append_data(&mut version_header, "version.txt", version.as_bytes())?;

    let mut hash_header = Header::new_gnu();
    hash_header.set_size(combined_hash.len() as u64);
    archive.append_data(&mut hash_header, "update_hash", combined_hash)?;

    archive.into_inner()?;

    let _ = tx.send(AppEvent::Output(
        "\x1b[32mArchive has been created successfully at ./update.tar\x1b[0m".into(),
    ));

    Ok(())
}

fn build_frontend(tx: &Sender<AppEvent>) -> io::Result<()> {
    let input_hash = hash_directory(Path::new("../frontend"), &["node_modules", "dist"])?;
    if build_is_current("frontend", input_hash, Path::new(STATIC_DIR))? {
        let _ = tx.send(AppEvent::Output("Frontend unchanged; skipping build".into()));
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
    let output_hash = hash_path(output_path)?
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "generated build output is missing"))?;
    std::fs::create_dir_all(BUILD_STATE_DIR)?;
    let mut state = Vec::with_capacity(64);
    state.extend_from_slice(&input_hash);
    state.extend_from_slice(&output_hash);
    std::fs::write(Path::new(BUILD_STATE_DIR).join(format!("{name}.sha256")), state)
}

fn hash_paths(paths: &[PathBuf]) -> io::Result<[u8; 32]> {
    let mut hasher = Sha256::new();
    for path in paths {
        hasher.update(path.to_string_lossy().replace('\\', "/").as_bytes());
        hasher.update([0]);
        if path.is_dir() {
            hasher.update(hash_directory(path, &[])?);
        } else {
            hasher.update(std::fs::read(path)?);
        }
        hasher.update([0]);
    }
    Ok(hasher.finalize().into())
}

fn hash_directory(path: &Path, ignored_names: &[&str]) -> io::Result<[u8; 32]> {
    let mut files = Vec::new();
    collect_files(path, path, ignored_names, &mut files)?;
    files.sort();

    let mut hasher = Sha256::new();
    for file in files {
        hash_path_into(&mut hasher, path, &file)?;
    }
    Ok(hasher.finalize().into())
}

fn collect_files(
    root: &Path,
    path: &Path,
    ignored_names: &[&str],
    files: &mut Vec<PathBuf>,
) -> io::Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if ignored_names.iter().any(|name| entry.file_name() == *name) {
            continue;
        }
        if entry_path.is_dir() {
            collect_files(root, &entry_path, ignored_names, files)?;
        } else if entry_path.is_file() {
            files.push(entry_path.strip_prefix(root).unwrap().to_path_buf());
        }
    }
    Ok(())
}

fn hash_path_into(hasher: &mut Sha256, root: &Path, path: &Path) -> io::Result<()> {
    let relative = path.strip_prefix(root).unwrap_or(path);
    hasher.update(relative.to_string_lossy().replace('\\', "/").as_bytes());
    hasher.update([0]);
    hasher.update(std::fs::read(root.join(path))?);
    hasher.update([0]);
    Ok(())
}

fn hash_path(path: &Path) -> io::Result<Option<[u8; 32]>> {
    if !path.exists() {
        return Ok(None);
    }
    if path.is_dir() {
        return Ok(Some(hash_directory(path, &[])?));
    }

    let mut hasher = Sha256::new();
    hasher.update(std::fs::read(path)?);
    Ok(Some(hasher.finalize().into()))
}

/// Spawns the command, streaming its combined stdout/stderr as `Output` events line by line.
fn run_command_streaming(
    mut command: Command,
    tx: &Sender<AppEvent>,
    error_message: &'static str,
) -> io::Result<()> {
    command
        .env("FORCE_COLOR", "1")
        .env("CLICOLOR_FORCE", "1")
        .env("TERM", "xterm-256color")
        .env("COLORTERM", "truecolor")
        .env("CARGO_TERM_COLOR", "always");

    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child: Child = command.spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let tx_out = tx.clone();
    let out_handle = thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let _ = tx_out.send(AppEvent::Output(line));
        }
    });

    let tx_err = tx.clone();
    let err_handle = thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            let _ = tx_err.send(AppEvent::Output(line));
        }
    });

    let status = child.wait()?;
    let _ = out_handle.join();
    let _ = err_handle.join();

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, error_message))
    }
}
