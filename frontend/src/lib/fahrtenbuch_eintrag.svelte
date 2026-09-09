
<article class:expanded={isExpanded} class="trip-card">
    <button
        type="button"
        class="toggle-button"
        aria-expanded={isExpanded}
        onclick={() => (isExpanded = !isExpanded)}
    >
        <div class="toggle-top">
            <span class="toggle-label">Fahrt am {new Date(fahrt.client_time).toLocaleString()}</span>
            <span class="toggle-meta">
                <span class="toggle-indicator">{isExpanded ? '▾' : '▸'}</span>
                {isExpanded ? 'Weniger' : 'Mehr'}
            </span>
        </div>

        <div class="preview-stats" aria-hidden="true">
            <span>{distance_traveled_km} km</span>
            <span>{minutes}:{seconds} min</span>
        </div>
    </button>

    {#if isExpanded}
        <div class="trip-body" transition:slide={{ duration: 220 }}>
            <div class="map-shell">
                {#if (fahrt.gps_positions && fahrt.gps_positions.length > 0)}
                <Map waypoints={
                    (fahrt.gps_positions ?? []).map((position) => [Number(position.lon), Number(position.lat)])
                } />
                {:else}
                <div class="no-map-placeholder">
                    <span>Keine GPS-Daten verfügbar</span>
                </div>
                {/if}
            </div>

            <div class="distance-summary">
                <span class="distance-label">Gesamtstrecke</span>
                <div class="distance-value-row">
                    <span class="distance-value">{distance_traveled_km}</span>
                    <span class="distance-unit">km</span>
                </div>
            </div>

            <div class="stats-grid">
                <div class="stat-card">
                    <span class="stat-tag">Dauer</span>
                    <span class="stat-value">{minutes}:{seconds}</span>
                    <span class="stat-unit">min:sek</span>
                </div>

                <div class="stat-card">
                    <span class="stat-tag">Max. Tempo</span>
                    <span class="stat-value">{max_speed_kmh}</span>
                    <span class="stat-unit">km/h</span>
                </div>

                <div class="stat-card">
                    <span class="stat-tag">Ø Tempo</span>
                    <span class="stat-value">{average_speed_kmh}</span>
                    <span class="stat-unit">km/h</span>
                </div>

                <div class="stat-card">
                    <span class="stat-tag">Ø Schlagfrequenz</span>
                    <span class="stat-value">{average_bpm}</span>
                    <span class="stat-unit">bpm</span>
                </div>
            </div>

            <button class="delete-button" onclick={() => deleteSession(index)}>
                Fahrt löschen
            </button>
        </div>
    {/if}
</article>

<script>
    import Map from "./map.svelte";
    import { slide } from "svelte/transition";

    let { fahrt, index, fahrtenbuch } = $props();

    let isExpanded = $state(false);
    let minutes = $derived((Math.floor(fahrt.duration_secs / 60)).toString().padStart(2, '0'));
    let seconds = $derived((Math.round(fahrt.duration_secs % 60)).toString().padStart(2, '0'));
    let distance_traveled_km = $derived(Number(fahrt.distance_traveled_km ?? 0).toFixed(2));
    let max_speed_kmh = $derived(Number(fahrt.max_speed_kmh ?? 0).toFixed(1));
    let average_speed_kmh = $derived(Number(fahrt.average_speed_kmh ?? 0).toFixed(1));
    let average_bpm = $derived(Number(fahrt.average_bpm ?? 0).toFixed(0));

    function deleteSession(index) {
        fahrtenbuch.update(f => {
            f.deleteSession(f.length() - 1 - index);
            return f;
        });
    }
</script>

<style>
    article {
        --muted-text: color-mix(in srgb, var(--text) 72%, transparent);
        --soft-text: color-mix(in srgb, var(--text) 84%, transparent);
        --subtle-border: color-mix(in srgb, var(--text) 18%, transparent);
        --chip-bg: color-mix(in srgb, var(--text) 11%, transparent);
        background-color: var(--section-background);
        border-radius: 1em;
        padding: 0.35em;
        margin: 0;
        overflow: hidden;
        box-shadow: 0 3px 10px rgba(0, 0, 0, 0.2);
        transition: box-shadow 0.22s ease, transform 0.22s ease;
    }

    .expanded {
        box-shadow: 0 10px 24px rgba(0, 0, 0, 0.22);
        transform: translateY(-1px);
    }

    .toggle-button {
        display: flex;
        flex-direction: column;
        align-items: stretch;
        gap: 0.55em;
        text-align: left;
        width: 100%;
        padding: 0.85em;
        border: none;
        border-radius: 0.75em;
        background-color: color-mix(in srgb, var(--background) 82%, white 18%);
        color: var(--text);
        font: inherit;
        cursor: pointer;
        transition: background-color 0.15s ease, transform 0.15s ease;
    }

    .toggle-button:hover {
        background-color: color-mix(in srgb, var(--background) 74%, white 26%);
        transform: translateY(-1px);
    }

    .toggle-button:focus-visible {
        outline: 2px solid var(--button-color);
        outline-offset: 2px;
    }

    .toggle-label {
        font-weight: 600;
        flex: 1;
        color: var(--text);
    }

    .toggle-top {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 0.65rem;
    }

    .toggle-meta {
        display: inline-flex;
        align-items: center;
        gap: 0.45rem;
        color: var(--muted-text);
        font-size: 0.76rem;
        font-weight: 600;
        letter-spacing: 0.04em;
        text-transform: uppercase;
        white-space: nowrap;
    }

    .preview-stats {
        display: flex;
        flex-wrap: wrap;
        gap: 0.4rem;
    }

    .preview-stats span {
        font-size: 0.76rem;
        color: var(--soft-text);
        background-color: var(--chip-bg);
        padding: 0.2rem 0.5rem;
        border-radius: 999px;
    }

    .toggle-indicator {
        font-size: 1rem;
        line-height: 1;
        color: var(--soft-text);
        transition: transform 0.22s ease;
    }

    .expanded .toggle-indicator {
        transform: rotate(90deg);
    }

    .trip-body {
        margin-top: 0.75em;
        display: flex;
        flex-direction: column;
        gap: 0.75em;
        padding: 0 0.5em 0.5em;
    }

    .map-shell {
        width: 100%;
        height: 260px;
        border-radius: 0.75em;
        overflow: hidden;
        border: 1px solid var(--subtle-border);
        background-color: var(--background);
    }

    .distance-summary {
        display: flex;
        align-items: end;
        justify-content: space-between;
        gap: 0.5rem;
        background-color: var(--background);
        border-radius: 0.75em;
        padding: 0.7rem 0.9rem;
    }

    .distance-label {
        font-size: 0.74rem;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        color: var(--muted-text);
    }

    .distance-value-row {
        display: inline-flex;
        align-items: baseline;
        gap: 0.3rem;
    }

    .distance-value {
        font-size: clamp(2rem, 4vw, 2.8rem);
        line-height: 1;
        font-weight: 800;
        color: var(--text);
    }

    .distance-unit {
        font-size: 0.9rem;
        color: var(--muted-text);
        text-transform: uppercase;
        letter-spacing: 0.08em;
    }

    .stats-grid {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr)) !important;
        gap: 0.75rem;
    }

    .stat-card {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 0.2rem;
        min-height: 8rem;
        padding: 0.9rem 0.7rem;
        border-radius: 0.75em;
        background-color: var(--background);
        border: 1px solid var(--subtle-border);
        text-align: center;
    }

    .stat-tag {
        font-size: 0.68rem;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        color: var(--muted-text);
    }

    .stat-value {
        font-size: clamp(1.7rem, 3vw, 2.45rem);
        line-height: 1.05;
        color: var(--text);
        font-weight: 800;
    }

    .stat-unit {
        font-size: 0.8rem;
        color: var(--muted-text);
        text-transform: uppercase;
        letter-spacing: 0.08em;
    }

    .delete-button {
        width: 100%;
        border: 0;
        border-radius: 0.75em;
        background: rgba(239, 68, 68, 0.12);
        color: var(--text);
        padding: 0.8rem 1rem;
        font-weight: 700;
        cursor: pointer;
        transition: background-color 0.15s ease;
    }

    .delete-button:hover {
        background-color: rgba(239, 68, 68, 0.22);
    }

    @media (max-width: 520px) {
        .stats-grid {
            grid-template-columns: repeat(2, minmax(0, 1fr)) !important;
        }

        .toggle-top {
            align-items: flex-start;
        }

        .toggle-label {
            font-size: 0.95rem;
        }

        .map-shell {
            height: 220px;
        }
    }

    .no-map-placeholder {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 100%;
        height: 100%;
        background-color: var(--background);
        color: var(--muted-text);
        font-size: 0.9rem;
    }
</style>