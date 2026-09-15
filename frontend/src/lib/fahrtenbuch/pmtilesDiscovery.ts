const PMTILES_MAGIC_NUMBER = 19792;
const PMTILES_VERIFY_TIMEOUT_MS = 3000;

const pmtilesFiles = [
    "germany.pmtiles",
    "france.pmtiles",
    "belgium.pmtiles",
    "netherlands.pmtiles",
    "poland.pmtiles",
    "denmark.pmtiles",
    "czech_republic.pmtiles",
    "luxembourg.pmtiles",
];

const verifyPmtilesFile = async (url: string): Promise<boolean> => {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), PMTILES_VERIFY_TIMEOUT_MS);
    try {
        const response = await fetch(url, {
            headers: { Range: "bytes=0-1" },
            signal: controller.signal
        });
        if (!response.ok) {
            return false;
        }

        const bytes = await response.arrayBuffer();
        if (bytes.byteLength < 2) {
            return false;
        }

        return new DataView(bytes).getUint16(0, true) === PMTILES_MAGIC_NUMBER;
    } catch {
        return false;
    } finally {
        clearTimeout(timeoutId);
    }
};

export const discoverPmtilesBaseUrl = async (): Promise<{ baseUrl: string; validFiles: string[] }> => {
    const origins = [...new Set([
        window.location.origin,
        `${window.location.protocol}//${window.location.hostname}:3000`
    ])];

    for (const origin of origins) {
        const baseUrl = `${origin}/maps`;
        const validFiles = (
            await Promise.all(
                pmtilesFiles.map(async (filename) => {
                    const isValid = await verifyPmtilesFile(`${baseUrl}/${filename}`);
                    return isValid ? filename : null;
                })
            )
        ).filter((filename): filename is string => filename !== null);

        if (validFiles.length > 0) {
            return { baseUrl, validFiles };
        }
    }

    return { baseUrl: `${origins[0]}/maps`, validFiles: [] };
};
