<article class="trip-card">
    <button
        type="button"
        class="toggle-button"
        aria-expanded={isExpanded}
        onclick={toggleExpanded}
    >
        <div class="toggle-top">
            <span class="toggle-label">Fahrt am {formattedClientTime}</span>
            <span class="toggle-meta">
                <span class="toggle-indicator">▸</span>
                Mehr
            </span>
        </div>

        <div class="preview-stats" aria-hidden="true">
            <span>{distance_traveled_km} km</span>
            <span>{minutes}:{seconds} min</span>
        </div>
    </button>
</article>

{#if isExpanded}
    <div class="trip-overlay" role="presentation" onclick={closeExpanded}>
        <div
            class="trip-fullscreen"
            role="dialog"
            tabindex="0"
            aria-modal="true"
            aria-label={`Fahrt am ${formattedClientTime}`}
            onclick={(event) => event.stopPropagation()}
            onkeydown={(event) => {
                if (event.key === 'Escape') {
                    closeExpanded();
                }
            }}
        >
            <div class="fullscreen-header">
                <div>
                    <span class="fullscreen-kicker">Fahrt am</span>
                    <h2>{formattedClientTime}</h2>
                </div>
                <button type="button" class="close-button" aria-label="Fahrt schließen" onclick={closeExpanded}>
                    ✕
                </button>
            </div>
            <div class="trip-body">
                <div class="map-shell">
                    {#if (fahrt.gps_positions && fahrt.gps_positions.length > 0)}
                        {#if showMap && MapComponent}
                            <MapComponent waypoints={
                                fahrt.gps_positions
                            } />
                        {:else}
                            <div class="map-loading-placeholder" aria-hidden="true"></div>
                        {/if}
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
                    <StatCard label="Dauer" value={`${minutes}:${seconds}`} unit="min:sek" />
                    <StatCard label="Ø Splittime" value={average_splittime_per_500m} unit="sek / 500 m" />
                    <StatCard label="Ø Tempo" value={average_speed_kmh} unit="km/h" />
                    <StatCard label="Ø Schlagzahl" value={average_bpm} unit="bpm" />
                </div>

                <button class="delete-button" class:confirming={confirmingDelete} onclick={handleDeleteClick}>
                    {confirmingDelete ? 'Löschen bestätigen' : 'Fahrt löschen'}
                </button>
            </div>
        </div>
    </div>
{/if}

<script lang="ts">
    import { onMount } from "svelte";
    import type { Component } from "svelte";
    import type { Writable } from "svelte/store";
    import type { FahrtenbuchStore, Session } from "./fahrtenbuchStore";
    import type { GpsPosition } from "../types";
    import StatCard from "./stat_card.svelte";

    let { fahrt, index, fahrtenbuch }: {
        fahrt: Session;
        index: number;
        fahrtenbuch: Writable<FahrtenbuchStore>;
    } = $props();

    let MapComponent = $state<Component<{ waypoints: GpsPosition[] }>>();
    let isExpanded = $state(false);
    let showMap = $state(false);
    let confirmingDelete = $state(false);
    let formattedClientTime = $derived(new Date(fahrt.client_time).toLocaleString([], {
        hour: '2-digit',
        minute: '2-digit',
        year: 'numeric',
        month: 'numeric',
        day: 'numeric'
    }));
    let minutes = $derived((Math.floor(fahrt.duration_secs / 60)).toString().padStart(2, '0'));
    let seconds = $derived((Math.round(fahrt.duration_secs % 60)).toString().padStart(2, '0'));
    let distance_traveled_km = $derived(Number(fahrt.distance_traveled_km ?? 0).toFixed(2));
    let average_splittime_per_500m_seconds = $derived(Number(
        fahrt.average_speed_kmh > 0
            ? (500 / (fahrt.average_speed_kmh * 1000 / 3600)).toFixed(1)
            : '0.0'
    ));
    let average_splittime_per_500m = $derived((
        Math.floor(average_splittime_per_500m_seconds / 60).toString().padStart(2, '0') +
        ':' +
        (Math.round(average_splittime_per_500m_seconds % 60)).toString().padStart(2, '0')
    ));
    
    let average_speed_kmh = $derived(Number(fahrt.average_speed_kmh ?? 0).toFixed(1));
    let average_bpm = $derived(Number(fahrt.average_bpm ?? 0).toFixed(0));

    onMount(() => {
        const handlePopState = (): void => {
            if (isExpanded && !isTripOverlayHistoryState()) {
                closeExpandedLocally();
            }
        };

        window.addEventListener("popstate", handlePopState);

        return () => {
            window.removeEventListener("popstate", handlePopState);
            if (isExpanded) {
                document.body.style.overflow = '';
            }
        };
    });

    async function loadMapComponent() {
        if (MapComponent) {
            return;
        }

        const module = await import("./map.svelte");
        MapComponent = module.default;
    }

    async function toggleExpanded() {
        if (isExpanded) {
            closeExpanded();
            return;
        }

        isExpanded = true;
        showMap = false;
        document.body.style.overflow = 'hidden';
        window.history.pushState({ ...(window.history.state ?? {}), activeTab: "sessions", tripOverlay: true }, "", window.location.href);
        await loadMapComponent();
        showMap = true;
    }

    function closeExpanded() {
        if (!isExpanded) {
            return;
        }

        if (isTripOverlayHistoryState()) {
            window.history.back();
            return;
        }

        closeExpandedLocally();
    }

    function closeExpandedLocally() {
        if (!isExpanded) {
            return;
        }

        showMap = false;
        isExpanded = false;
        confirmingDelete = false;
        document.body.style.overflow = '';
    }

    function isTripOverlayHistoryState(): boolean {
        return (window.history.state as { tripOverlay?: boolean } | null)?.tripOverlay === true;
    }

    function handleDeleteClick() {
        if (!confirmingDelete) {
            confirmingDelete = true;
        } else {
            deleteSession(index);
            confirmingDelete = false;
        }
    }

    function deleteSession(index: number) {
        fahrtenbuch.update((store) => {
            store.deleteSession(store.length() - 1 - index);
            return store;
        });
    }
</script>

<style>
    article {
        --muted-text: color-mix(in srgb, var(--text) 72%, transparent);
        --soft-text: color-mix(in srgb, var(--text) 84%, transparent);
        --subtle-border: color-mix(in srgb, var(--text) 18%, transparent);
        --chip-bg: color-mix(in srgb, var(--text) 11%, transparent);
        border: 1px solid color-mix(in srgb, var(--text) 14%, transparent);
        background: color-mix(in srgb, var(--section-background) 70%, var(--background) 30%);
        border-radius: 1rem;
        padding: 0;
        margin: 0;
        overflow: hidden;
        transition: border-color 0.22s ease, transform 0.22s ease;
    }

    .toggle-button {
        display: flex;
        flex-direction: column;
        align-items: stretch;
        gap: 0.55em;
        text-align: left;
        width: 100%;
        padding: 0.85rem 1rem;
        border: none;
        border-radius: 0;
        background: transparent;
        color: var(--text);
        font: inherit;
        cursor: pointer;
        -webkit-tap-highlight-color: transparent;
        transition: background-color 0.15s ease, transform 0.15s ease;
    }

    .toggle-button:hover {
        background-color: transparent;
        transform: none;
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

    .toggle-meta,
    .distance-label,
    .distance-unit {
        color: var(--muted-text);
        text-transform: uppercase;
    }

    .toggle-meta {
        display: inline-flex;
        align-items: center;
        gap: 0.45rem;
        font-size: 0.76rem;
        font-weight: 600;
        letter-spacing: 0.04em;
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

    .trip-overlay {
        position: fixed;
        inset: 0;
        z-index: 40;
        background: color-mix(in srgb, var(--background) 78%, transparent);
        backdrop-filter: blur(8px);
        display: flex;
        align-items: stretch;
        justify-content: center;
        padding: 0.7rem;
    }

    .trip-fullscreen {
        width: min(100%, 960px);
        max-height: 100vh;
        background: color-mix(in srgb, var(--section-background) 92%, var(--background) 8%);
        border: 1px solid color-mix(in srgb, var(--text) 18%, transparent);
        border-radius: 1.25rem;
        box-shadow: 0 32px 80px rgba(0, 0, 0, 0.45);
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    .fullscreen-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        padding: 1rem 1.25rem 0.75rem;
        border-bottom: 1px solid color-mix(in srgb, var(--text) 12%, transparent);
    }

    .fullscreen-kicker {
        display: block;
        font-size: 0.72rem;
        text-transform: uppercase;
        letter-spacing: 0.12em;
        color: var(--muted-text);
        margin-bottom: 0.2rem;
    }

    .fullscreen-header h2 {
        margin: 0;
        font-size: clamp(1.1rem, 2vw, 1.6rem);
        line-height: 1.2;
    }

    .close-button {
        width: 2.5rem;
        height: 2.5rem;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        border: 1px solid color-mix(in srgb, var(--text) 18%, transparent);
        border-radius: 999px;
        background: transparent;
        color: var(--text);
        font-size: 1.35rem;
        line-height: 1;
        padding: 0;
        cursor: pointer;
    }

    .trip-body {
        display: flex;
        flex-direction: column;
        gap: 0.75em;
        padding: 1rem 1.25rem 1.25rem;
        overflow-y: auto;
    }

    .map-shell {
        width: 100%;
        height: clamp(260px, 40vh, 420px);
        border-radius: 0.75em;
        overflow: hidden;
        border: 1px solid var(--subtle-border);
        background-color: var(--background);
    }

    .map-loading-placeholder,
    .no-map-placeholder {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .map-loading-placeholder {
        background:
            linear-gradient(120deg, transparent 30%, color-mix(in srgb, var(--text) 8%, transparent) 45%, transparent 60%),
            var(--section-background);
        background-size: 220% 100%, 100% 100%;
        animation: map-placeholder-shimmer 0.9s ease-out 1;
    }

    .distance-summary {
        display: flex;
        align-items: end;
        justify-content: space-between;
        gap: 0.5rem;
        background-color: var(--background);
        border: 1px solid var(--subtle-border);
        border-radius: 0.75em;
        padding: 0.7rem 0.9rem;
    }

    @keyframes map-placeholder-shimmer {
        from {
            background-position: 100% 0, 0 0;
        }

        to {
            background-position: 0 0, 0 0;
        }
    }

    .distance-label {
        font-size: 0.74rem;
        letter-spacing: 0.12em;
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
        letter-spacing: 0.08em;
    }

    .stats-grid {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr)) !important;
        gap: 0.75rem;
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

    .delete-button.confirming {
        background-color: rgba(239, 68, 68, 0.25);
    }

    .delete-button.confirming:hover {
        background-color: rgba(239, 68, 68, 0.35);
    }

    @media (max-width: 520px) {
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
        background-color: var(--background);
        color: var(--muted-text);
        font-size: 0.9rem;
    }
</style>