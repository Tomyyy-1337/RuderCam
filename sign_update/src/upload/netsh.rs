use std::{
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

pub fn list_networks() -> Result<Vec<String>, String> {
    let output = Command::new("netsh")
        .args(["wlan", "show", "networks"])
        .output()
        .map_err(|e| format!("failed to run netsh: {e}"))?;

    if !output.status.success() {
        return Err("netsh wlan show networks failed".to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let networks = parse_networks(&text);
    if networks.is_empty() {
        Err("No wireless networks were found".to_string())
    } else {
        Ok(networks)
    }
}

fn parse_networks(text: &str) -> Vec<String> {
    let mut networks = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("SSID") {
            if let Some(idx) = rest.find(':') {
                let name = rest[idx + 1..].trim().to_string();
                if !name.is_empty() && !networks.contains(&name) {
                    networks.push(name);
                }
            }
        }
    }
    networks
}

/// Whether Windows already has a saved profile (and therefore a stored password) for `ssid`.
pub fn has_saved_profile(ssid: &str) -> bool {
    Command::new("netsh")
        .args(["wlan", "show", "profiles", &format!("name={ssid}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// The SSID the WLAN interface is currently associated with, if any.
pub fn current_ssid() -> Option<String> {
    let output = Command::new("netsh")
        .args(["wlan", "show", "interfaces"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    parse_current_ssid(&text)
}

fn parse_current_ssid(text: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let trimmed = l.trim_start();
        if trimmed.starts_with("SSID") && !trimmed.starts_with("BSSID") {
            trimmed.split_once(':').map(|(_, v)| v.trim().to_string())
        } else {
            None
        }
    })
}

pub fn connect(ssid: &str) -> Result<(), String> {
    let output = Command::new("netsh")
        .args([
            "wlan",
            "connect",
            &format!("name={ssid}"),
            &format!("ssid={ssid}"),
        ])
        .output()
        .map_err(|e| format!("failed to run netsh connect: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "netsh wlan connect failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(())
}

pub fn wait_until_connected(ssid: &str, timeout: Duration, cancel: &AtomicBool) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if cancel.load(Ordering::Relaxed) {
            return false;
        }
        if let Ok(output) = Command::new("netsh")
            .args(["wlan", "show", "interfaces"])
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            // netsh's field labels ("State"/"Status", "connected"/"Verbunden", ...) are localized,
            // so match on the (untranslated) SSID field instead of the connection state text.
            let matching_ssid = text.lines().any(|l| {
                let trimmed = l.trim_start();
                trimmed.starts_with("SSID")
                    && !trimmed.starts_with("BSSID")
                    && trimmed.contains(ssid)
            });
            if matching_ssid {
                return true;
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
    false
}
