import { hasRequiredFields } from "./json_helpers";

export type OverlayPosition = 'top' | 'bottom'

const overlaySettingsFields = [
    "show_overlay",
    "position",
    "show_speed",
    "show_split_time",
    "show_schlagzahl",
    "show_fahrtzeit",
    "show_distanz",
    "show_distanc_per_stroke",
    "auto_level",
    "rotation_offset",
] as const;

export class OverlaySettings {
    show_overlay: boolean = $state(true);
    position: OverlayPosition = $state<OverlayPosition>("top");
    show_speed: boolean = $state(true);
    show_split_time: boolean = $state(true);
    show_schlagzahl: boolean = $state(true);
    show_fahrtzeit: boolean = $state(true);
    show_distanz: boolean = $state(true);
    show_distanc_per_stroke: boolean = $state(true);
    auto_level: boolean = $state(false);
    rotation_offset: number = $state(0);

    updateFromJson(json: Record<string, unknown>): boolean {
        if (!hasRequiredFields(json, overlaySettingsFields)) {
            return false;
        }

        Object.assign(this, json);
        
        return true;
    }

    toJSONstring(): string {
        return JSON.stringify({
            show_overlay: this.show_overlay,
            position: this.position,
            show_speed: this.show_speed,
            show_split_time: this.show_split_time,
            show_schlagzahl: this.show_schlagzahl,
            show_fahrtzeit: this.show_fahrtzeit,
            show_distanz: this.show_distanz,
            show_distanc_per_stroke: this.show_distanc_per_stroke,
            auto_level: this.auto_level,
            rotation_offset: this.rotation_offset,
        });
    }
}

export const overlay_settings = new OverlaySettings();