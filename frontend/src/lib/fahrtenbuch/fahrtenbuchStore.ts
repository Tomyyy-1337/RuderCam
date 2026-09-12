import { isSessionJson, type GpsPosition, type SessionJson } from '../types'

function parseStoredSessions(): SessionJson[] {
    const raw = localStorage.getItem('fahrtenbuch')

    if (!raw) {
        return []
    }

    try {
        const parsed: unknown = JSON.parse(raw)
        return Array.isArray(parsed) ? parsed.filter(isSessionJson) : []
    } catch {
        return []
    }
}

export class Session {
    client_time: Date;
    duration_secs: number;
    distance_traveled_km: number;
    max_speed_kmh: number;
    average_speed_kmh: number;
    average_bpm: number;
    gps_positions: GpsPosition[];

    constructor(sessionJson: SessionJson) {
        this.client_time = new Date(sessionJson.client_time)
        this.duration_secs = Number(sessionJson.duration_secs)
        this.distance_traveled_km = Number(sessionJson.distance_traveled_km)
        this.max_speed_kmh = Number(sessionJson.max_speed_kmh)
        this.average_speed_kmh = Number(sessionJson.average_speed_kmh)
        this.average_bpm = Number(sessionJson.average_bpm)
        this.gps_positions = sessionJson.gps_positions || []
    }

    toJSON(): SessionJson {
        return {
            client_time: this.client_time.toISOString(),
            duration_secs: this.duration_secs,
            distance_traveled_km: this.distance_traveled_km,
            max_speed_kmh: this.max_speed_kmh,
            average_speed_kmh: this.average_speed_kmh,
            average_bpm: this.average_bpm,
            gps_positions: this.gps_positions,
        }
    }
}

export class FahrtenbuchStore {
    sessionHistory: Session[];

    constructor() {
        this.sessionHistory = parseStoredSessions().map((item) => new Session(item))
    }

    getSessions(): Session[] {
        return this.sessionHistory
    }

    length(): number {
        return this.sessionHistory.length
    }

    addSession(session: Session): void {
        if (session.duration_secs <= 3) {
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