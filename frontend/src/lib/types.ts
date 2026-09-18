export type Theme = 'light' | 'dark'
export type AppTab = 'sessions' | 'settings' | 'camera'
export type SessionButtonVariant = 'primary' | 'overlay'

export interface GpsPosition {
    lat: number;
    lon: number;
    speed_kmh: number;
}