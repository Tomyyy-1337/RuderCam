use std::{cell::UnsafeCell, mem::MaybeUninit, ops::Deref};

use crate::camera_interface::{FocusMode, Metering};


/// Data that is send to the frontend periodically
#[derive(serde::Serialize, Clone)]
pub struct DeviceState {
    pub velocity: f32,
    pub satellite_count: u8,
    pub schlagzahl: f32,
    pub battery_percentage: u8,
}

#[derive(serde::Serialize, Clone)]
pub struct HighFrequencyUpdate {
    pub roll: f32,
}

impl HighFrequencyUpdate {
    pub const fn default() -> Self {
        HighFrequencyUpdate {
            roll: 0.0,
        }
    }
}

impl DeviceState {
    pub const fn default() -> Self {
        DeviceState {
            velocity: 0.0,
            satellite_count: 0,
            schlagzahl: 0.0,
            battery_percentage: 95,
        }
    }

    pub fn set_schlagzahl(&mut self, schlagzahl: f32) {
        self.schlagzahl = if self.velocity > 0.0 {
            let new_schlagzahl = schlagzahl.round();
            if schlagzahl - new_schlagzahl >= 0.5 {
                new_schlagzahl + 0.5
            } else {
                new_schlagzahl
            } 
        } else {
            0.0
        };
    }

    pub fn set_velocity(&mut self, velocity: f32) {
        self.velocity = if velocity >= 2.0 {
            (velocity * 10.0).round() / 10.0
        } else {
            0.0
        };
    }
}


/// Configuration data that can be changed by the user through the frontend
/// This is stored in the EEPROM and loaded on startup, persisting user settings across reboots
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Config {
    pub ssid: String,
    pub password: String,
    pub auto_shutdown_time: u64, // in minutes
    pub focus_mode: FocusMode,
    pub metering_mode: Metering,
    pub bitrate: u32,
    pub exposure_compenstion: f32
}

impl Config {
    pub const fn new_uninitialized() -> Self {
        Config {
            ssid: String::new(),
            password: String::new(),
            auto_shutdown_time: 30, // default to 30 minutes
            focus_mode: FocusMode::Fixed,
            metering_mode: Metering::Average,
            bitrate: 1600000, 
            exposure_compenstion: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.ssid = String::from("Ruder Cam Beta");
        self.password = String::from("bootkamera");
        self.auto_shutdown_time = 30;
        self.focus_mode = FocusMode::Fixed;
        self.metering_mode = Metering::Average;
        self.bitrate = 1600000;
        self.exposure_compenstion = 0.0;
    } 
}

/// Internal tempory state that is not sent to the frontend
/// Lost on reboot
pub struct InternalState {
    pub active_client_count: usize,
    last_connection_time: MaybeUninit<std::time::Instant>,
}

impl InternalState {
    pub const fn new() -> Self {
        InternalState {
            active_client_count: 0,
            last_connection_time: MaybeUninit::uninit(),
        }
    }

    pub fn get_last_connection_time(&self) -> std::time::Instant {
        unsafe { self.last_connection_time.assume_init() }
    }

    pub fn set_last_connection_time(&mut self, time: std::time::Instant) {
        self.last_connection_time = MaybeUninit::new(time);
    }

    pub fn new_client_connected(&mut self) {
        self.active_client_count += 1;
    }

    pub fn client_disconnected(&mut self) {
        if self.active_client_count > 0 {
            self.active_client_count -= 1;
        }
        if self.active_client_count == 0 {
            self.set_last_connection_time(std::time::Instant::now());
        }
    }
}

pub struct Global<T>(UnsafeCell<T>);

impl<T> Global<T> {
    pub const fn new(value: T) -> Self {
        Global(UnsafeCell::new(value))
    }

    #[inline(always)]
    pub fn modify<U>(&self, f: impl FnOnce(&mut T) -> U) -> U {
        // println!("Thread id: {:?} - Modifying global value", std::thread::current().id());
        // SAFETY: This is safe as long as the caller ensures that there are no concurrent accesses to the same Global instance.
        let value = unsafe { &mut *self.0.get() };
        f(value)
    }
}

impl<T> Global<Option<T>> {
    #[inline(always)]
    pub fn modify_option(&self, f: impl FnOnce(&mut T)) {
        // println!("Thread id: {:?} - Modifying global option value", std::thread::current().id());
        // SAFETY: This is safe as long as the caller ensures that there are no concurrent accesses to the same Global instance.
        let value = unsafe { &mut *self.0.get() };
        if let Some(inner) = value {
            f(inner);
        }
    }
}


impl<T> Deref for Global<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: This is safe as long as the caller ensures that there are no concurrent accesses to the same Global instance.
        unsafe { &*self.0.get() }
    }
}

unsafe impl<T> Sync for Global<T> {}
