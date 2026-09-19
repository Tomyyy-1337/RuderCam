import { hasRequiredFields } from "./json_helpers";

const highFrequencyUpdateFields = ["roll"];

export class HighFrequencyUpdate {
    roll = $state(0);

    updateFromJson(json: Record<string, unknown>): boolean {
        if (!hasRequiredFields(json, highFrequencyUpdateFields)) {
            return false;
        }

        Object.assign(this, json);
        
        return true;
    }
}

export const highFrequencyUpdate = new HighFrequencyUpdate();