mod gps_interface;
mod bpm_fft;
mod i2c_interface;
mod axum_server;
mod shared;
mod pi_interface;
mod session;
mod hotspot;
mod camera_interface;

use std::{sync::Mutex, thread::sleep};

use crate::{bpm_fft::FFTBPMDetector, gps_interface::GPSPositionalData, hotspot::Hotspot, i2c_interface::I2CInterface, session::ActiveSession, shared::{Config, DeviceState, Global, InternalState}}; 

use futures::stream::StreamExt;
use tokio::{pin, runtime::LocalOptions, task, time::MissedTickBehavior};

/// Only use these static variables once they have been initialized by calling `initialize_statics()`, otherwise it will lead to undefined behavior.
/// Only use these static variables on the main thread, as they are not thread-safe. Accessing them from multiple threads will lead to undefined behavior.
/// Only access CONFIG, SHARED_STATE and INTERNAL_STATE variables through their provided methods (`modify` and `get`) 
static CONFIG: Global<Config> = Global::new(Config::new_uninitialized());
static SHARED_STATE: Global<DeviceState> = Global::new(DeviceState::default());
static INTERNAL_STATE: Global<InternalState> = Global::new(InternalState::new());
static I2C_INTERFACE: I2CInterface = I2CInterface::new_uninitialized();

static CURRENT_SESSION: Global<Option<ActiveSession>> = Global::new(None); 

fn main() {
    // Initialize the I2C interface, load config from EEPROM and initialize internal state
    // Not calling this function will lead to undefined behavior. 
    initialize_statics();
    
    tokio::runtime::Builder::new_current_thread()
        .max_blocking_threads(2)
        .enable_all()
        .build_local(LocalOptions::default())
        .unwrap()
        .block_on(async {
            let (accelerometer_sender, accelerometer_receiver) = tokio::sync::mpsc::channel(100);

            task::spawn_local(read_accelerometer_task(accelerometer_sender));
            task::spawn_local(accelerometer_processing_task(accelerometer_receiver));
            task::spawn_local(read_battery_task());
            task::spawn_local(read_gps_task());
            task::spawn_local(idle_auto_shutdown());
            task::spawn_local(axum_server::start_server());

            futures::future::pending::<()>().await;
        });
}

/// Initialize the I2C interface, load the configuration from EEPROM and initialize the internal state.
/// This function must be called before any other operation that accesses the 
/// I2C interface, the configuration or the internal state, otherwise it will lead to undefined behavior.
fn initialize_statics() {
    // Initialize the I2C interface
    I2CInterface::initialize(&I2C_INTERFACE);

    // Read config from EEPROM or initialize it with default values
    if !I2C_INTERFACE.eeprom_initialized_blocking() {
        println!("First run detected, initializing EEPROM with default configuration");
        CONFIG.modify(|cfg| cfg.reset());
        I2C_INTERFACE.write_config_to_eeprom_blocking(&*CONFIG);
    } else {
        match I2C_INTERFACE.read_config_from_eeprom_blocking() {
            Some(cfg) => CONFIG.modify(|c| *c = cfg),
            None => {
                println!("Failed to read config from EEPROM, using default configuration");
                CONFIG.modify(|cfg| cfg.reset());
                I2C_INTERFACE.write_config_to_eeprom_blocking(&*CONFIG);
            }
        }
    }

    // Initialize Hotspot
    #[cfg(target_os = "linux")]
    Hotspot::initialize(&CONFIG.ssid, &CONFIG.password);

    INTERNAL_STATE.modify(|state| {
        state.set_last_connection_time(std::time::Instant::now());
    });
}

/// Periodically check if a keep-alive signal has been received from the frontend and 
/// shut down the device if the configured timeout has been exceeded. 
async fn idle_auto_shutdown() {
    let mut timer = tokio::time::interval(tokio::time::Duration::from_secs(30));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        timer.tick().await;

        let shudown_timeout = CONFIG.auto_shutdown_time;
        if shudown_timeout > 0 {
            if INTERNAL_STATE.active_client_count == 0 
            && INTERNAL_STATE.get_last_connection_time().elapsed().as_secs() / 60 >= shudown_timeout { 
                println!("No keep-alive received for {:?} minute, shutting down...", shudown_timeout);
                pi_interface::shutdown();
            }
        }
    }
}

