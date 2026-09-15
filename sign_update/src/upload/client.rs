use std::{
    io::{Read, Write},
    net::TcpStream,
    path::PathBuf,
    time::Duration,
};

const PI_ADDR: &str = "192.168.50.1:4000";

pub fn upload_archive(path: &PathBuf) -> Result<(), String> {
    let data = std::fs::read(path).map_err(|e| format!("failed to read archive: {e}"))?;

    let mut stream =
        TcpStream::connect(PI_ADDR).map_err(|e| format!("failed to connect to pi: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(60)))
        .map_err(|e| e.to_string())?;

    let request_header = format!(
        "POST /api/update HTTP/1.1\r\nHost: {PI_ADDR}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        data.len()
    );

    stream
        .write_all(request_header.as_bytes())
        .map_err(|e| format!("failed to send request: {e}"))?;
    stream
        .write_all(&data)
        .map_err(|e| format!("failed to send archive: {e}"))?;

    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(|e| format!("failed to read response: {e}"))?;

    let response = String::from_utf8_lossy(&response);
    let status_line = response.lines().next().unwrap_or("");
    if status_line.contains("200") {
        Ok(())
    } else {
        Err(format!("pi rejected the update: {status_line}"))
    }
}
