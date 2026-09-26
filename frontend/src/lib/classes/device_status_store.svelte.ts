export class DeviceStatus {
    battery_percentage: number = $state(0);
    speed_kmh: number = $state(0);
    schlagzahl: number = $state(0);
    satellite_count: number = $state(0);

    updateFromMsgpack(record: Record<string, unknown>): void {
        Object.assign(this, record);
    }
}

export const deviceStatus = new DeviceStatus();