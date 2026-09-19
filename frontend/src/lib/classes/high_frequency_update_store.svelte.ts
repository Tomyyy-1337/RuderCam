import { decode } from "@msgpack/msgpack";
import { hasRequiredFields } from "./json_helpers";

const highFrequencyUpdateFields = ["roll"];

export class HighFrequencyUpdate {
    roll = $state(0);

    updateFromMsgpack(data: Uint8Array): boolean {
        try {
            let record = decode(data) as Record<string, unknown>;
            if (!hasRequiredFields(record, highFrequencyUpdateFields)) {
                return false;
            }
            Object.assign(this, record);
            return true;
        } catch {
            return false;
        }
    }
}

export const highFrequencyUpdate = new HighFrequencyUpdate();