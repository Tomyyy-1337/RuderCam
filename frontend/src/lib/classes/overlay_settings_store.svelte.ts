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
    show_overlay = $state(true);
    position = $state<OverlayPosition>("top");
    show_speed = $state(true);
    show_split_time = $state(true);
    show_schlagzahl = $state(true);
    show_fahrtzeit = $state(true);
    show_distanz = $state(true);
    show_distanc_per_stroke = $state(true);
    auto_level = $state(false);
    rotation_offset = $state(0);

    updateFromJson(json: Record<string, unknown>): boolean {
        if (!hasRequiredFields(json, overlaySettingsFields)) {
            return false;
        }

        this.show_overlay = Boolean(json.show_overlay);
        this.position = String(json.position) as OverlayPosition;
        this.show_speed = Boolean(json.show_speed);
        this.show_split_time = Boolean(json.show_split_time);
        this.show_schlagzahl = Boolean(json.show_schlagzahl);
        this.show_fahrtzeit = Boolean(json.show_fahrtzeit);
        this.show_distanz = Boolean(json.show_distanz);
        this.show_distanc_per_stroke = Boolean(json.show_distanc_per_stroke);
        this.auto_level = Boolean(json.auto_level);
        this.rotation_offset = Number(json.rotation_offset);
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