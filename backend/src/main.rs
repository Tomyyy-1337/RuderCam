mod gps_interface;
// mod bpm_fft;
mod i2c_interface;
mod axum_server;
mod shared;
mod pi_interface;
mod session;
mod hotspot;
mod camera_interface;
mod bpm_peak;

use crate::{camera_interface::CameraInterface, gps_interface::GPSPositionalData, hotspot::Hotspot, i2c_interface::I2CInterface, session::ActiveSession, shared::{Config, DeviceState, Global, HighFrequencyUpdate, InternalState}}; 

use futures::stream::StreamExt;
use tokio::{pin, runtime::LocalOptions, task};

/// Only use these static variables once they have been initialized by calling `initialize_statics()`, otherwise it will lead to undefined behavior.
/// Only use these static variables on the main thread, as they are not thread-safe. Accessing them from multiple threads will lead to undefined behavior.
/// Only access CONFIG, SHARED_STATE and INTERNAL_STATE variables through their provided methods (`modify` and `get`) 
static CONFIG: Global<Config> = Global::new(Config::new_uninitialized());
static SHARED_STATE: Global<DeviceState> = Global::new(DeviceState::default());
static HIGH_FREQUENCY_UPDATE: Global<HighFrequencyUpdate> = Global::new(HighFrequencyUpdate::default());
static INTERNAL_STATE: Global<InternalState> = Global::new(InternalState::new());
static I2C_INTERFACE: I2CInterface = I2CInterface::new_uninitialized();
static CAMERA_INTERFACE: Global<CameraInterface> = Global::new(CameraInterface::new());

static CURRENT_SESSION: Global<Option<ActiveSession>> = Global::new(None); 

fn main() {
    // Initialize the I2C interface, load config from EEPROM and initialize internal state
    // Not calling this function will lead to undefined behavior. 
    initialize_statics();
    
    tokio::runtime::Builder::new_current_thread()
        .max_blocking_threads(4)
        .enable_all()
        .build_local(LocalOptions::default())
        .unwrap()
        .block_on(async {
            task::spawn_local(read_accelerometer_task());
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
    
    #[cfg(target_os = "linux")]
    Hotspot::initialize(&CONFIG.ssid, &CONFIG.password);
    
    #[cfg(target_os = "linux")]
    let _ = CAMERA_INTERFACE.modify(|camera| camera.start_camera());
    
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
                if satellites < 5 {
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
async fn read_accelerometer_task() {
    let mut timer = tokio::time::interval(tokio::time::Duration::from_millis(50));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    // Has to be even length
    let mut orientation_history = [0.0f32; 60];
    let mut orientation_index = 0;

    let mut bpm_processor = bpm_peak::BpmPeak::default();

    for loop_counter in 0usize.. {
        // every 50ms
        timer.tick().await;

        let (_x,y,z) = I2C_INTERFACE.read_accelerometer_data().await;
        
        let roll = (y as f32).atan2(z as f32).to_degrees();
        orientation_history[orientation_index] = roll;
        orientation_index = (orientation_index + 1) % orientation_history.len();
        let average_roll = orientation_history.iter().sum::<f32>() / orientation_history.len() as f32;
        const SENSOR_ROLL_OFFSET: f32 = 2.2;

        HIGH_FREQUENCY_UPDATE.modify(|state| state.roll = average_roll + SENSOR_ROLL_OFFSET);

        // every 100ms 
        if loop_counter % 2 == 0 {
            bpm_processor.add_sample(roll as i16);
        }

        // every 500ms 
        if loop_counter % 10 == 0 {
            SHARED_STATE.modify(|state| state.set_schlagzahl(bpm_processor.get_current_schläge_pro_minute()));
            CURRENT_SESSION.modify_option(|session| session.update_bpm_data(bpm_processor.get_incremental_schläge()));
        }
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