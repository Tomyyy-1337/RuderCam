<SettingsCard title="Brennweite" description="Die Brennweite der Kamera einstellen.">
    <select id="camera-focal-length" name="camera-focal-length" bind:value={app_config.focal_length} onchange={saveFocalLength}>
        <option value={28}>28mm</option>
        <option value={35}>35mm</option>
        <option value={42}>42mm</option>
    </select>
</SettingsCard>

<SettingsCard title="HDR" description="Aktiviere erhöhten Dynamikumfang für kontrastreiche Szenarien.">
    <label class:inactive={!app_config.hdr_enabled} class="toggle-card">
        <input
            id="camera-hdr"
            name="camera-hdr"
            type="checkbox"
            bind:checked={app_config.hdr_enabled}
            onchange={saveHdrEnabled}
        />
        <span>HDR aktivieren</span>
    </label>
    {#if app_config.hdr_enabled}
        <p class="hint">
            Die Belichtungskorrektur ist nicht verfügbar wenn HDR aktiviert ist.
        </p>
    {/if}
</SettingsCard>

{#if !app_config.hdr_enabled}
<SettingsCard title="Belichtungskorrektur" description="Die Belichtungskorrektur für die Kamera einstellen.">
    <div class="slider-wrap">
        <div class="slider-value" aria-hidden="true">
            {app_config.exposure_compenstion >= 0 ? "+" : ""}{app_config.exposure_compenstion.toFixed(1)} EV
        </div>
        <input
            id="camera-exposure-compensation"
            name="camera-exposure-compensation"
            type="range"
            min="-3"
            max="3"
            step="0.5"
            bind:value={app_config.exposure_compenstion}
            onchange={saveExposureCompensation}
            aria-label="Belichtungskorrektur"
        />
        <div class="slider-legend" aria-hidden="true">
            {#each exposureSteps as value, index}
                <span class:major={index % 2 === 0} class="slider-step">
                    <i></i>
                    {#if index % 2 === 0}
                        <b>{value > 0 ? "+" : ""}{value}</b>
                    {/if}
                </span>
            {/each}
        </div>
    </div>
</SettingsCard>

<SettingsCard title="Belichtungsmessung" description="Belichtungsmessung für die Kamera auswählen.">
    <select id="camera-metering-mode" name="camera-metering-mode" bind:value={app_config.metering_mode} onchange={saveMeteringMode}>
        <option value="Average">Durchschnitt</option>
        <option value="Center">Mitte</option>
    </select>

    <p class="hint">
        {#if app_config.metering_mode === "Average"}
            Die Kamera misst die Helligkeit über das gesamte Bild.
        {:else}
            Die Kamera gewichtet die Bildmitte stärker.
        {/if}
    </p>
</SettingsCard>
{/if}

<SettingsCard title="Bildausrichtung" description="Das Kamerabild automatisch ausrichten oder manuell drehen.">
    <label class:inactive={!persistant_state.auto_level} class="toggle-card">
        <input id="camera-auto-level" name="camera-auto-level" type="checkbox" bind:checked={persistant_state.auto_level} />
        <span>Auto-Level</span>
    </label>

    <div class="rotation-slider">
        <div class="slider-header">
            <label for="camera-rotation-offset">Drehwinkel</label>
            <output for="camera-rotation-offset">{persistant_state.rotation_offset}°</output>
        </div>
        <input
            id="camera-rotation-offset"
            name="camera-rotation-offset"
            type="range"
            min="-10"
            max="10"
            step="0.5"
            bind:value={persistant_state.rotation_offset}
        />
        <div class="rotation-legend" aria-hidden="true">
            <span>-10°</span>
            <span>0°</span>
            <span>10°</span>
        </div>
    </div>

    <p class="hint">
        Der Drehwinkel hat auch im Automatikmodus Einfluss auf die Bildausrichtung.
    </p>
</SettingsCard>

<SettingsCard title="Fokusmodus" description="Autofokus oder festen Fokus für die Kamera auswählen.">
    <select id="camera-focus-mode" name="camera-focus-mode" bind:value={app_config.focus_mode} onchange={saveFocusMode}>
        <option value="Auto">Autofokus</option>
        <option value="Fixed">Fester Fokus</option>
    </select>

    <p class="hint">
        {#if app_config.focus_mode === "Auto"}
            Die Kamera passt den Fokus automatisch an.
        {:else}
            Die Kamera bleibt auf Hyperfokalpunkt eingestellt. Nahe Objekte können unscharf erscheinen.
        {/if}
    </p>
</SettingsCard>

<SettingsCard title="Bitrate" description="Die Bitrate für den Livestream einstellen.">

    <select id="camera-bitrate" name="camera-bitrate" bind:value={app_config.bitrate} onchange={saveBitrate}>
        <option value={1600000}>1,6MB/s</option>
        <option value={2400000}>2,4MB/s</option>
        <option value={3200000}>3,2MB/s (Standard)</option>
        <option value={4000000}>4,0MB/s</option>
        <option value={8000000}>8,0MB/s</option>
    </select>

    <p class="hint">
        Höhere Bitraten führen zu besserer Bildqualität aber können zu Verbindungsproblemen bei schlechter Netzwerkverbindung führen.
    </p>
    <p class="hint">
        Niedrigere Bitraten schonen die Bandbreite, verschlechtern jedoch die Bildqualität.
    </p>
    
</SettingsCard>

<Dialog
    bind:open={dialogOpen}
    title={dialogTitle}
    message={dialogMessage}
    confirmLabel="OK"
    showCancel={false}
    tone={dialogTone}
/>

<script lang="ts">
    import Dialog from "../components/dialog.svelte";
    import SettingsCard from "./settings_card.svelte";
    import { persistant_state } from "../classes/persistant_state_store.svelte";
    import { app_config } from "../classes/app_config.svelte";

    type DialogTone = "info" | "warning" | "danger";

    let dialogOpen = $state(false);
    let dialogTitle = $state("");
    let dialogMessage = $state("");
    let dialogTone = $state<DialogTone>("info");
    const exposureSteps = [-3, -2.5, -2, -1.5, -1, -0.5, 0, 0.5, 1, 1.5, 2, 2.5, 3];

    async function saveFocalLength(): Promise<void> {
        try {
            const response = await fetch("/api/set_focal_length", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({ focal_length: app_config.focal_length }),
            });

            if (!response.ok) {
                throw new Error("focal length update failed");
            }

        } catch {
            openDialog("Speichern fehlgeschlagen", "Die Brennweite konnte nicht gespeichert werden.", "danger");
        }
    }

    async function saveExposureCompensation(): Promise<void> {
        try {
            const response = await fetch("/api/set_exposure_compensation", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({ exposure_compensation: app_config.exposure_compenstion }),
            });

            if (!response.ok) {
                throw new Error("exposure compensation update failed");
            }

        } catch {
            openDialog("Speichern fehlgeschlagen", "Die Belichtungskorrektur konnte nicht gespeichert werden.", "danger");
        }
    }

    async function saveHdrEnabled(): Promise<void> {
        try {
            const response = await fetch("/api/set_hdr_enabled", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({ hdr_enabled: app_config.hdr_enabled }),
            });

            if (!response.ok) {
                throw new Error("HDR update failed");
            }

        } catch {
            openDialog("Speichern fehlgeschlagen", "HDR konnte nicht gespeichert werden.", "danger");
        }
    }

    function openDialog(title: string, message: string, tone: DialogTone): void {
        dialogTitle = title;
        dialogMessage = message;
        dialogTone = tone;
        dialogOpen = true;
    }

    async function saveBitrate(): Promise<void> {
        try {
            const response = await fetch("/api/set_bitrate", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({ bitrate: app_config.bitrate }),
            });

            if (!response.ok) {
                throw new Error("bitrate update failed");
            }

        } catch {
            openDialog("Speichern fehlgeschlagen", "Die Bitrate konnte nicht gespeichert werden.", "danger");
        }
    }

    async function saveFocusMode(): Promise<void> {
        try {
            const response = await fetch("/api/set_focus_mode", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({ focus_mode: app_config.focus_mode }),
            });

            if (!response.ok) {
                throw new Error("focus mode update failed");
            }

        } catch {
            openDialog("Speichern fehlgeschlagen", "Der Fokusmodus konnte nicht gespeichert werden.", "danger");
        }
    }

    async function saveMeteringMode(): Promise<void> {
        try {
            const response = await fetch("/api/set_metering_mode", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({ metering_mode: app_config.metering_mode }),
            });

            if (!response.ok) {
                throw new Error("metering mode update failed");
            }

        } catch {
            openDialog("Speichern fehlgeschlagen", "Die Belichtungsmessung konnte nicht gespeichert werden.", "danger");
        }
    }
