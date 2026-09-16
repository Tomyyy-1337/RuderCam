<SettingsCard title="Fokusmodus" description="Autofokus oder festen Fokus für die Kamera auswählen.">

    <select id="camera-focus-mode" name="camera-focus-mode" bind:value={focusMode} onchange={saveFocusMode}>
        <option value="Auto">Autofokus</option>
        <option value="Fixed">Fester Fokus</option>
    </select>

    <p class="hint">
        {#if focusMode === "Auto"}
            Die Kamera passt den Fokus automatisch an.
        {:else}
            Die Kamera bleibt auf Hyperfokalpunkt eingestellt. Nahe Objekte können unscharf erscheinen.
        {/if}
    </p>

</SettingsCard>

<SettingsCard title="Belichtungsmessung" description="Belichtungsmessung für die Kamera auswählen.">

    <select id="camera-metering-mode" name="camera-metering-mode" bind:value={meteringMode} onchange={saveMeteringMode}>
        <option value="Average">Durchschnitt</option>
        <option value="Center">Mitte</option>
    </select>

    <p class="hint">
        {#if meteringMode === "Average"}
            Die Kamera misst die Helligkeit über das gesamte Bild.
        {:else}
            Die Kamera gewichtet die Bildmitte stärker.
        {/if}
    </p>

</SettingsCard>

<SettingsCard title="Bitrate" description="Die Bitrate für den Livestream einstellen.">

    <select id="camera-bitrate" name="camera-bitrate" bind:value={config.bitrate} onchange={saveBitrate}>
        <option value={100000}>100kB/s</option>
        <option value={400000}>400kB/s</option>
        <option value={800000}>800kB/s</option>
        <option value={1200000}>1,2MB/s</option>
        <option value={1600000}>1,6MB/s</option>
        <option value={2000000}>2,0MB/s</option>
        <option value={2400000}>2,4MB/s</option>
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
    import type { AppConfig, FocusMode, MeteringMode } from "../types";

    let { config }: { config: AppConfig } = $props();

    type DialogTone = "info" | "warning" | "danger";

    let focusMode = $state<FocusMode>("Fixed");
    let meteringMode = $state<MeteringMode>("Average");
    let dialogOpen = $state(false);
    let dialogTitle = $state("");
    let dialogMessage = $state("");
    let dialogTone = $state<DialogTone>("info");
    let bitrate = $state<number>(1600000);

    $effect(() => {
        bitrate = config.bitrate;
    });

    $effect(() => {
        focusMode = config.focus_mode;
    });

    $effect(() => {
        meteringMode = config.metering_mode;
    });

    function openDialog(title: string, message: string, tone: DialogTone): void {
        dialogTitle = title;
        dialogMessage = message;
        dialogTone = tone;
        dialogOpen = true;
    }

    async function saveBitrate(): Promise<void> {
        const previousBitrate = config.bitrate;
        config.bitrate = bitrate;

        try {
            const response = await fetch("/api/set_bitrate", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({ bitrate: bitrate }),
            });

            if (!response.ok) {
                throw new Error("bitrate update failed");
            }

        } catch {
            config.bitrate = previousBitrate;
            bitrate = previousBitrate;
            openDialog("Speichern fehlgeschlagen", "Die Bitrate konnte nicht gespeichert werden.", "danger");
        }
    }

    async function saveFocusMode(): Promise<void> {
        const previousFocusMode = config.focus_mode;
        config.focus_mode = focusMode;

        try {
            const response = await fetch("/api/set_focus_mode", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({ focus_mode: focusMode }),
            });

            if (!response.ok) {
                throw new Error("focus mode update failed");
            }

        } catch {
            config.focus_mode = previousFocusMode;
            focusMode = previousFocusMode;
            openDialog("Speichern fehlgeschlagen", "Der Fokusmodus konnte nicht gespeichert werden.", "danger");
        }
    }

    async function saveMeteringMode(): Promise<void> {
        const previousMeteringMode = config.metering_mode;
        config.metering_mode = meteringMode;

        try {
            const response = await fetch("/api/set_metering_mode", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({ metering_mode: meteringMode }),
            });

            if (!response.ok) {
                throw new Error("metering mode update failed");
            }

        } catch {
            config.metering_mode = previousMeteringMode;
            meteringMode = previousMeteringMode;
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
</style>