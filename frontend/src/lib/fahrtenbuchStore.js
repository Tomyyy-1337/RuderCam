export class Session {
    constructor(sessionJson) {
        this.client_time = new Date(sessionJson.client_time); 
        this.duration_secs = Number(sessionJson.duration_secs);
        this.distance_traveled_km = Number(sessionJson.distance_traveled_km);
        this.max_speed_kmh = Number(sessionJson.max_speed_kmh); 
        this.average_speed_kmh = Number(sessionJson.average_speed_kmh);
        this.average_bpm = Number(sessionJson.average_bpm);
        this.gps_positions = sessionJson.gps_positions || [];
    }

    toJSON() {
        return {
            client_time: this.client_time.toISOString(),
            duration_secs: this.duration_secs,
            distance_traveled_km: this.distance_traveled_km,
            max_speed_kmh: this.max_speed_kmh,
            average_speed_kmh: this.average_speed_kmh,
            average_bpm: this.average_bpm,
            gps_positions: this.gps_positions
        };
    }
}

export class FahrtenbuchStore {
    constructor() { 
        const raw = JSON.parse(localStorage.getItem('fahrtenbuch')) || [];
        this.sessionHistory = raw.map(item => new Session(item));
    }

    getSessions() {
        return this.sessionHistory;
    }

    length() {
        return this.sessionHistory.length;
    }

    addSession(session) {
        this.sessionHistory.push(session);
        if (this.sessionHistory.length > 20) {
            this.sessionHistory.shift(); 
        }

        localStorage.setItem('fahrtenbuch', JSON.stringify(this.sessionHistory));
    }

    clearHistory() {
        this.sessionHistory = [];
        localStorage.removeItem('fahrtenbuch');
    }

    deleteSession(index) {
        if (index < 0 || index >= this.sessionHistory.length) {
            return;
        }

        this.sessionHistory.splice(index, 1);
        localStorage.setItem('fahrtenbuch', JSON.stringify(this.sessionHistory));
    }
}
