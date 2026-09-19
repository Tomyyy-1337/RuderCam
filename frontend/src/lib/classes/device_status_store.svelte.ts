import { decode } from "@msgpack/msgpack";
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

    updateFromMsgpack(data: Uint8Array): boolean {
        try {
            let record = decode(data) as Record<string, unknown>;
            if (!hasRequiredFields(record, deviceStatusFields)) {
                return false;
            }
            Object.assign(this, record);
            return true;
        } catch {
            return false;
        }
    }
}

export const deviceStatus = new DeviceStatus();