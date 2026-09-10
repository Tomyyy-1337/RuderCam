export type Theme = 'light' | 'dark'

export interface DeviceStatus {
    isConnected: boolean;
    battery_percentage: number;
    speed_kmh: number;
    schlagzahl: number;
    satellite_count: number;
}

export interface ActiveSession {
    isActive: boolean;
    end_time: Date;
    client_time: number | null;
    distance_traveled_km: number;
    average_speed_kmh: number;
    max_speed_kmh: number;
    average_bpm: number;
    duration_secs: number;
    pausiert: boolean;
}

export interface OverlaySettings {
    show_overlay: boolean;
    position: string;
    show_speed: boolean;
    show_split_time: boolean;
    show_schlagzahl: boolean;
    show_fahrtzeit: boolean;
    show_distanz: boolean;
    show_distanc_per_stroke: boolean;
}

export interface GpsPosition {
    lat: number;
    lon: number;
    speed_kmh?: number;
}

export interface ProjectedGpsPosition extends GpsPosition {
    x: number;
    y: number;
    speed_kmh: number;
}

export interface SessionJson {
    client_time: string;
    duration_secs: number | string;
    distance_traveled_km: number | string;
    max_speed_kmh: number | string;
    average_speed_kmh: number | string;
    average_bpm: number | string;
    gps_positions?: GpsPosition[];
}

export interface DeviceStateMessage {
    velocity: number;
    satellite_count: number;
    schlagzahl: number;
    battery_percentage: number;
}

export interface RunningSessionMessage {
    client_time: string;
    distance_traveled_km: number;
    average_speed_kmh: number;
    max_speed: number;
    average_bpm: number;
    duration_secs: number;
    pausiert: boolean;
}

export interface AppConfig {
    ssid: string;
    password: string;
    auto_shutdown_time: number;
}

export type Waypoint = [number, number]
export type WaypointInput = Waypoint | GpsPosition