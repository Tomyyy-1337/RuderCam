<article>
    <h4>Kamera Fokus</h4>
    <p>Autofokus oder festen Fokus für den Videostream auswählen.</p>

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
    <h4>Kamera Messung</h4>
    <p>Belichtungsmessung für den Videostream auswählen.</p>

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

<script lang="ts">
    import type { AppConfig, FocusMode, MeteringMode } from "../types";

    let { config }: { config: AppConfig } = $props();

    let focusMode = $state<FocusMode>("Fixed");
    let meteringMode = $state<MeteringMode>("Average");

    $effect(() => {
        focusMode = config.focus_mode;
        meteringMode = config.metering_mode;
    });

    async function saveFocusMode(): Promise<void> {
        config.focus_mode = focusMode;
        try {
            await fetch("/api/set_focus_mode", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({ focus_mode: focusMode }),
            });
        } catch {
        }
    }

    async function saveMeteringMode(): Promise<void> {
        config.metering_mode = meteringMode;
        try {
            await fetch("/api/set_metering_mode", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({ metering_mode: meteringMode }),
            });
        } catch {
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