use std::process::{Command, Stdio};

#[cfg(target_os = "linux")]
use std::os::unix::process::CommandExt;

use futures::io;
use serde::{Deserialize, Serialize};

pub struct CameraInterface {
    metering: Metering, 
    focus_mode: FocusMode,
    stream_process: Option<std::process::Child>,
}

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
            metering: Metering::Average,
            stream_process: None,
            focus_mode: FocusMode::Fixed,
        }
    }

    pub fn start_camera(&mut self) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
        if self.stream_process.is_some() {
            return Ok(());
        }

        let cmd = format!(
            "/usr/bin/rpicam-vid -t 0 --inline --width 1920 --height 1080 --framerate 25 --hflip 1 --low-latency 1 --bitrate 2000000 --metering {} {} -o - | \
             /usr/bin/ffmpeg -fflags +genpts -flags low_delay -fflags nobuffer -f h264 -i - -c copy -f rtsp -rtsp_transport udp rtsp://127.0.0.1:8554/stream",
            self.metering.to_string(),
            self.focus_mode.to_arg()
        );

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

    pub fn set_metering(&mut self, metering: Metering) {
        self.metering = metering;
        self.restart_camera();
    }

    pub fn set_focus_mode(&mut self, focus_mode: FocusMode) {
        self.focus_mode = focus_mode;
        self.restart_camera();
    }

    pub fn get_focus_mode(&self) -> FocusMode {
        self.focus_mode
    }
}