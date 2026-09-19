use axum::{Json, Router, extract::ws::{WebSocket, WebSocketUpgrade}, http::StatusCode, routing::{get, get_service, post}};
use lazy_static::lazy_static;
use regex::Regex;
use tower_http::{cors::CorsLayer, services::ServeDir};


use std::{net::SocketAddr, time::Duration};

use crate::{CAMERA_INTERFACE, CONFIG, CURRENT_SESSION, HIGH_FREQUENCY_UPDATE, I2C_INTERFACE, INTERNAL_STATE, SHARED_STATE, camera_interface::{FocusMode, Metering}, pi_interface, session::{ActiveSession, FinishedSession}, shared::Config};

lazy_static!(
    static ref PASSWORD_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9!@#$%^&*()_+\-=?]*$").expect("Failed to compile password regex");
);

#[derive(serde::Deserialize)]
struct WifiPasswordMessage {
    password: String,
}

#[derive(serde::Deserialize)]
struct WifiSSIDMessage {
    ssid: String,
}

#[derive(serde::Deserialize)]
struct UpdateShutdownTimerMessage {
    auto_shutdown_time: u64,
}

pub async fn start_server() {
    let app = Router::new()
        .route("/api/set_wifi_ssid", post(update_wifi_ssid))
        .route("/api/set_wifi_password", post(update_wifi_password))
        .route("/api/set_shutdown_timer", post(update_shutdown_timer))
        .route("/api/shutdown", get(shutdown_handler))
        .route("/api/get_config", get(get_config))
        .route("/api/reboot", get(reboot_handler))
        .route("/api/start_session", post(start_session_handler))
        .route("/api/stop_session", post(stop_session_handler))
        .route("/api/set_focus_mode", post(change_focus_mode))
        .route("/api/set_metering_mode", post(change_metering_mode))
        .route("/api/get_firmware_version", get(get_current_firmware_version))
        .route("/api/set_bitrate", post(change_bitrate))
        .route("/api/set_exposure_compensation", post(change_exposure_compensation))
        .route("/ws", get(websocket_handler))
        .nest_service("/maps", get_service(ServeDir::new("./maps")))    
        .fallback_service(ServeDir::new("./static").precompressed_gzip())
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running on http://127.0.0.1:3000");

    let listener = loop {
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => break listener,
            Err(e) => {
                println!("Failed to bind to {}: {}. Retrying...", addr, e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    };
    let _ = axum::serve(listener, app).await;
}

async fn get_current_firmware_version() -> String {
    let version = match std::fs::read_to_string("./version.txt") {
        Ok(content) => content.trim().to_string(),
        Err(_) => "No Version".to_string(),
    };
    version
}

#[derive(serde::Deserialize)]
struct ExposureCompensationMessage {
    exposure_compensation: f32,
}

async fn change_exposure_compensation(
    Json(payload): Json<ExposureCompensationMessage>,
) -> StatusCode {
    CONFIG.modify(|cfg| cfg.exposure_compenstion = payload.exposure_compensation);
    I2C_INTERFACE.write_config_to_eeprom().await;
    CAMERA_INTERFACE.modify(|camera| camera.restart_camera());
    StatusCode::OK
}

#[derive(serde::Deserialize)]
struct BitrateMessage {
    bitrate: u32,
}

async fn change_bitrate(
    Json(payload): Json<BitrateMessage>,
) -> StatusCode {
    CONFIG.modify(|cfg| cfg.bitrate = payload.bitrate);
    CAMERA_INTERFACE.modify(|camera| camera.restart_camera());
    I2C_INTERFACE.write_config_to_eeprom().await;
    StatusCode::OK
}

#[derive(serde::Deserialize)]
struct MeteringModeMessage {
    metering_mode: Metering,
}

async fn change_metering_mode(
    Json(payload): Json<MeteringModeMessage>,
) -> StatusCode {
    CONFIG.modify(|cfg| cfg.metering_mode = payload.metering_mode);
    I2C_INTERFACE.write_config_to_eeprom().await;
    CAMERA_INTERFACE.modify(|camera| camera.restart_camera());
    StatusCode::OK
}

#[derive(serde::Deserialize)]
struct FocusModeMessage {
    focus_mode: FocusMode,
}

async fn change_focus_mode(
    Json(payload): Json<FocusModeMessage>,
) -> StatusCode {
    CONFIG.modify(|cfg| cfg.focus_mode = payload.focus_mode);
    I2C_INTERFACE.write_config_to_eeprom().await;
    CAMERA_INTERFACE.modify(|camera| camera.restart_camera());
    StatusCode::OK
}

#[derive(serde::Deserialize)]
struct StartSessionMessage {
    client_time: String,
}

async fn start_session_handler(
    Json(payload): Json<StartSessionMessage>,
) -> axum::http::StatusCode {
    if CURRENT_SESSION.is_some() {
        // A session is already active, cannot start a new one
        return axum::http::StatusCode::BAD_REQUEST;
    }

    CURRENT_SESSION.modify(|session| {
        *session = Some(ActiveSession::new(payload.client_time));
    });

    axum::http::StatusCode::OK
}

async fn stop_session_handler() -> Result<Json<FinishedSession>, axum::http::StatusCode> {
    // If there is no active session, return 404
    if CURRENT_SESSION.is_none() {
        return Err(axum::http::StatusCode::NOT_FOUND);
    }

    // Take the active session and finish it
    let finished_session = CURRENT_SESSION.modify(|session| session.take().unwrap().finish());

    Ok(Json(finished_session))
}

async fn update_shutdown_timer(
    Json(payload): Json<UpdateShutdownTimerMessage>,
) -> axum::http::StatusCode {
    println!("Auto shutdown timer updated: {} minutes", payload.auto_shutdown_time);

    CONFIG.modify(|cfg| cfg.auto_shutdown_time = payload.auto_shutdown_time);
    I2C_INTERFACE.write_config_to_eeprom().await;
        
    axum::http::StatusCode::OK
}

async fn update_wifi_ssid(
    Json(payload): Json<WifiSSIDMessage>,
) -> axum::http::StatusCode {
    println!("Wifi SSID updated: SSID={}", payload.ssid);

    // #[cfg(target_os = "linux")]
    // if Hotspot::update_hotspot_credentials(&payload.ssid, &CONFIG.password).is_err() {
    //     println!("Failed to update hotspot credentials");
    //     return axum::http::StatusCode::INTERNAL_SERVER_ERROR;
    // }

    CONFIG.modify(|cfg| cfg.ssid = payload.ssid);
    I2C_INTERFACE.write_config_to_eeprom().await;

    axum::http::StatusCode::OK
}

async fn update_wifi_password(
    Json(payload): Json<WifiPasswordMessage>,
) -> axum::http::StatusCode 
{
    if !verify_password(&payload.password) {
        return axum::http::StatusCode::BAD_REQUEST;
    }

    // #[cfg(target_os = "linux")]
    // if Hotspot::update_hotspot_credentials(&CONFIG.ssid, &payload.password).is_err() {
    //     println!("Failed to update hotspot credentials");
    //     return axum::http::StatusCode::INTERNAL_SERVER_ERROR;
    // }
    
    println!("WiFi config updated: Password={}", payload.password);
    
    CONFIG.modify(|cfg| cfg.password = payload.password);
    I2C_INTERFACE.write_config_to_eeprom().await;

    axum::http::StatusCode::OK
}

fn verify_password(password: &str) -> bool {
    if password.len() < 8 || password.len() > 32 {
        return false;
    }
    
    PASSWORD_REGEX.is_match(password)
}

async fn get_config() -> Json<Config> {
    Json(CONFIG.clone())
}

async fn shutdown_handler() -> axum::http::StatusCode {
    println!("Shutdown command received, exiting...");

    tokio::task::spawn_blocking(move || {
        // Give the server a moment to send the response before shutting down
        std::thread::sleep(std::time::Duration::from_secs(3));

        pi_interface::shutdown();
    });

    axum::http::StatusCode::OK
}

async fn reboot_handler() -> axum::http::StatusCode {
    println!("Reboot command received, rebooting...");

    tokio::task::spawn_blocking(move || {
        // Give the server a moment to send the response before rebooting
        std::thread::sleep(std::time::Duration::from_secs(3));

        pi_interface::reboot();
    });

    axum::http::StatusCode::OK
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket))
}

