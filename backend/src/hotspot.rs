use std::process::{Command, Stdio};

pub struct Hotspot;

impl Hotspot{
    const HOTSPOT_IFACE: &str = "wlan0";
    const HOTSPOT_PROFILE_NAME: &str = "treiber_hotspot";

    fn nmcli(args: &[&str]) -> Result<(), ()> {
        let output = Command::new("nmcli")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|_e| {()})?;
        
        if output.status.success() {
            Ok(())
        } else {
            Err(())
        }
    }

    fn nmcli_get(setting: &str) -> Option<String> {
        let output = Command::new("nmcli")
            .args(["-t", "-s", "-g", setting, "connection", "show", Self::HOTSPOT_PROFILE_NAME])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    }

    fn profile_matches(ssid: &str, password: &str) -> bool {
        let existing_ssid = Self::nmcli_get("802-11-wireless.ssid");
        let existing_password = Self::nmcli_get("802-11-wireless-security.psk");

        matches!(
            (existing_ssid, existing_password),
            (Some(existing_ssid), Some(existing_password))
                if existing_ssid == ssid && existing_password == password
        )
    }

    pub fn initialize(ssid: &str, password: &str) {
        if Self::profile_matches(ssid, password) {
            return;
        }

        let _ = Self::nmcli(&["connection", "down", Self::HOTSPOT_PROFILE_NAME]);

        let _ = Self::nmcli(&["connection", "delete", Self::HOTSPOT_PROFILE_NAME]);

        let _ = Self::nmcli(&[
            "connection",
            "add",
            "type",
            "wifi",
            "ifname",
            Self::HOTSPOT_IFACE,
            "con-name",
            Self::HOTSPOT_PROFILE_NAME,
            "autoconnect",
            "no",
            "ssid",
            ssid,
        ]);

        let _ = Self::nmcli(&[
            "connection",
            "modify",
            Self::HOTSPOT_PROFILE_NAME,
            "802-11-wireless.mode",
            "ap",
            "802-11-wireless.band",
            "bg",
            "ipv4.method",
            "shared",
            "ipv4.addresses",
            "192.168.50.1/24",
            "ipv6.method",
            "ignore",
            "wifi-sec.key-mgmt",
            "wpa-psk",
            "wifi-sec.psk",
            password,
        ]);
        
        let _ = Self::nmcli(&["connection", "up", Self::HOTSPOT_PROFILE_NAME]);
    }

}