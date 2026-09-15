<article>
    <h4>Fokusmodus</h4>
    <p>Autofokus oder festen Fokus für die Kamera auswählen.</p>

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

    <div class="spacer"></div>
</article>

<article>
    <h4>Belichtungsmessung</h4>
    <p>Belichtungsmessung für die Kamera auswählen.</p>

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

    <div class="spacer"></div>
</article>

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
    import type { AppConfig, FocusMode, MeteringMode } from "../types";

    let { config }: { config: AppConfig } = $props();

    type DialogTone = "info" | "warning" | "danger";

    let focusMode = $state<FocusMode>("Fixed");
    let meteringMode = $state<MeteringMode>("Average");
    let dialogOpen = $state(false);
    let dialogTitle = $state("");
    let dialogMessage = $state("");
    let dialogTone = $state<DialogTone>("info");

    $effect(() => {
        focusMode = config.focus_mode;
        meteringMode = config.metering_mode;
    });

    function openDialog(title: string, message: string, tone: DialogTone): void {
        dialogTitle = title;
        dialogMessage = message;
        dialogTone = tone;
        dialogOpen = true;
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