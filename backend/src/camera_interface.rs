use std::process::{Command, Stdio};

use futures::io;

enum Metering {
    Center,
    Average,
}

impl Metering {
    fn to_string(&self) -> &str {
        match self {
            Metering::Center => "centre",
            Metering::Average => "average",
        }
    }
}

pub struct CameraInterface {
    metering: Metering,
    exposure_compensation: i32,   
    stream_process: Option<std::process::Child>,
}

impl CameraInterface {
    pub fn new() -> Self {
        CameraInterface {
            metering: Metering::Center,
            exposure_compensation: 0,
            stream_process: None,
        }
    }

    pub fn start_camera(&mut self) -> io::Result<()> {
        if self.stream_process.is_some() {
            return Ok(());
        }

        let cmd = format!(
            "/usr/bin/rpicam-vid -t 0 --inline --width 1920 --height 1080 --framerate 25 --bitrate 2000000 --metering {} --ev {} -o - | \
             /usr/bin/ffmpeg -fflags +genpts -flags low_delay -fflags nobuffer -f h264 -i - -c copy -f rtsp -rtsp_transport udp rtsp://127.0.0.1:8554/stream",
            self.metering.to_string(),
            self.exposure_compensation
        );

        let child = Command::new("/bin/bash")
            .arg("-c")
            .arg(cmd)
            .current_dir("/home/pi/tmp")
            .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()?;

        self.stream_process = Some(child);
        Ok(())
    }

    pub fn stop_camera(&mut self) -> io::Result<()> {
        if let Some(mut child) = self.stream_process.take() {
            let _ = child.kill();
            let _ = child.wait();
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

    pub fn set_exposure_compensation(&mut self, compensation: i32) {
        self.exposure_compensation = compensation;
        self.restart_camera();
    }
}