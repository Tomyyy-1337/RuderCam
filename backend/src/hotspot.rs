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


    fn disable_wifi_client() {
        let _ = Self::nmcli(&[
            "connection",
            "modify",
            "netplan-wlan0-Internetz 2.4 GHz",
            "connection.autoconnect",
            "no",
        ]);

        let _ = Self::nmcli(&["connection", "down", "netplan-wlan0-Internetz 2.4 GHz"]);

        let _ = Self::nmcli(&["device", "set", Self::HOTSPOT_IFACE, "autoconnect", "no"]);
    }

    fn ensure_hotspot_profile(
        ssid: &str,
        password: &str,
    ) {
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
    }

    fn start_hotspot() {
        let _ = Self::nmcli(&["connection", "up", Self::HOTSPOT_PROFILE_NAME]);
    }

    pub fn update_hotspot_credentials(
        new_ssid: &str,
        new_password: &str,
    ) -> Result<(), ()> {
        Self::nmcli(&[
            "connection",
            "modify",
            Self::HOTSPOT_PROFILE_NAME,
            "802-11-wireless.ssid",
            new_ssid,
            "wifi-sec.psk",
            new_password,
        ])?;

        let _ = Self::nmcli(&["connection", "down", Self::HOTSPOT_PROFILE_NAME]);
        Self::nmcli(&["connection", "up", Self::HOTSPOT_PROFILE_NAME])?;

        Ok(())
    }

    pub fn initialize(ssid: &str, password: &str) {
        Self::disable_wifi_client();

        Self::ensure_hotspot_profile(ssid, password);

        Self::start_hotspot();
    }

}