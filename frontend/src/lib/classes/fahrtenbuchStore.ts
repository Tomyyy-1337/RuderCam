import { writable } from 'svelte/store'
import { type GpsPosition } from '../classes/gps_position'

function parseStoredSessions(): Record<string, unknown>[] {
    const raw = localStorage.getItem('fahrtenbuch')

    if (!raw) {
        return []
    }

    try {
        const parsed: unknown = JSON.parse(raw)
        return Array.isArray(parsed) ? parsed : []
    } catch {
        return []
    }
}

export class Session {
    client_time: Date = new Date();
    duration_secs: number = 0;
    distance_traveled_km: number = 0;
    max_speed_kmh: number = 0;
    average_speed_kmh: number = 0;
    average_bpm: number = 0
    gps_positions: GpsPosition[] = []

    fromJSON(json: Record<string, unknown>): Session {
        return Object.assign(new Session(), json)
    }
}

export class FahrtenbuchStore {
    sessionHistory: Session[];

    constructor() {
        this.sessionHistory = parseStoredSessions().map((item) => new Session().fromJSON(item))
    }

    getSessions(): Session[] {
        return this.sessionHistory
    }

    length(): number {
        return this.sessionHistory.length
    }

    addSession(session: Session): void {
        if (session.duration_secs <= 10) {
            return
        }

        this.sessionHistory.push(session)
        if (this.sessionHistory.length > 20) {
            this.sessionHistory.shift()
        }

        localStorage.setItem('fahrtenbuch', JSON.stringify(this.sessionHistory))
    }

    clearHistory(): void {
        this.sessionHistory = []
        localStorage.removeItem('fahrtenbuch')
    }

    deleteSession(index: number): void {
        if (index < 0 || index >= this.sessionHistory.length) {
            return
        }

        this.sessionHistory.splice(index, 1)
        localStorage.setItem('fahrtenbuch', JSON.stringify(this.sessionHistory))
    }
}

export const fahrtenbuch = writable(new FahrtenbuchStore())