mod client;
mod netsh;
mod windows_scan;

use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
    thread,
    time::Duration,
};

const LAST_NETWORK_PATH: &str = "last_network.txt";

static LAST_NETWORK_SSID: std::sync::OnceLock<std::sync::Mutex<Option<String>>> =
    std::sync::OnceLock::new();
static AUTO_UPLOAD_ENABLED: AtomicBool = AtomicBool::new(false);

fn last_network_ssid() -> std::sync::MutexGuard<'static, Option<String>> {
    LAST_NETWORK_SSID
        .get_or_init(|| {
            let ssid = std::fs::read_to_string(LAST_NETWORK_PATH)
                .ok()
                .map(|ssid| ssid.trim().to_string())
                .filter(|ssid| !ssid.is_empty());
            std::sync::Mutex::new(ssid)
        })
        .lock()
        .unwrap()
}

fn prioritize_networks(mut networks: Vec<String>, preferred: Option<&str>) -> Vec<String> {
    if let Some(preferred) = preferred {
        if let Some(index) = networks.iter().position(|ssid| ssid == preferred) {
            let preferred_network = networks.remove(index);
            networks.insert(0, preferred_network);
        }
    }
    networks
}

pub fn remember_last_network(ssid: &str) {
    let mut last = last_network_ssid();
    if last.as_deref() != Some(ssid) {
        *last = Some(ssid.to_string());
        let _ = std::fs::write(LAST_NETWORK_PATH, ssid);
    }
}

pub fn last_network() -> Option<String> {
    last_network_ssid().clone()
}

pub fn auto_upload_enabled() -> bool {
    AUTO_UPLOAD_ENABLED.load(Ordering::Relaxed)
}

pub fn toggle_auto_upload() -> bool {
    let enabled = !auto_upload_enabled();
    AUTO_UPLOAD_ENABLED.store(enabled, Ordering::Relaxed);
    enabled
}

pub enum FlashEvent {
    NetworksFound(Result<Vec<String>, String>),
    Log(String),
    Finished(Result<(), String>),
}

pub use netsh::{current_ssid, has_saved_profile};

/// Scans for visible WLAN networks on a background thread using `netsh`, keeping only ones
/// Windows already has a saved profile for (connecting to unknown networks is not supported).
pub fn scan_networks_async(tx: Sender<FlashEvent>, refresh: bool) {
    thread::spawn(move || {
        if refresh && windows_scan::request_wlan_scan() {
            thread::sleep(Duration::from_secs(2));
        }
        let result = netsh::list_networks()
            .map(|networks| {
                let mut networks: Vec<String> = networks
                    .into_iter()
                    .filter(|ssid| has_saved_profile(ssid))
                    .collect();
                networks.sort();
                prioritize_networks(networks, last_network().as_deref())
            })
            .and_then(|networks| {
                if networks.is_empty() {
                    Err("No known WLAN networks were found nearby".to_string())
                } else {
                    Ok(networks)
                }
            });
        let _ = tx.send(FlashEvent::NetworksFound(result));
    });
}

/// Connects to `ssid` (which must already have a saved Windows profile) and uploads `archive_path` to the pi.
/// Returns a flag that can be set to `true` to cooperatively cancel the operation.
pub fn flash_async(tx: Sender<FlashEvent>, ssid: String, archive_path: PathBuf) -> Arc<AtomicBool> {
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_thread = cancel.clone();

    thread::spawn(move || {
        let cancel = cancel_thread;

        let _ = tx.send(FlashEvent::Log(format!("Connecting to \"{ssid}\"...")));

        if let Err(e) = netsh::connect(&ssid) {
            let _ = tx.send(FlashEvent::Finished(Err(e)));
            return;
        }

        if cancel.load(Ordering::Relaxed) {
            return;
        }

        if !netsh::wait_until_connected(&ssid, Duration::from_secs(20), &cancel) {
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            let _ = tx.send(FlashEvent::Finished(Err(
                "Timed out waiting for the WLAN connection to come up".into(),
            )));
            return;
        }

        let _ = tx.send(FlashEvent::Log(
            "Connected. Uploading update to the pi...".into(),
        ));

        let mut last_err = String::new();
        for attempt in 1..=5 {
            if cancel.load(Ordering::Relaxed) {
                return;
            }

            match client::upload_archive(&archive_path) {
                Ok(()) => {
                    let _ = tx.send(FlashEvent::Finished(Ok(())));
                    return;
                }
                Err(e) => {
                    last_err = e;
                    let _ = tx.send(FlashEvent::Log(format!(
                        "Upload attempt {attempt}/5 failed: {last_err}. Retrying..."
                    )));

                    for _ in 0..20 {
                        if cancel.load(Ordering::Relaxed) {
                            return;
                        }
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        }

        let _ = tx.send(FlashEvent::Finished(Err(last_err)));
    });

    cancel
}

/// Reconnects to `ssid` (using its existing saved profile) on a background thread, best-effort.
pub fn reconnect_async(ssid: String) {
    thread::spawn(move || {
        let _ = netsh::connect(&ssid);
    });
}