async fn handle_socket(
    mut socket: WebSocket, 
) {
    let mut timer = tokio::time::interval(Duration::from_millis(50));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    INTERNAL_STATE.modify(|state| state.new_client_connected());

    for conter in 0usize.. {
        timer.tick().await;

        // Send high-frequency update data to the frontend every 50ms
        let json_data = serde_json::to_string(&*HIGH_FREQUENCY_UPDATE).unwrap();
        if socket.send(axum::extract::ws::Message::Text(json_data.into())).await.is_err() {
            INTERNAL_STATE.modify(|state| state.client_disconnected());
            break;
        }
        
        // Skip sending the shared state and session summary for 19 out of 20 ticks (every 50ms)
        if conter % 20 != 0 {
            continue;
        }

        // Send the current shared state to the frontend every second
        let json_data = serde_json::to_string(&*SHARED_STATE).unwrap();
        if socket.send(axum::extract::ws::Message::Text(json_data.into())).await.is_err() {
            INTERNAL_STATE.modify(|state| state.client_disconnected());
            break;
        }

        // Send running session summary if a session is active
        if let Some(current_session) = &*CURRENT_SESSION {
            let session_summary = current_session.get_summary();
            let json_data = serde_json::to_string(&session_summary).unwrap();
            
            if socket.send(axum::extract::ws::Message::Text(json_data.into())).await.is_err() {
                INTERNAL_STATE.modify(|state| state.client_disconnected());
                break;
            }
        }
    }
}
