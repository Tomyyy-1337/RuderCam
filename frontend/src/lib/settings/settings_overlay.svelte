<article>
    <h4>Overlay Aktivieren/Deaktivieren</h4>
    <p>Overlay in der Vollbildansicht ein- oder ausblenden.</p>
    <select id="overlay-show-overlay" name="overlay-show-overlay" bind:value={overlay_settings.show_overlay}>
        <option value={true}>Overlay anzeigen</option>
        <option value={false}>Overlay ausblenden</option>
    </select>
    <div class="spacer"></div>
</article>

{#if overlay_settings.show_overlay}
<article>
    <h4>Overlay Position</h4>
    <p>Position des Overlays in der Vollbildansicht.</p>
    <select id="overlay-position" name="overlay-position" bind:value={overlay_settings.position}>
        <option value="top">Overlay Oben</option>
        <option value="bottom">Overlay Unten</option>
    </select>
    <div class="spacer"></div>
</article>

<article>
    <h4>Overlay Elemente</h4>
    <p>Verwalten welche Elemente im Overlay angezeigt werden sollen.</p>

    <label class:inactive={!overlay_settings.show_speed} class="toggle-card">
        <input id="overlay-show-speed" name="overlay-show-speed" type="checkbox" bind:checked={overlay_settings.show_speed} />
        <span>Geschwindigkeit (km/h)</span>
    </label>

    <label class:inactive={!overlay_settings.show_split_time} class="toggle-card">
        <input id="overlay-show-split-time" name="overlay-show-split-time" type="checkbox" bind:checked={overlay_settings.show_split_time} />
        <span>500m Split</span>
    </label>

    <label class:inactive={!overlay_settings.show_schlagzahl} class="toggle-card">
        <input id="overlay-show-schlagzahl" name="overlay-show-schlagzahl" type="checkbox" bind:checked={overlay_settings.show_schlagzahl} />
        <span>Schlagzahl</span>
    </label>

    <label class:inactive={!overlay_settings.show_distanc_per_stroke} class="toggle-card">
        <input id="overlay-show-distance-per-stroke" name="overlay-show-distance-per-stroke" type="checkbox" bind:checked={overlay_settings.show_distanc_per_stroke} />
        <span>Distanz pro Schlag</span>
    </label>
    
    <label class:inactive={!overlay_settings.show_distanz} class="toggle-card">
        <input id="overlay-show-distanz" name="overlay-show-distanz" type="checkbox" bind:checked={overlay_settings.show_distanz} />
        <span>Distanz</span>
    </label>

    <label class:inactive={!overlay_settings.show_fahrtzeit} class="toggle-card">
        <input id="overlay-show-fahrtzeit" name="overlay-show-fahrtzeit" type="checkbox" bind:checked={overlay_settings.show_fahrtzeit} />
        <span>Fahrtzeit</span>
    </label>

    <div class="spacer"></div>
</article>
{/if}

<script lang="ts">
    import type { OverlaySettings } from "../types";

    let { overlay_settings = $bindable() }: { overlay_settings: OverlaySettings } = $props();
</script>

<style>
    .toggle-card {
        display: flex;
        align-items: center;
        width: 100%;
        margin: 0.65rem 0;
        padding: 0.9rem 1rem;
        border-radius: 0.9rem;
        background: rgba(255, 255, 255, 0.06);
        border: 1px solid rgba(255, 255, 255, 0.08);
        color: var(--text);
        cursor: pointer;
        transition:
            background-color 0.18s ease,
            border-color 0.18s ease,
            opacity 0.18s ease,
            transform 0.18s ease;
        user-select: none;
        gap: 0.75rem;
        box-sizing: border-box;
        border: #0061c8 solid 2px;
    }

    .toggle-card:hover {
        background: rgba(255, 255, 255, 0.1);
        transform: translateY(-1px);
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
</style>