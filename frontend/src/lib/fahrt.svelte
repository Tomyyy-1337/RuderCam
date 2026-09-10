<section>  
    <ToggleSessionButton {fahrtenbuch} bind:activeSession variant="primary" />

    <div class="spacer"></div>

    <div class="ride-metrics">
        <div class="stat-card">
            <span class="stat-tag">Dauer</span>
            <span class="stat-value">{duration_mins}:{duration_secs}</span>
            <span class="stat-unit">min:sek</span>
        </div>

        <div class="stat-card">
            <span class="stat-tag">Distanz</span>
            <span class="stat-value">{distance_traveled_km}</span>
            <span class="stat-unit">km</span>
        </div>
    </div>

</section>
    
<script lang="ts">
    import ToggleSessionButton from "./toggleSessionButton.svelte";
    import type { Writable } from "svelte/store";
    import type { FahrtenbuchStore } from "./fahrtenbuchStore";
    import type { ActiveSession } from "./types";

    let {
        fahrtenbuch = $bindable(),
        activeSession = $bindable(),
    }: {
        fahrtenbuch: Writable<FahrtenbuchStore>;
        activeSession: ActiveSession;
    } = $props();

    let duration_secs = $derived(activeSession.isActive ? String(Math.floor(Math.round(activeSession.duration_secs) % 60)).padStart(2,'0') : '--');
    let duration_mins = $derived(activeSession.isActive ? String(Math.floor(Math.round(activeSession.duration_secs) / 60)).padStart(2,'0') : '--');
    let distance_traveled_km = $derived(activeSession.isActive ? activeSession.distance_traveled_km.toFixed(2) : '--.--');
</script>

<style>
    .ride-metrics {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 0.75rem;
        align-items: stretch;
    }

    .stat-card {
        display: flex;
        flex-direction: column;
        justify-content: center;
        gap: 0.2rem;
        min-height: 6rem;
        padding: 0.9rem 0.85rem;
        border-radius: 0.75rem;
        background-color: var(--background);
        border: 1px solid color-mix(in srgb, var(--text) 18%, transparent);
    }

    .stat-tag,
    .stat-unit {
        color: color-mix(in srgb, var(--text) 72%, transparent);
        text-transform: uppercase;
    }

    .stat-tag {
        font-size: 0.68rem;
        letter-spacing: 0.12em;
    }

    .stat-card {
        align-items: center;
        text-align: center;
    }

    .stat-value {
        font-size: clamp(1.7rem, 3vw, 2.45rem);
        line-height: 1.05;
        color: var(--text);
        font-weight: 800;
    }

    .stat-unit {
        font-size: 0.8rem;
        letter-spacing: 0.08em;
    }

</style>