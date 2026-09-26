use std::{fs, io, path::PathBuf, time::Duration};

use axum::{body::{Body, to_bytes}, http::StatusCode};
use sha2::{Digest, Sha256};
use tokio::{runtime::LocalOptions};

fn main() { 
    tokio::runtime::Builder::new_current_thread()
        .max_blocking_threads(1)
        .enable_all()
        .build_local(LocalOptions::default())
        .unwrap()
        .block_on(async {
            let app = axum::Router::new()
                .route("/api/update", axum::routing::post(update_handler))
                .route("/api/current_version", axum::routing::get(current_version_handler))
                .fallback_service(tower_http::services::ServeDir::new("./static"))
                .layer(tower_http::cors::CorsLayer::permissive());

            let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 4000));
            println!("Update service running on http://127.0.0.1:4000/");

            let listener = loop {
                match tokio::net::TcpListener::bind(addr).await {
                    Ok(listener) => break listener,
                    Err(e) => {
                        println!("Failed to bind to {}: {}. Retrying...", addr, e);
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                }
            };

            let _ = axum::serve(listener, app).await;
        });
}

async fn current_version_handler() -> String {
    // Read the current version from a file or return a default value
    let version_file_path = PathBuf::from("/home/pi/treiber/version.txt");
    match fs::read_to_string(&version_file_path) {
        Ok(version) => version.trim().to_string(),
        Err(_) => "No firmware".to_string(), // Default version if file not found
    }
}

async fn update_handler(req: axum::http::Request<Body>) -> StatusCode {
    let body = req.into_body();
    let raw_archive = to_bytes(body, usize::MAX).await.unwrap();

    println!("Received update request with archive size: {} bytes", raw_archive.len());

    if checksum_is_valid(&raw_archive) {
        #[cfg(not(target_os = "linux"))]
        {
            println!("Not running on Linux, skipping writing to disk.");
        }

        #[cfg(target_os = "linux")]
        {
            println!("Checksum is valid, writing archive to disk...");

            let result = modify_sd_card(|| async {
                write_archive_to_disk(&raw_archive, "/home/pi/treiber")
            }).await;
            
            if let Err(e) = result {
                eprintln!("Failed to write archive to disk: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
    } else {
        eprintln!("Checksum is invalid, rejecting update.");
        return StatusCode::BAD_REQUEST;
    }

    StatusCode::OK
}

async fn modify_sd_card(f: impl AsyncFnOnce() -> Result<(),io::Error>) -> Result<(), io::Error> {
    let _ = tokio::process::Command::new("systemctl")
        .arg("stop")
        .arg("backend.service")
        .status().await;

    tokio::process::Command::new("rm")
        .arg("-rf")
        .arg("/home/pi/treiber/static")
        .status().await?;

    f().await?;

    tokio::process::Command::new("chmod")
        .arg("+x")
        .arg("/home/pi/treiber/server")
        .status().await?;

    tokio::process::Command::new("systemctl")
        .arg("start")
        .arg("backend.service")
        .status().await?;

    Ok(())
}

fn write_archive_to_disk(archive_bytes: &[u8], output_path: &str) -> std::io::Result<()> {
    let mut archive = tar::Archive::new(archive_bytes);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let out_path = PathBuf::from(output_path).join(&path);

        if entry.header().entry_type().is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            entry.unpack(&out_path)?;
        }
    }
    Ok(())
}

fn checksum_is_valid(raw_archive: &[u8]) -> bool {
    let mut archive = tar::Archive::new(&raw_archive[..]);

    let mut checksum: Vec<u8> = Vec::new();
    let mut hash_sum = vec![0u8; 32];  

    match archive.entries() {
        Ok(entries) => {
            for file in entries {
                let file = match file {
                    Ok(f) => f,
                    Err(_) => return false,
                };

                let file_path = match file.path() {
                    Ok(p) => String::from(p.to_string_lossy()),
                    Err(_) => return false,
                };

                if file_path == "update_hash" {
                    checksum = raw_archive[file.raw_file_position() as usize..file.raw_file_position() as usize + file.size() as usize].to_vec();
                    continue;
                }
                let raw_data = raw_archive[file.raw_file_position() as usize..file.raw_file_position() as usize + file.size() as usize].to_vec();
                let hash = Sha256::digest(&raw_data);
                hash_sum = xor_hashes(&hash_sum, &hash);
            }    
        },
        Err(_) => return false,
    }

    hash_sum == checksum
}

pub fn xor_hashes(hash1: &[u8], hash2: &[u8]) -> Vec<u8> {
    hash1.iter().zip(hash2.iter()).map(|(a, b)| a ^ b).collect()
}