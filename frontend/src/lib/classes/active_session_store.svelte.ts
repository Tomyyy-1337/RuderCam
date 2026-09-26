export class ActiveSession {
    distance_traveled_km: number = $state(0);
    duration_secs: number = $state(0);
    pausiert: boolean = $state(false);
    schlag_count: number = $state(0);

    updateFromMsgpack(record: Record<string, unknown>): void {
        Object.assign(this, record);
    }
}

export const activeSession = new ActiveSession();