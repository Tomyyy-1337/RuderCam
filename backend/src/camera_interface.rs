use reqwest::Client;
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use crate::CONFIG;

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub enum Metering {
    Center,
    Average,
}

impl std::fmt::Display for Metering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {    
        match self {
            Metering::Center => write!(f, "centre"),
            Metering::Average => write!(f, "matrix"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub enum FocusMode {
    Auto,
    Fixed,
}

impl std::fmt::Display for FocusMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FocusMode::Auto => write!(f, "auto"),
            FocusMode::Fixed => write!(f, "manual"),
        }
    }
}

#[allow(dead_code)]
fn focal_length_to_roi(focal_length: f32) -> String {
    const BASE_FOCAL_LENGTH: f32 = 28.0;

    let crop = BASE_FOCAL_LENGTH / focal_length;
    let offset = (1.0 - crop) / 2.0;

    format!("{offset:.6},{offset:.6},{crop:.6},{crop:.6}")
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct CameraConfig {
    pub rpiCameraHDR: bool,
    pub rpiCameraEV: f32,
    pub rpiCameraMetering: String,
    pub rpiCameraAfMode: String,
    pub rpiCameraROI: String,
    pub rpiCameraBitrate: u32,
    pub rpiCameraFPS: f32,
    pub rpiCameraIDRPeriod: u32,
}

pub async fn update_camera_config() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let roi = focal_length_to_roi(CONFIG.focal_length as f32);

    let config = CameraConfig {
        rpiCameraHDR: CONFIG.hdr_enabled,
        rpiCameraEV: CONFIG.exposure_compenstion,
        rpiCameraMetering: CONFIG.metering_mode.to_string(),
        rpiCameraAfMode: CONFIG.focus_mode.to_string(),
        rpiCameraROI: roi,
        rpiCameraBitrate: CONFIG.bitrate,
        rpiCameraFPS: 25.0,
        rpiCameraIDRPeriod: 25,
    };
    internal_update_camera_config(&client, &config).await?;
    Ok(())
}


#[allow(dead_code)]
async fn internal_update_camera_config(
    client: &Client,
    config: &CameraConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client
        .patch("http://127.0.0.1:9997/v3/config/paths/patch/stream")
        .json(config)
        .send()
        .await?;

    let status = response.status();
    if status.is_success() {
        Ok(())
    } else {
        let body = response.text().await.unwrap_or_default();
        Err(format!("API error: {status}: {body}").into())
    }
}