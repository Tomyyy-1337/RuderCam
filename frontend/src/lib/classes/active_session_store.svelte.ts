import { hasRequiredFields } from "./json_helpers";

const activeSessionFields = [
    "distance_traveled_km",
    "average_speed_kmh",
    "max_speed_kmh",
    "average_bpm",
    "duration_secs",
    "pausiert",
] as const;

export class ActiveSession {
    distance_traveled_km: number = $state(0);
    average_speed_kmh: number = $state(0);
    max_speed_kmh: number = $state(0);
    average_bpm: number = $state(0);
    duration_secs: number = $state(0);
    pausiert: boolean = $state(true);

    updateFromJson(json: Record<string, unknown>): boolean {
        if (!hasRequiredFields(json, activeSessionFields)) {
            return false;
        }

        Object.assign(this, json);

        return true;
    }
}

export const activeSession = new ActiveSession();