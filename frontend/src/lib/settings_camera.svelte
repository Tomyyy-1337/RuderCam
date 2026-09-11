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
            Die Kamera bleibt auf eine feste Entfernung eingestellt.
        {/if}
    </p>

    <div class="spacer"></div>
</article>

<script lang="ts">
    import { onMount } from "svelte";

    type FocusMode = "Auto" | "Fixed";

    interface FocusModeMessage {
        focus_mode: FocusMode;
    }

    let focusMode = $state<FocusMode>("Fixed");

    onMount(() => {
        void fetchFocusMode();
    });

    async function fetchFocusMode(): Promise<void> {
        try {
            const response = await fetch("/api/get_focus_mode", {
                method: "GET",
                headers: {
                    "Content-Type": "application/json",
                },
            });

            if (!response.ok) {
                return;
            }

            const payload: unknown = await response.json();
            if (isFocusModeMessage(payload)) {
                focusMode = payload.focus_mode;
            }
        } catch {
        }
    }

    async function saveFocusMode(): Promise<void> {
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

    function isFocusModeMessage(value: unknown): value is FocusModeMessage {
        return typeof value === "object"
            && value !== null
            && "focus_mode" in value
            && ((value as FocusModeMessage).focus_mode === "Auto" || (value as FocusModeMessage).focus_mode === "Fixed");
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