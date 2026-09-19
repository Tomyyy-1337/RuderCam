import { hasRequiredFields } from "./json_helpers";

export type OverlayPosition = 'top' | 'bottom'
export type AppTab = 'sessions' | 'settings' | 'camera'

const persistantStateFields = [
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
    "active_tab",
] as const;

export class PersistantState {
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
    active_tab: AppTab = $state<AppTab>("camera");

    updateFromJson(json: Record<string, unknown>): boolean {
        if (!hasRequiredFields(json, persistantStateFields)) {
            return false;
        }

        Object.assign(this, json);
        
        return true;
    }

    toJSONstring(): string {
        const state = Object.fromEntries(
            persistantStateFields.map((field) => [field, this[field]])
        );

        return JSON.stringify(state);
    }
}

export const persistant_state = new PersistantState();