</script>

<style>
    .hint {
        margin: 0.7rem 0 0;
        color: color-mix(in srgb, var(--text) 70%, transparent);
        font-size: 0.92rem;
        line-height: 1.45;
    }

    .toggle-card {
        display: flex;
        align-items: center;
        width: 100%;
        margin: 0.1rem;
        padding: 0.9rem 1rem;
        border-radius: 0.9rem;
        background: rgba(255, 255, 255, 0.06);
        border: 2px solid #0061c8;
        color: var(--text);
        cursor: pointer;
        user-select: none;
        gap: 0.75rem;
        box-sizing: border-box;
        -webkit-tap-highlight-color: transparent;
        outline: none;
    }

    .toggle-card input {
        position: absolute;
        opacity: 0;
        pointer-events: none;
        width: 0;
        height: 0;
        margin: 0;
        padding: 0;
    }

    .toggle-card span {
        flex: 1;
        font-weight: 600;
        letter-spacing: 0.01em;
    }

    .toggle-card.inactive {
        background: rgba(255, 255, 255, 0.03);
        border-color: rgba(255, 255, 255, 0.05);
        opacity: 0.55;
    }

    .rotation-slider {
        margin-top: 1rem;
    }

    .slider-header,
    .rotation-legend {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .slider-header {
        margin-bottom: 0.35rem;
        color: var(--text);
        font-weight: 600;
    }

    .slider-header output {
        color: color-mix(in srgb, var(--text) 72%, transparent);
        font-variant-numeric: tabular-nums;
    }

    .rotation-slider input[type="range"] {
        width: 100%;
        accent-color: var(--button-color);
        cursor: pointer;
    }

    .rotation-legend {
        color: color-mix(in srgb, var(--text) 62%, transparent);
        font-size: 0.75rem;
        font-variant-numeric: tabular-nums;
    }

    .slider-wrap {
        position: relative;
        width: calc(100% + 1.5rem);
        margin: 0 -0.75rem;
        padding: 0 0.75rem;
        box-sizing: border-box;
    }

    input[type="range"] {
        width: 100%;
        height: 1.5rem;
        margin: 0;
        padding: 0;
        appearance: none;
        -webkit-appearance: none;
        background: transparent;
        cursor: pointer;
        -webkit-tap-highlight-color: transparent;
    }

    input[type="range"]::-webkit-slider-runnable-track {
        height: 0.45rem;
        border-radius: 999px;
        background: color-mix(in srgb, var(--text) 18%, var(--section-background));
        box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--text) 12%, transparent);
    }

    input[type="range"]::-moz-range-track {
        height: 0.45rem;
        border-radius: 999px;
        background: color-mix(in srgb, var(--text) 18%, var(--section-background));
        box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--text) 12%, transparent);
    }

    input[type="range"]::-moz-range-progress {
        height: 0.45rem;
        border-radius: 999px;
        background: color-mix(in srgb, var(--text) 18%, var(--section-background));
    }

    input[type="range"]::-webkit-slider-thumb {
        width: 1.25rem;
        height: 1.25rem;
        margin-top: -0.4rem;
        appearance: none;
        -webkit-appearance: none;
        border: 3px solid var(--section-background);
        border-radius: 50%;
        background: var(--button-color);
        box-shadow: 0 0 0 1px color-mix(in srgb, var(--button-color) 70%, white 30%), 0 3px 10px rgba(0, 0, 0, 0.35);
    }

    input[type="range"]::-moz-range-thumb {
        width: 1rem;
        height: 1rem;
        border: 3px solid var(--section-background);
        border-radius: 50%;
        background: var(--button-color);
        box-shadow: 0 0 0 1px color-mix(in srgb, var(--button-color) 70%, white 30%), 0 3px 10px rgba(0, 0, 0, 0.35);
    }

    input[type="range"]:focus-visible {
        outline: 2px solid color-mix(in srgb, var(--button-color) 88%, white 12%);
        outline-offset: 4px;
        border-radius: 0.4rem;
    }

    .slider-value {
        display: block;
        margin: 0 0 0.25rem;
        color: var(--text);
        font-size: 1.8rem;
        font-weight: 700;
        line-height: 1.1;
        text-align: center;
        letter-spacing: 0.02em;
        pointer-events: none;
    }

    .slider-legend {
        display: grid;
        grid-template-columns: repeat(13, minmax(0, 1fr));
        height: 1.65rem;
        margin-top: 0.1rem;
        color: color-mix(in srgb, var(--text) 62%, transparent);
    }

    .slider-step {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.2rem;
        min-width: 0;
        font-size: 0.7rem;
        line-height: 1;
    }

    .slider-step i {
        display: block;
        width: 1px;
        height: 0.3rem;
        background: color-mix(in srgb, var(--text) 35%, transparent);
    }

    .slider-step.major i {
        height: 0.5rem;
        background: color-mix(in srgb, var(--text) 62%, transparent);
    }

    .slider-step b {
        font-weight: 600;
        white-space: nowrap;
    }
</style>