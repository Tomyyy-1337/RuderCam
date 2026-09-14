use std::{
    ffi::c_void,
    io::{Read, Write},
    net::TcpStream,
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

const PI_ADDR: &str = "192.168.50.1:4000";

#[cfg(windows)]
#[repr(C)]
struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

#[cfg(windows)]
#[repr(C)]
struct WlanInterfaceInfo {
    interface_guid: Guid,
    description: [u16; 256],
    state: i32,
}

#[cfg(windows)]
#[repr(C)]
struct WlanInterfaceInfoList {
    item_count: u32,
    current_index: u32,
    items: [WlanInterfaceInfo; 1],
}

#[cfg(windows)]
#[link(name = "wlanapi")]
unsafe extern "system" {
    fn WlanOpenHandle(
        client_version: u32,
        reserved: *mut c_void,
        negotiated_version: *mut u32,
        client_handle: *mut *mut c_void,
    ) -> u32;
    fn WlanEnumInterfaces(
        client_handle: *mut c_void,
        reserved: *mut c_void,
        interface_list: *mut *mut WlanInterfaceInfoList,
    ) -> u32;
    fn WlanScan(
        client_handle: *mut c_void,
        interface_guid: *const Guid,
        ssid: *const c_void,
        information_element_data: *const c_void,
        reserved: *mut c_void,
    ) -> u32;
    fn WlanFreeMemory(memory: *mut c_void);
    fn WlanCloseHandle(client_handle: *mut c_void, reserved: *mut c_void) -> u32;
}

pub enum FlashEvent {
    NetworksFound(Result<Vec<String>, String>),
    Log(String),
    Finished(Result<(), String>),
}

/// Scans for visible WLAN networks on a background thread using `netsh`, keeping only ones
/// Windows already has a saved profile for (connecting to unknown networks is not supported).
pub fn scan_networks_async(tx: Sender<FlashEvent>, refresh: bool) {
    thread::spawn(move || {
        if refresh && request_wlan_scan() {
            thread::sleep(Duration::from_secs(2));
        }
        let result = list_networks().map(|networks| {
            let mut networks: Vec<String> = networks.into_iter().filter(|ssid| has_saved_profile(ssid)).collect();
            networks.sort();
            networks
        }).and_then(|networks| {
            if networks.is_empty() {
                Err("No known WLAN networks were found nearby".to_string())
            } else {
                Ok(networks)
            }
        });
        let _ = tx.send(FlashEvent::NetworksFound(result));
    });
}

#[cfg(windows)]
fn request_wlan_scan() -> bool {
    unsafe {
        let mut negotiated_version = 0;
        let mut client_handle = std::ptr::null_mut();
        if WlanOpenHandle(2, std::ptr::null_mut(), &mut negotiated_version, &mut client_handle) != 0 {
            return false;
        }

        let mut interface_list = std::ptr::null_mut();
        if WlanEnumInterfaces(client_handle, std::ptr::null_mut(), &mut interface_list) != 0 {
            WlanCloseHandle(client_handle, std::ptr::null_mut());
            return false;
        }

        let list = &*interface_list;
        let mut requested = false;
        for index in 0..list.item_count as usize {
            let interface = list.items.as_ptr().add(index);
            requested |= WlanScan(
                client_handle,
                &(*interface).interface_guid,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null_mut(),
            ) == 0;
        }

        WlanFreeMemory(interface_list.cast());
        WlanCloseHandle(client_handle, std::ptr::null_mut());
        requested
    }
}

#[cfg(not(windows))]
fn request_wlan_scan() -> bool {
    false
}

/// Connects to `ssid` (which must already have a saved Windows profile) and uploads `archive_path` to the pi.
/// Returns a flag that can be set to `true` to cooperatively cancel the operation.
pub fn flash_async(tx: Sender<FlashEvent>, ssid: String, archive_path: PathBuf) -> Arc<AtomicBool> {
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_thread = cancel.clone();

    thread::spawn(move || {
        let cancel = cancel_thread;

        let _ = tx.send(FlashEvent::Log(format!("Connecting to \"{ssid}\"...")));

        if let Err(e) = connect(&ssid) {
            let _ = tx.send(FlashEvent::Finished(Err(e)));
            return;
        }

        if cancel.load(Ordering::Relaxed) {
            return;
        }

        if !wait_until_connected(&ssid, Duration::from_secs(20), &cancel) {
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            let _ = tx.send(FlashEvent::Finished(Err(
                "Timed out waiting for the WLAN connection to come up".into(),
            )));
            return;
        }

        let _ = tx.send(FlashEvent::Log("Connected. Uploading update to the pi...".into()));

        let mut last_err = String::new();
        for attempt in 1..=5 {
            if cancel.load(Ordering::Relaxed) {
                return;
            }

            match upload_archive(&archive_path) {
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

fn list_networks() -> Result<Vec<String>, String> {
    let output = Command::new("netsh")
        .args(["wlan", "show", "networks"])
        .output()
        .map_err(|e| format!("failed to run netsh: {e}"))?;

    if !output.status.success() {
        return Err("netsh wlan show networks failed".to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout);
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

    if networks.is_empty() {
        Err("No wireless networks were found".to_string())
    } else {
        Ok(networks)
    }
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
    let output = Command::new("netsh").args(["wlan", "show", "interfaces"]).output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines().find_map(|l| {
        let trimmed = l.trim_start();
        if trimmed.starts_with("SSID") && !trimmed.starts_with("BSSID") {
            trimmed.split_once(':').map(|(_, v)| v.trim().to_string())
        } else {
            None
        }
    })
}

/// Reconnects to `ssid` (using its existing saved profile) on a background thread, best-effort.
pub fn reconnect_async(ssid: String) {
    thread::spawn(move || {
        let _ = connect(&ssid);
    });
}

fn connect(ssid: &str) -> Result<(), String> {
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

fn wait_until_connected(ssid: &str, timeout: Duration, cancel: &AtomicBool) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if cancel.load(Ordering::Relaxed) {
            return false;
        }
        if let Ok(output) = Command::new("netsh").args(["wlan", "show", "interfaces"]).output() {
            let text = String::from_utf8_lossy(&output.stdout);
            // netsh's field labels ("State"/"Status", "connected"/"Verbunden", ...) are localized,
            // so match on the (untranslated) SSID field instead of the connection state text.
            let matching_ssid = text.lines().any(|l| {
                let trimmed = l.trim_start();
                trimmed.starts_with("SSID") && !trimmed.starts_with("BSSID") && trimmed.contains(ssid)
            });
            if matching_ssid {
                return true;
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
    false
}

fn upload_archive(path: &PathBuf) -> Result<(), String> {
    let data = std::fs::read(path).map_err(|e| format!("failed to read archive: {e}"))?;

    let mut stream = TcpStream::connect(PI_ADDR).map_err(|e| format!("failed to connect to pi: {e}"))?;
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
