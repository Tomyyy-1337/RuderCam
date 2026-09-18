import { hasRequiredFields } from "./json_helpers";

const highFrequencyUpdateFields = ["roll"] as const;

export class HighFrequencyUpdate {
    roll = $state(0);

    updateFromJson(json: Record<string, unknown>): boolean {
        if (!hasRequiredFields(json, highFrequencyUpdateFields)) {
            return false;
        }

        this.roll = Number(json.roll);
        return true;
    }
}

export const highFrequencyUpdate = new HighFrequencyUpdate();