use std::process::{Command, Stdio};

#[cfg(target_os = "linux")]
use std::os::unix::process::CommandExt;

use futures::io;
use serde::{Deserialize, Serialize};

use crate::CONFIG;

pub struct CameraInterface {
    stream_process: Option<std::process::Child>,
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub enum Metering {
    Center,
    Average,
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub enum FocusMode {
    Auto,
    Fixed,
}

impl Metering {
    fn to_string(&self) -> &str {
        match self {
            Metering::Center => "centre",
            Metering::Average => "average",
        }
    }
}

impl FocusMode {
    fn to_arg(&self) -> &str {
        match self {
            FocusMode::Auto => "--autofocus-mode continuous",
            FocusMode::Fixed => "--lens-position default",
        }
    }
}

impl CameraInterface {
    pub const fn new() -> Self {
        CameraInterface {
            stream_process: None,
        }
    }

    fn focal_length_to_roi(focal_length: f32) -> String {
        const BASE_FOCAL_LENGTH: f32 = 28.0;

        let crop = BASE_FOCAL_LENGTH / focal_length;
        let offset = (1.0 - crop) / 2.0;

        format!("{offset:.6},{offset:.6},{crop:.6},{crop:.6}")
    }

    pub fn start_camera(&mut self) -> io::Result<()> {
        if self.stream_process.is_some() {
            return Ok(());
        }

        let cmd = format!(
            "/usr/bin/rpicam-vid -t 0 --inline --width 1920 --height 1080 --framerate 25 --intra 25 --hflip 1 --low-latency 1 {} --bitrate {} --metering {} {} --roi {} -o - | \
             /usr/bin/ffmpeg -fflags +genpts -flags low_delay -fflags nobuffer -f h264 -i - -c copy -f rtsp -rtsp_transport udp rtsp://127.0.0.1:8554/stream",
            if CONFIG.hdr_enabled { "--hdr".to_string() } else { format!("--ev {}", CONFIG.exposure_compenstion) },
            CONFIG.bitrate,
            CONFIG.metering_mode.to_string(),
            CONFIG.focus_mode.to_arg(),
            CameraInterface::focal_length_to_roi(CONFIG.focal_length as f32)
        );

        #[cfg(target_os = "linux")]
        {
        let child = unsafe {
            Command::new("/bin/bash")
                .arg("-c")
                .arg(cmd)
                .current_dir("/home/pi/tmp")
                .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::inherit())
                .pre_exec(|| {
                    if libc::setsid() == -1 {
                        return Err(std::io::Error::last_os_error());
                    }

                    Ok(())
                })
                .spawn()?
        };

        self.stream_process = Some(child);
        }
        Ok(())
    }

    pub fn stop_camera(&mut self) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
        if let Some(mut child) = self.stream_process.take() {
            let pid = child.id() as i32;

            unsafe {
                libc::kill(-pid, libc::SIGKILL);
            }

            let _ = child.wait();
        }
        }
        Ok(())
    }

    pub fn restart_camera(&mut self) {
        let _ = self.stop_camera();
        let _ = self.start_camera();
    }
}