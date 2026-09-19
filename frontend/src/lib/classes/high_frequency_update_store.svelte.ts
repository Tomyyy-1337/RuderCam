import { hasRequiredFields } from "./json_helpers";

const highFrequencyUpdateFields = ["roll"];

export class HighFrequencyUpdate {
    roll = $state(0);

    updateFromMsgpack(record: Record<string, unknown>): boolean {
        if (!hasRequiredFields(record, highFrequencyUpdateFields)) {
            return false;
        }
        Object.assign(this, record);
        return true;
    }
}

export const highFrequencyUpdate = new HighFrequencyUpdate();