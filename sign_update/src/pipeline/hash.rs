use std::{
    io,
    path::{Path, PathBuf},
    sync::mpsc::Sender,
};

use sha2::{Digest, Sha256};

use crate::events::AppEvent;

use super::{BACKEND_BIN_PATH, BACKEND_CODE_PATH, STATIC_DIR, read_version_number};

pub fn calculate_hash(tx: &Sender<AppEvent>) -> io::Result<Vec<u8>> {
    let _ = tx.send(AppEvent::Output(
        "\x1b[36mCalculating hash of files...\x1b[0m".into(),
    ));

    let backend_path = Path::new(BACKEND_CODE_PATH).join(BACKEND_BIN_PATH);

    let backend_hash = Sha256::digest(&std::fs::read(&backend_path)?);
    let _ = tx.send(AppEvent::Output(format!(
        "\x1b[90mHashed\x1b[0m \x1b[33mBackend binary\x1b[0m"
    )));
    let version_hash = Sha256::digest(read_version_number()?.as_bytes());
    let _ = tx.send(AppEvent::Output(format!(
        "\x1b[90mHashed\x1b[0m \x1b[33mversion.txt\x1b[0m"
    )));
    let mut combined_hash = xor_hashes(&backend_hash, &version_hash);

    for file in all_files_in_dir(STATIC_DIR)? {
        let raw_data = std::fs::read(&file)?;
        let file_hash = Sha256::digest(&raw_data);
        combined_hash = xor_hashes(&combined_hash, &file_hash);
        let _ = tx.send(AppEvent::Output(format!(
            "\x1b[90mHashed\x1b[0m \x1b[33m{file}\x1b[0m"
        )));
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

pub fn hash_paths(paths: &[PathBuf]) -> io::Result<[u8; 32]> {
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

pub fn hash_directory(path: &Path, ignored_names: &[&str]) -> io::Result<[u8; 32]> {
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

pub fn hash_path(path: &Path) -> io::Result<Option<[u8; 32]>> {
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
