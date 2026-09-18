import { hasRequiredFields } from "./json_helpers";

const deviceStatusFields = [
    "battery_percentage",
    "speed_kmh",
    "schlagzahl",
    "satellite_count",
] as const;

export class DeviceStatus {
    battery_percentage = $state(0);
    speed_kmh = $state(0);
    schlagzahl = $state(0);
    satellite_count = $state(0);

    updateFromJson(json: Record<string, unknown>): boolean {
        if (!hasRequiredFields(json, deviceStatusFields)) {
            return false;
        }

        this.battery_percentage = Number(json.battery_percentage);
        this.speed_kmh = Number(json.speed_kmh);
        this.schlagzahl = Number(json.schlagzahl);
        this.satellite_count = Number(json.satellite_count);
        return true;
    }
}

export const deviceStatus = new DeviceStatus();