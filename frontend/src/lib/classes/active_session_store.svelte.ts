import { hasRequiredFields } from "./json_helpers";

const activeSessionFields = [
    "distance_traveled_km",
    "duration_secs",
    "pausiert",
    "schlag_count",
] as const;

export class ActiveSession {
    distance_traveled_km: number = $state(0);
    duration_secs: number = $state(0);
    pausiert: boolean = $state(false);
    schlag_count: number = $state(0);

    updateFromMsgpack(record: Record<string, unknown>): boolean {
        if (!hasRequiredFields(record, activeSessionFields)) {
            return false;
        }
        Object.assign(this, record);
        return true;
    }
}

export const activeSession = new ActiveSession();