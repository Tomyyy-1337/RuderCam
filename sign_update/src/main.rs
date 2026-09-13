use std::{io::{self, Write}, path::Path, process::{Command, Stdio}};
use sha2::{Sha256, Digest};
use tar::{Builder, Header};
use std::fs::File;

// This will be displayed to the User as the current version of the Firmware. 
const VERSION_NUMBER: &str = "0.2.2";

const STATIC_DIR: &str = "../backend/static";
const BACKEND_CODE_PATH: &str = "../backend";
const BACKEND_BIN_PATH: &str = "./backend_bin"; // Relative to BACKEND_CODE_PATH

fn main() {
    compile_and_sign(VERSION_NUMBER).expect("Failed to compile and sign the update package");
}

pub fn compile_and_sign(version: &str) -> io::Result<()> {
    // Build frontend and backend
    build_frontend()?;
    build_backend()?;

    // Calculate Hash
    let combined_hash = calculate_hash()?;

    // Create Archive
    print!("\rCreating update archive...             ");
    std::io::stdout().flush().unwrap();

    let file = File::create("update.tar").unwrap();
    let mut archive = Builder::new(file);

    archive.append_dir_all("static", STATIC_DIR)?;
    archive.append_file("server", &mut File::open(Path::new(BACKEND_CODE_PATH).join(BACKEND_BIN_PATH))?)?;
    
    let mut version_header = Header::new_gnu();
    version_header.set_size(version.as_bytes().len() as u64);
    archive.append_data(&mut version_header, "version.txt", version.as_bytes())?;

    let mut hash_header = Header::new_gnu();
    hash_header.set_size(combined_hash.len() as u64);
    archive.append_data(&mut hash_header, "update_hash", combined_hash.as_slice())?;
    
    archive.into_inner()?;    

    println!("\rArchive has been created successfully at ./update.tar");

    Ok(())
}

pub fn calculate_hash() -> io::Result<Vec<u8>> {
    print!("Calculating hash of files...");
    std::io::stdout().flush().unwrap();

    let backend_path = Path::new(BACKEND_CODE_PATH).join(BACKEND_BIN_PATH);

    let backend_hash = Sha256::digest(&std::fs::read(&backend_path)?);
    let version_hash = Sha256::digest(VERSION_NUMBER.as_bytes());
    let mut combined_hash = xor_hashes(&backend_hash, &version_hash);
    
    for file in all_files_in_dir(STATIC_DIR)? {
        let raw_data = std::fs::read(file)?;
        let file_hash = Sha256::digest(&raw_data);
        combined_hash = xor_hashes(&mut combined_hash, &file_hash);
    }

    Ok(combined_hash)
}

pub fn xor_hashes(hash1: &[u8], hash2: &[u8]) -> Vec<u8> {
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

fn build_frontend() -> io::Result<()> {
    print!("Building frontend...");
    std::io::stdout().flush().unwrap();

    let mut command = Command::new("npm.cmd");
    command
        .arg("run")
        .arg("build")
        .current_dir("../frontend");

    run_command_silently(command, "frontend build failed")?;

    println!("\rFrontend built successfully.");

    Ok(())
}

fn build_backend() -> io::Result<()> {
    print!("Building backend...");
    std::io::stdout().flush().unwrap();

    let backend_dir = Path::new(BACKEND_CODE_PATH);

    let mut build = Command::new("docker");
    build
        .arg("build")
        .arg("-t")
        .arg("pi-backend")
        .arg(".")
        .current_dir(backend_dir);
    run_command_silently(build, "docker build failed")?;

    let mut create = Command::new("docker");
    create
        .arg("create")
        .arg("--name")
        .arg("temp")
        .arg("pi-backend");
    run_command_silently(create, "docker create failed")?;

    let mut cp = Command::new("docker");
    cp
        .arg("cp")
        .arg("temp:/backend")
        .arg(BACKEND_BIN_PATH)
        .current_dir(backend_dir);
    run_command_silently(cp, "docker cp failed")?;

    let mut rm = Command::new("docker");
    rm.arg("rm").arg("temp");
    run_command_silently(rm, "docker rm failed")?;

    println!("\rBackend built successfully.");

    Ok(())
}

fn run_command_silently(mut command: Command, error_message: &'static str) -> io::Result<()> {
    let status = command
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, error_message))
    }
}