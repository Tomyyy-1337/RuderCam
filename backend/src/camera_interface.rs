use reqwest::Client;
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use crate::CONFIG;

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub enum Metering {
    Center,
    Average,
}

impl Metering {
    fn name(&self) -> &str {
        match self {
            Metering::Center => "centre",
            Metering::Average => "matrix",
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub enum FocusMode {
    Auto,
    Fixed,
}

impl FocusMode {
    fn name(&self) -> &str {
        match self {
            FocusMode::Auto => "auto",
            FocusMode::Fixed => "manual",
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
    pub rpiCameraMetering: &'static str,
    pub rpiCameraAfMode: &'static str,
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
        rpiCameraMetering: CONFIG.metering_mode.name(),
        rpiCameraAfMode: CONFIG.focus_mode.name(),
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