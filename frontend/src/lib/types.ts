export type Theme = 'light' | 'dark'
export type AppTab = 'sessions' | 'settings' | 'camera'
export type SessionButtonVariant = 'primary' | 'overlay'

export interface FrontendState {
    is_connected: boolean;
    session_is_active: boolean;
}

export interface GpsPosition {
    lat: number;
    lon: number;
    speed_kmh: number;
}