import { hasRequiredFields } from "./json_helpers";

const deviceStatusFields = [
    "battery_percentage",
    "speed_kmh",
    "schlagzahl",
    "satellite_count"
] as const;

export class DeviceStatus {
    battery_percentage: number = $state(0);
    speed_kmh: number = $state(0);
    schlagzahl: number = $state(0);
    satellite_count: number = $state(0);

    updateFromMsgpack(record: Record<string, unknown>): boolean {
        if (!hasRequiredFields(record, deviceStatusFields)) {
            return false;
        }
        Object.assign(this, record);
        return true;
    }
}

export const deviceStatus = new DeviceStatus();