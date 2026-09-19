<SettingsCard title="Overlay Aktivieren/Deaktivieren" description="Overlay in der Vollbildansicht ein- oder ausblenden.">
    <select id="overlay-show-overlay" name="overlay-show-overlay" bind:value={persistant_state.show_overlay}>
        <option value={true}>Overlay anzeigen</option>
        <option value={false}>Overlay ausblenden</option>
    </select>
</SettingsCard>

{#if persistant_state.show_overlay}
<SettingsCard title="Overlay Position" description="Position des Overlays in der Vollbildansicht.">
    <select id="overlay-position" name="overlay-position" bind:value={persistant_state.position}>
        <option value="top">Overlay Oben</option>
        <option value="bottom">Overlay Unten</option>
    </select>
</SettingsCard>

<SettingsCard title="Overlay Elemente" description="Verwalten welche Elemente im Overlay angezeigt werden sollen.">

    <label class:inactive={!persistant_state.show_speed} class="toggle-card">
        <input id="overlay-show-speed" name="overlay-show-speed" type="checkbox" bind:checked={persistant_state.show_speed} />
        <span>Geschwindigkeit (km/h)</span>
    </label>

    <label class:inactive={!persistant_state.show_split_time} class="toggle-card">
        <input id="overlay-show-split-time" name="overlay-show-split-time" type="checkbox" bind:checked={persistant_state.show_split_time} />
        <span>500m Split</span>
    </label>

    <label class:inactive={!persistant_state.show_schlagzahl} class="toggle-card">
        <input id="overlay-show-schlagzahl" name="overlay-show-schlagzahl" type="checkbox" bind:checked={persistant_state.show_schlagzahl} />
        <span>Schlagzahl</span>
    </label>

    <label class:inactive={!persistant_state.show_distanc_per_stroke} class="toggle-card">
        <input id="overlay-show-distance-per-stroke" name="overlay-show-distance-per-stroke" type="checkbox" bind:checked={persistant_state.show_distanc_per_stroke} />
        <span>Distanz pro Schlag</span>
    </label>
    
    <label class:inactive={!persistant_state.show_distanz} class="toggle-card">
        <input id="overlay-show-distanz" name="overlay-show-distanz" type="checkbox" bind:checked={persistant_state.show_distanz} />
        <span>Distanz</span>
    </label>

    <label class:inactive={!persistant_state.show_fahrtzeit} class="toggle-card">
        <input id="overlay-show-fahrtzeit" name="overlay-show-fahrtzeit" type="checkbox" bind:checked={persistant_state.show_fahrtzeit} />
        <span>Fahrtzeit</span>
    </label>

    <label class:inactive={!persistant_state.show_schlag_count} class="toggle-card">
        <input id="overlay-show-schlag-count" name="overlay-show-schlag-count" type="checkbox" bind:checked={persistant_state.show_schlag_count} />
        <span>Schläge</span>
    </label>

</SettingsCard>
{/if}

<script lang="ts">
    import { persistant_state } from "../classes/persistant_state_store.svelte";
    import SettingsCard from "./settings_card.svelte";
</script>

<style>
    .toggle-card {
        display: flex;
        align-items: center;
        width: 100%;
        margin: 0.1rem;
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
</style>