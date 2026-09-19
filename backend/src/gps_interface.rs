use futures::Stream;
use serde::Serialize;
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio::io::{AsyncBufReadExt, BufReader};
use std::time::Duration;

const GPS_SERIAL_PORT: &str = "/dev/serial0";
const GPS_BAUD_RATE: u32 = 9600;

pub struct GPSModule {
    reader: Option<BufReader<SerialStream>>,
    line: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct GPSPositionalData {
    pub lat: f64,
    pub lon: f64,
    pub speed_kmh: f32,
}

#[derive(Debug, Clone)]
pub enum GPSMessage {
    GGA { satellites: u32 },
    RMC (GPSPositionalData)
}

impl GPSModule {
    pub fn new() -> Self {
        GPSModule {
            reader: None,
            line: String::new(),
        }
    }

    async fn connect(&mut self) {
        self.reader = None;
        while self.reader.is_none() {
            match tokio_serial::new(GPS_SERIAL_PORT, GPS_BAUD_RATE).open_native_async() {
                Ok(port) => self.reader = Some(BufReader::new(port)),
                Err(_e) => tokio::time::sleep(Duration::from_millis(2000)).await,
            }
        }
    }

    pub async fn messages_stream(&mut self) -> impl Stream<Item = GPSMessage> {
        async_stream::stream! {
            loop {
                if let Some(reader) = &mut self.reader {
                    self.line.clear();
                    
                    match reader.read_line(&mut self.line).await {
                        Ok(0) | Err(_) => self.reader = None,
                        Ok(_) => {
                            match Self::parse_line(&self.line) {
                                Some(msg) => yield msg,
                                None => continue,
                            }
                        }
                    }
                } else {
                    self.connect().await;
                }
            }
        }
    }    

    fn parse_line(line: &str) -> Option<GPSMessage> {
        let line = line.trim();
        if line.starts_with("$GNGGA") || line.starts_with("$GPGGA") {
            Self::parse_gga(line).map(|sats| GPSMessage::GGA { satellites: sats })
        } else if line.starts_with("$GNRMC") || line.starts_with("$GPRMC") {
            Self::parse_gprmc(line).map(|(lat, lon, speed)| GPSMessage::RMC (GPSPositionalData { lat, lon, speed_kmh: speed as f32}))
        } else {
            None
        }
    }
    
    fn parse_gprmc(line: &str) -> Option<(f64, f64, f64)> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 9 || parts[2] != "A" {
            return None;
        }
        let lat = Self::nmea_coord_to_decimal(parts[3], parts[4])?;
        let lon = Self::nmea_coord_to_decimal(parts[5], parts[6])?;
        let speed_knots: f64 = parts[7].parse().ok()?;
        let speed_kmh = speed_knots * 1.852;
        if speed_kmh > 255.0 {
            return None;
        }
        Some((lat, lon, speed_kmh))
    }

    fn parse_gga(line: &str) -> Option<u32> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 8 {
            return None;
        }
        let satelite_count: u32 = parts[7].parse().ok()?;
        if satelite_count > 255 {
            return None;
        }

        Some(satelite_count)
    }

    fn nmea_coord_to_decimal(coord: &str, dir: &str) -> Option<f64> {
        let coord = coord.trim();
        let dir = dir.trim();

        if coord.is_empty() || dir.is_empty() {
            return None; 
        }

        let degree_digits = match dir {
            "N" | "S" => 2,
            "E" | "W" => 3,
            _ => return None,
        };

        if coord.len() <= degree_digits {
            return None;
        }

        let (deg, min) = coord.split_at(degree_digits);
        let deg: f64 = deg.parse().ok()?;
        let min: f64 = min.parse().ok()?;
        let mut dec = deg + min / 60.0;
        if dir == "S" || dir == "W" { 
            dec = -dec; 
        }
        Some(dec)
    }
}