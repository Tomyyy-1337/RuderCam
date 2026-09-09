use crate::{gps_interface::GPSPositionalData};

pub struct ActiveSession {
    client_time: String,
    start_time: std::time::Instant,
    distance_traveled_km: f32,
    average_speed_kmh: f32,
    max_speed: f32,
    bpm_sum: u32,
    bpm_count: u32,
    last_gps_position: Option<GPSPositionalData>,
    active_duration: std::time::Duration,
    last_gps_udpate: std::time::Instant,
    pausiert: bool,
    gps_position_history: Vec<GPSPositionalData>,
    time_since_last_history_update: std::time::Duration
}

#[derive(serde::Serialize)]
pub struct FinishedSession {
    pub client_time: String,
    pub duration_secs: f32,
    pub distance_traveled_km: f32,
    pub max_speed_kmh: f32,
    pub average_speed_kmh: f32,
    pub average_bpm: u8,
    pub gps_positions: Vec<GPSPositionalData>,
}

#[derive(serde::Serialize)]
pub struct SessionInfo {
    client_time: String,
    distance_traveled_km: f32,
    average_speed_kmh: f32,
    max_speed_kmh: f32,
    average_bpm: u8,
    duration_secs: f32,
    pausiert: bool,
}

impl ActiveSession {
    pub fn new(client_time: String) -> Self {
        ActiveSession { 
            client_time, 
            start_time: std::time::Instant::now(),
            distance_traveled_km: 0.0,
            average_speed_kmh: 0.0,
            bpm_sum: 0,
            bpm_count: 0,
            max_speed: 0.0,
            last_gps_position: None,
            active_duration: std::time::Duration::new(0, 0),
            last_gps_udpate: std::time::Instant::now(),
            pausiert: true,
            gps_position_history: Vec::new(),
            time_since_last_history_update: std::time::Duration::new(30, 0),
        }
    }

    pub fn get_summary(&self) -> SessionInfo {
        SessionInfo {
            client_time: self.client_time.clone(),
            distance_traveled_km: self.distance_traveled_km,
            average_speed_kmh: self.average_speed_kmh,
            max_speed_kmh: self.max_speed,
            average_bpm: self.average_bpm(),
            duration_secs: self.active_duration.as_secs_f32(),
            pausiert: self.pausiert,
        }
    }

    pub fn add_gps_data(&mut self, data: GPSPositionalData) {
        if data.speed_kmh > self.max_speed {
            self.max_speed = data.speed_kmh;
        }

        let last_update_time = self.last_gps_udpate;
        self.last_gps_udpate = std::time::Instant::now();
        
        self.pausiert = data.speed_kmh < 2.0;
        if !self.pausiert {
            if let Some(last_position) = &self.last_gps_position {
                self.distance_traveled_km += Self::calculate_distance(last_position.lat, last_position.lon, data.lat, data.lon)
            };

            let delta_time = self.last_gps_udpate.duration_since(last_update_time);

            self.active_duration += delta_time;
            self.time_since_last_history_update += delta_time;

            if self.time_since_last_history_update >= std::time::Duration::from_secs(30) {
                self.gps_position_history.push(data.clone());
                self.time_since_last_history_update = std::time::Duration::new(0, 0);
            }
        }

        self.average_speed_kmh = self.distance_traveled_km / (self.active_duration.as_secs_f32() / 3600.0);

        self.last_gps_position = Some(data);
    }

    pub fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f32 {
        let r = 6371.0; // Earth radius in kilometers
        let dlat = (lat2 - lat1).to_radians();
        let dlon = (lon2 - lon1).to_radians();
        let a = (dlat / 2.0).sin().powi(2) + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        (r * c) as f32 
    }

    pub fn add_bpm_data(&mut self, bpm: u8) {
        self.bpm_sum += bpm as u32;
        self.bpm_count += 1;
    }

    pub fn finish(self) -> FinishedSession {
        let duration_secs = self.start_time.elapsed().as_secs_f32();
        let average_speed_kmh = if duration_secs > 0.0 {
            self.distance_traveled_km / (duration_secs / 3600.0)
        } else {
            0.0
        };

        FinishedSession {
            average_bpm: self.average_bpm(),
            duration_secs: self.active_duration.as_secs_f32(),
            distance_traveled_km: self.distance_traveled_km,
            max_speed_kmh: self.max_speed,
            client_time: self.client_time,
            average_speed_kmh,
            // gps_positions: self.gps_position_history,
            gps_positions: vec![ 

                GPSPositionalData { lat: 49.3211515159501, lon: 8.447589581936127, speed_kmh: 10.0 },
                GPSPositionalData { lat: 49.323002757630334, lon: 8.44853694800425, speed_kmh: 12.0 },
                GPSPositionalData { lat: 49.32881255443851, lon: 8.452414421410182, speed_kmh: 15.0 }, 
                GPSPositionalData { lat: 49.33533692926359, lon: 8.460190512457913, speed_kmh: 20.0 },
                GPSPositionalData { lat: 49.341353790023916, lon: 8.471708848852389, speed_kmh: 25.0 },
                GPSPositionalData { lat: 49.3521822855019, lon: 8.487309631066697, speed_kmh: 30.0 },
                GPSPositionalData { lat: 49.358397676789295, lon: 8.492992932500439, speed_kmh: 35.0 },
            ]
        }
    }
    
    fn average_bpm(&self) -> u8 {
        if self.bpm_count > 0 {
            (self.bpm_sum / self.bpm_count) as u8
        } else {
            0
        }
    }
}