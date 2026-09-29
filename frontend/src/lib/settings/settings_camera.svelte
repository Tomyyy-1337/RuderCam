<div class="camera-panel">
    <div class="control-grid">
        <label class="control-field">
            <span class="control-label">Brennweite</span>
            <select id="camera-focal-length" name="camera-focal-length" bind:value={app_config.focal_length} onchange={saveFocalLength}>
                <option value={28}>28 mm</option>
                <option value={35}>35 mm</option>
                <option value={42}>42 mm</option>
            </select>
        </label>

        <label class="control-field">
            <span class="control-label">Fokus</span>
            <select id="camera-focus-mode" name="camera-focus-mode" bind:value={app_config.focus_mode} onchange={saveFocusMode}>
                <option value="Auto">Auto</option>
                <option value="Fixed">Fest</option>
            </select>
        </label>

        <label class="control-field">
            <span class="control-label">Messung</span>
            <select id="camera-metering-mode" name="camera-metering-mode" bind:value={app_config.metering_mode} onchange={saveMeteringMode}>
                <option value="Average">Gesamtbild</option>
                <option value="Center">Mitte</option>
            </select>
        </label>

        <label class="control-field">
            <span class="control-label">Stream</span>
            <select id="camera-bitrate" name="camera-bitrate" bind:value={app_config.bitrate} onchange={saveBitrate}>
                <option value={1600000}>1,6 MB/s</option>
                <option value={2400000}>2,4 MB/s</option>
                <option value={3200000}>3,2 MB/s</option>
                <option value={4000000}>4,0 MB/s</option>
                <option value={8000000}>8,0 MB/s</option>
            </select>
        </label>
    </div>

    <div class="toggle-row">
        <label class:inactive={!app_config.hdr_enabled} class="toggle-control">
            <span class="toggle-copy">
                <span class="control-label">HDR</span>
                <small>{app_config.hdr_enabled ? "Aktiv" : "Aus"}</small>
            </span>
            <input
                id="camera-hdr"
                name="camera-hdr"
                type="checkbox"
                bind:checked={app_config.hdr_enabled}
                onchange={saveHdrEnabled}
            />
            <span class="switch" aria-hidden="true"></span>
        </label>

        <label class:inactive={!persistant_state.auto_level} class="toggle-control">
            <span class="toggle-copy">
                <span class="control-label">Auto-Level</span>
                <small>{persistant_state.auto_level ? "Aktiv" : "Aus"}</small>
            </span>
            <input id="camera-auto-level" name="camera-auto-level" type="checkbox" bind:checked={persistant_state.auto_level} />
            <span class="switch" aria-hidden="true"></span>
        </label>
    </div>

    {#if !app_config.hdr_enabled}
        <section class="control-block exposure-slot exposure-control">
            <div class="control-heading">
                <span class="control-label">Belichtung</span>
                <output for="camera-exposure-compensation">
                    {app_config.exposure_compenstion >= 0 ? "+" : ""}{app_config.exposure_compenstion.toFixed(1)} EV
                </output>
            </div>
            <div class="slider-wrap">
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
        </section>
    {:else}
        <p class="exposure-slot status-note">Belichtung ist bei HDR nicht verfügbar.</p>
    {/if}

    <section class="control-block rotation-control">
        <div class="control-heading">
            <label for="camera-rotation-offset" class="control-label">Drehwinkel</label>
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
    </section>
</div>

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
    .camera-panel {
        --camera-line: color-mix(in srgb, var(--text) 16%, transparent);
        display: grid;
        gap: 0.5rem;
        padding: 0.65rem;
        color: var(--text);
        background: linear-gradient(145deg, color-mix(in srgb, var(--section-background) 88%, #263442), var(--background));
        border: 1px solid var(--camera-line);
        border-radius: 0.9rem;
        box-shadow: 0 0.75rem 2rem rgba(0, 0, 0, 0.2);
    }

    .control-heading,
    .rotation-legend {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .control-label,
    .status-note {
        font-size: 0.66rem;
        font-weight: 700;
        letter-spacing: 0.1em;
        text-transform: uppercase;
    }

    .control-grid,
    .toggle-row {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 0.45rem;
    }

    .control-field,
    .toggle-control,
    .control-block {
        min-width: 0;
        box-sizing: border-box;
        border: 1px solid var(--camera-line);
        background: color-mix(in srgb, var(--section-background) 72%, transparent);
        border-radius: 0.65rem;
    }

    .control-field {
        display: grid;
        gap: 0.25rem;
        padding: 0.45rem 0.5rem 0.5rem;
    }

    .control-label {
        color: color-mix(in srgb, var(--text) 62%, transparent);
        line-height: 1;
        white-space: nowrap;
    }

    .control-field select {
        width: 100%;
        height: 1.9rem;
        min-width: 0;
        margin: 0;
        padding: 0.25rem 1.6rem 0.25rem 0.4rem;
        border-color: color-mix(in srgb, var(--text) 14%, transparent);
        border-radius: 0.4rem;
        background-color: color-mix(in srgb, var(--background) 70%, var(--section-background));
        font-size: 0.82rem;
        font-weight: 600;
    }

    .toggle-control {
        display: flex;
        align-items: center;
        justify-content: space-between;
        min-height: 3.05rem;
        gap: 0.4rem;
        padding: 0.45rem 0.55rem;
        cursor: pointer;
        user-select: none;
        -webkit-tap-highlight-color: transparent;
    }

    .toggle-control.inactive {
        background: color-mix(in srgb, var(--background) 48%, transparent);
    }

    .toggle-copy {
        display: grid;
        gap: 0.25rem;
        min-width: 0;
    }

    .toggle-copy small {
        color: color-mix(in srgb, var(--text) 52%, transparent);
        font-size: 0.68rem;
        line-height: 1;
    }

    .toggle-control input {
        position: absolute;
        width: 1px;
        height: 1px;
        opacity: 0;
    }

    .switch {
        position: relative;
        flex: 0 0 auto;
        width: 2rem;
        height: 1.15rem;
        border-radius: 999px;
        background: color-mix(in srgb, var(--text) 18%, var(--background));
        transition: background-color 160ms ease;
    }

    .switch::after {
        position: absolute;
        top: 0.18rem;
        left: 0.18rem;
        width: 0.79rem;
        height: 0.79rem;
        border-radius: 50%;
        background: color-mix(in srgb, var(--text) 78%, transparent);
        content: "";
        transition: transform 160ms ease, background-color 160ms ease;
    }

    .toggle-control input:checked + .switch {
        background: color-mix(in srgb, var(--button-color) 78%, #1e91ff);
    }

    .toggle-control input:checked + .switch::after {
        background: var(--text);
        transform: translateX(0.85rem);
    }

    .toggle-control input:focus-visible + .switch {
        outline: 2px solid var(--button-color);
        outline-offset: 2px;
    }

    .control-block {
        margin: 0;
        padding: 0.5rem 0.6rem 0.45rem;
    }

    .exposure-slot {
        height: 4.9rem;
        box-sizing: border-box;
    }

    .control-heading {
        margin-bottom: 0.2rem;
    }

    output {
        color: var(--text);
        font-size: 0.78rem;
        font-variant-numeric: tabular-nums;
        font-weight: 700;
    }

    .slider-wrap {
        position: relative;
    }

    input[type="range"] {
        width: 100%;
        height: 1.35rem;
        margin: 0;
        padding: 0;
        appearance: none;
        -webkit-appearance: none;
        background: transparent;
        cursor: pointer;
        -webkit-tap-highlight-color: transparent;
    }

    input[type="range"]::-webkit-slider-runnable-track {
        height: 0.35rem;
        border-radius: 999px;
        background: color-mix(in srgb, var(--text) 18%, var(--section-background));
        box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--text) 12%, transparent);
    }

    input[type="range"]::-moz-range-track {
        height: 0.35rem;
        border-radius: 999px;
        background: color-mix(in srgb, var(--text) 18%, var(--section-background));
        box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--text) 12%, transparent);
    }

    input[type="range"]::-moz-range-progress {
        height: 0.35rem;
        border-radius: 999px;
        background: var(--button-color);
    }

    input[type="range"]::-webkit-slider-thumb {
        width: 1.05rem;
        height: 1.05rem;
        margin-top: -0.35rem;
        appearance: none;
        -webkit-appearance: none;
        border: 3px solid var(--section-background);
        border-radius: 50%;
        background: var(--button-color);
        box-shadow: 0 0 0 1px color-mix(in srgb, var(--button-color) 70%, white 30%);
    }

    input[type="range"]::-moz-range-thumb {
        width: 0.8rem;
        height: 0.8rem;
        border: 3px solid var(--section-background);
        border-radius: 50%;
        background: var(--button-color);
        box-shadow: 0 0 0 1px color-mix(in srgb, var(--button-color) 70%, white 30%);
    }

    input[type="range"]:focus-visible {
        outline: 2px solid color-mix(in srgb, var(--button-color) 88%, white 12%);
        outline-offset: 3px;
        border-radius: 0.4rem;
    }

    .slider-legend {
        display: grid;
        grid-template-columns: repeat(13, minmax(0, 1fr));
        height: 1.15rem;
        color: color-mix(in srgb, var(--text) 56%, transparent);
    }

    .slider-step {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.15rem;
        min-width: 0;
        font-size: 0.57rem;
        line-height: 1;
    }

    .slider-step i {
        display: block;
        width: 1px;
        height: 0.25rem;
        background: color-mix(in srgb, var(--text) 35%, transparent);
    }

    .slider-step.major i {
        height: 0.4rem;
        background: color-mix(in srgb, var(--text) 62%, transparent);
    }

    .slider-step b {
        font-weight: 600;
        white-space: nowrap;
    }

    .rotation-control {
        padding-bottom: 0.35rem;
    }

    .rotation-legend {
        color: color-mix(in srgb, var(--text) 56%, transparent);
        font-size: 0.62rem;
        font-variant-numeric: tabular-nums;
    }

    .status-note {
        display: flex;
        align-items: center;
        min-height: 4.9rem;
        box-sizing: border-box;
        margin: 0;
        padding: 0.5rem 0.6rem;
        color: color-mix(in srgb, var(--text) 58%, transparent);
        border: 1px dashed var(--camera-line);
        border-radius: 0.65rem;
        font-size: 0.6rem;
        letter-spacing: 0.06em;
        text-transform: none;
    }

    @media (max-width: 360px) {
        .camera-panel {
            padding: 0.5rem;
        }

        .control-field {
            padding-inline: 0.4rem;
        }

        .control-field select {
            font-size: 0.76rem;
        }
    }
</style>