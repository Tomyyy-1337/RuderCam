use crate::{gps_interface::GPSPositionalData};

pub struct ActiveSession {
    client_time: String, // Start time of the session according to the client's clock 
    start_time: std::time::Instant,
    distance_traveled_km: f32,
    last_gps_position: Option<GPSPositionalData>,
    pausiert: bool,
    gps_position_history: Vec<GPSPositionalData>,
    last_history_update: std::time::Instant,
    schlag_count: u32,
}

#[derive(serde::Serialize)]
pub struct FinishedSession {
    pub client_time: String,
    pub duration_secs: f32,
    pub distance_traveled_km: f32,
    pub average_speed_kmh: f32,
    pub average_bpm: u8,
    pub gps_positions: Vec<GPSPositionalData>,
}

#[derive(serde::Serialize)]
pub struct SessionInfo {
    distance_traveled_km: f32,
    duration_secs: f32,
    pausiert: bool,
    schlag_count: u32,
}

impl ActiveSession {
    pub fn new(client_time: String) -> Self {
        ActiveSession { 
            client_time, 
            start_time: std::time::Instant::now(),
            distance_traveled_km: 0.0,
            last_gps_position: None,
            pausiert: false,
            gps_position_history: Vec::new(),
            last_history_update: std::time::Instant::now(),
            schlag_count: 0,
        }
    }

    pub fn get_summary(&self) -> SessionInfo {
        SessionInfo {
            distance_traveled_km: self.distance_traveled_km,
            duration_secs: self.start_time.elapsed().as_secs_f32(),
            pausiert: self.pausiert,
            schlag_count: self.schlag_count,
        }
    }

    pub fn add_gps_data(&mut self, data: GPSPositionalData) {
        if let Some(last_position) = &self.last_gps_position {
            self.distance_traveled_km += Self::calculate_distance(last_position.lat, last_position.lon, data.lat, data.lon)
        };
        self.last_gps_position = Some(data);

        if self.last_history_update.elapsed() >= std::time::Duration::from_secs(30) {
            self.gps_position_history.push(data);
            self.last_history_update = std::time::Instant::now();
        }
    }

    pub fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f32 {
        let r = 6371.0; 
        let dlat = (lat2 - lat1).to_radians();
        let dlon = (lon2 - lon1).to_radians();
        let a = (dlat / 2.0).sin().powi(2) + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        (r * c) as f32 
    }

    pub fn update_bpm_data(&mut self, schläge_gesamt: u32) {
        self.schlag_count += schläge_gesamt;
    }

    pub fn finish(self) -> FinishedSession {
        let duration_secs = self.start_time.elapsed().as_secs_f32();
        let average_speed_kmh = self.distance_traveled_km / (duration_secs / 3600.0);
        let average_bpm = (self.schlag_count as f32 / duration_secs * 60.0) as u8;

        FinishedSession {
            average_bpm,
            client_time: self.client_time,
            duration_secs,
            distance_traveled_km: self.distance_traveled_km,
            average_speed_kmh,
            gps_positions: self.gps_position_history,
        }
    }
}