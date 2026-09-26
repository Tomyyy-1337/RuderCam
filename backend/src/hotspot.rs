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

    fn nmcli_output(args: &[&str]) -> Result<String, ()> {
        let output = Command::new("nmcli")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|_| ())?;

        if !output.status.success() {
            return Err(());
        }

        Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end_matches(['\r', '\n'])
            .to_owned())
    }

    fn is_running(ssid: &str, password: &str) -> bool {
        let active = Self::nmcli_output(&[
            "-t",
            "-f", "NAME,DEVICE",
            "connection", "show", "--active",
        ])
        .map(|output| output.lines().any(|line| {
            line == format!("{}:{}", Self::HOTSPOT_PROFILE_NAME, Self::HOTSPOT_IFACE)
        }))
        .unwrap_or(false);

        if !active {
            return false;
        }

        let configured_ssid = Self::nmcli_output(&[
            "--show-secrets",
            "-g", "802-11-wireless.ssid",
            "connection", "show",
            Self::HOTSPOT_PROFILE_NAME,
        ]).ok();

        let configured_password = Self::nmcli_output(&[
            "--show-secrets",
            "-g", "802-11-wireless-security.psk",
            "connection", "show",
            Self::HOTSPOT_PROFILE_NAME,
        ]).ok();

        configured_ssid.as_deref() == Some(ssid)
            && configured_password.as_deref() == Some(password)
    }

    pub fn initialize(ssid: &str, password: &str) {
        if Self::is_running(ssid, password) {
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