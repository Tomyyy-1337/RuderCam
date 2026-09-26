export class HighFrequencyUpdate {
    roll = $state(0);

    updateFromMsgpack(record: Record<string, unknown>): void {
        Object.assign(this, record);
    }
}

export const highFrequencyUpdate = new HighFrequencyUpdate();