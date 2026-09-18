import { hasRequiredFields } from "./json_helpers";

const activeSessionFields = [
    "client_time",
    "distance_traveled_km",
    "average_speed_kmh",
    "max_speed_kmh",
    "average_bpm",
    "duration_secs",
    "pausiert",
] as const;

export class ActiveSession {
    client_time = $state(0);
    distance_traveled_km = $state(0);
    average_speed_kmh = $state(0);
    max_speed_kmh = $state(0);
    average_bpm = $state(0);
    duration_secs = $state(0);
    pausiert = $state(true);

    updateFromJson(json: Record<string, unknown>): boolean {
        if (!hasRequiredFields(json, activeSessionFields)) {
            return false;
        }

        this.client_time = Date.parse(String(json.client_time));
        this.distance_traveled_km = Number(json.distance_traveled_km);
        this.average_speed_kmh = Number(json.average_speed_kmh);
        this.max_speed_kmh = Number(json.max_speed_kmh);
        this.average_bpm = Number(json.average_bpm);
        this.duration_secs = Number(json.duration_secs);
        this.pausiert = Boolean(json.pausiert);
        return true;
    }
}

export const activeSession = new ActiveSession();