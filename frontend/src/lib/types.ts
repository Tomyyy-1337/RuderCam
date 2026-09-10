export type Theme = 'light' | 'dark'
export type OverlayPosition = 'top' | 'bottom'
export type AppTab = 'sessions' | 'settings'
export type SessionButtonVariant = 'primary' | 'overlay'

export interface TableColumn {
    header: string;
    value: string | number;
}

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
    position: OverlayPosition;
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
    duration_secs: number;
    distance_traveled_km: number;
    max_speed_kmh: number;
    average_speed_kmh: number;
    average_bpm: number;
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

function isRecord(value: unknown): value is Record<string, unknown> {
    return !!value && typeof value === 'object'
}

export function isGpsPosition(value: unknown): value is GpsPosition {
    return isRecord(value)
        && typeof value.lat === 'number'
        && typeof value.lon === 'number'
        && (value.speed_kmh === undefined || typeof value.speed_kmh === 'number')
}

export function isOverlaySettings(value: unknown): value is OverlaySettings {
    return isRecord(value)
        && typeof value.show_overlay === 'boolean'
        && (value.position === 'top' || value.position === 'bottom')
        && typeof value.show_speed === 'boolean'
        && typeof value.show_split_time === 'boolean'
        && typeof value.show_schlagzahl === 'boolean'
        && typeof value.show_fahrtzeit === 'boolean'
        && typeof value.show_distanz === 'boolean'
        && typeof value.show_distanc_per_stroke === 'boolean'
}

export function isDeviceStateMessage(value: unknown): value is DeviceStateMessage {
    return isRecord(value)
        && typeof value.velocity === 'number'
        && typeof value.satellite_count === 'number'
        && typeof value.schlagzahl === 'number'
        && typeof value.battery_percentage === 'number'
}

export function isRunningSessionMessage(value: unknown): value is RunningSessionMessage {
    return isRecord(value)
        && typeof value.client_time === 'string'
        && typeof value.distance_traveled_km === 'number'
        && typeof value.average_speed_kmh === 'number'
        && typeof value.max_speed === 'number'
        && typeof value.average_bpm === 'number'
        && typeof value.duration_secs === 'number'
        && typeof value.pausiert === 'boolean'
}

export function isSessionJson(value: unknown): value is SessionJson {
    return isRecord(value)
        && typeof value.client_time === 'string'
        && typeof value.duration_secs === 'number'
        && typeof value.distance_traveled_km === 'number'
        && typeof value.max_speed_kmh === 'number'
        && typeof value.average_speed_kmh === 'number'
        && typeof value.average_bpm === 'number'
        && (value.gps_positions === undefined
            || (Array.isArray(value.gps_positions) && value.gps_positions.every(isGpsPosition)))
}

export function isAppConfig(value: unknown): value is AppConfig {
    return isRecord(value)
        && typeof value.ssid === 'string'
        && typeof value.password === 'string'
        && typeof value.auto_shutdown_time === 'number'
}