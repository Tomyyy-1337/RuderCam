import { hasRequiredFields } from "./json_helpers";

export type FocusMode = 'Auto' | 'Fixed'
export type MeteringMode = 'Average' | 'Center'

const appConfigFields = [
    "ssid",
    "password",
    "auto_shutdown_time",
    "focus_mode",
    "metering_mode",
    "bitrate",
    "exposure_compenstion",
] as const;

export class AppConfig {
    ssid: string = $state("");
    password: string = $state("");
    auto_shutdown_time: number = $state(0);
    focus_mode: FocusMode = $state<FocusMode>("Auto");
    metering_mode: MeteringMode = $state<MeteringMode>("Average");
    bitrate: number = $state(0);
    exposure_compenstion: number = $state(0);

    updateFromJson(json: Record<string, unknown>): boolean {
        if (!hasRequiredFields(json, appConfigFields)) {
            return false;
        }

        Object.assign(this, json);
        
        return true;
    }

    async fetch(): Promise<void> {
        const response = await fetch("/api/get_config", {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
            },
        });

        if (response.ok) {
            const parsedConfig = await response.json();
            this.updateFromJson(parsedConfig);
        }
    }
}

export const app_config = new AppConfig();