/// Continuously read GPS messages and update the shared state 
async fn read_gps_task() {
    let mut gps_module = gps_interface::GPSModule::new();
    let gps_stream = gps_module.messages_stream().await;
    pin!(gps_stream);

    while let Some(message) = gps_stream.next().await {
        match message {
            gps_interface::GPSMessage::GGA { satellites } => {
                SHARED_STATE.modify(|state| state.satellite_count = satellites as u8);
                if satellites < 4 {
                    SHARED_STATE.modify(|state| state.set_velocity(0.0));
                }
            }
            gps_interface::GPSMessage::RMC (data @ GPSPositionalData { speed_kmh, .. }) => {
                if SHARED_STATE.satellite_count >= 4 {
                    SHARED_STATE.modify(|state| state.set_velocity(speed_kmh));
                    CURRENT_SESSION.modify_option(|session| session.add_gps_data(data));
                }
            }
        }
    }
}

/// Continuously read raw accelerometer data and send it through a channel to the BPM processing task
async fn read_accelerometer_task(
    accelerometer_sender: tokio::sync::mpsc::Sender<(i16, i16, i16)>,
) {
    let mut timer = tokio::time::interval(tokio::time::Duration::from_millis(5));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let mut last_values = (0i16, 0i16, 0i16);

    loop {
        timer.tick().await;

        let accelerometer_data = I2C_INTERFACE.read_accelerometer_data().await;

        if accelerometer_data != last_values {
            accelerometer_sender.send(accelerometer_data).await.unwrap_or_else(|e| {
                println!("Failed to send accelerometer data: {}", e);
            });
            last_values = accelerometer_data;
        } 

    }
}

/// BPM Processing Task 
/// Reveive raw accelerometer data and proccess it to calculate BPM, 
/// then update the shared state with the latest BPM value
async fn accelerometer_processing_task(
    mut accelerometer_receiver: tokio::sync::mpsc::Receiver<(i16, i16, i16)>,
) {
    static BPM_FFT: Mutex<FFTBPMDetector> = Mutex::new(FFTBPMDetector::new(50.0, 12.0));

    // let mut bpm_fft = bpm_fft::FFTBPMDetector::new(50.0, 12.0);
    let mut timer = tokio::time::interval(tokio::time::Duration::from_secs(1));
    timer.set_missed_tick_behavior(MissedTickBehavior::Delay);
    loop {
        timer.tick().await;

        {
            let mut bpm_fft = BPM_FFT.lock().unwrap();
            while let Ok((x, y, z)) = accelerometer_receiver.try_recv() {
                bpm_fft.add_accelerometer_data(x, y, z);
            }
        }

        let bpm = tokio::task::spawn_blocking(|| {
            let bpm_fft = BPM_FFT.lock().unwrap();
            bpm_fft.get_current_bpm()
        }).await.unwrap_or(0.0);
        
        SHARED_STATE.modify(|state| state.set_schlagzahl(bpm));
        CURRENT_SESSION.modify_option(|session| {
            if SHARED_STATE.schlagzahl > 0.0 {
                session.add_bpm_data(bpm as u8);
            }
        });
    }
}

/// Continuously read battery status and update shared state
async fn read_battery_task(
) {
    let mut timer = tokio::time::interval(tokio::time::Duration::from_secs(30));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        timer.tick().await;
        
        let battery_status = I2C_INTERFACE.read_battery_status().await;
        SHARED_STATE.modify(|state| {
            state.battery_percentage = battery_status.battery_level;
        });

        #[cfg(target_os = "linux")]
        if battery_status.battery_level <= 5 {
            println!("Battery critically low ({}%m, {}V), shutting down...", battery_status.battery_level, battery_status.voltage);
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            pi_interface::shutdown();
        }  
    }
}