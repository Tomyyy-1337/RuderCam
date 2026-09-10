{#if overlay_settings.show_overlay}
<div class="overlay {overlay_settings.position}">
    <div class="overlay-content">
        <div class="stats">
            <div class="card-group">
                {#each visibleCards as card (card.id)}
                    <div
                        class="overlay-stat"
                        in:fly={{ x: 36, duration: 800 }}
                        out:fly={{ x: -36, duration: 600 }}
                        animate:flip={{ duration: 600, easing: (t) => 1 - Math.pow(1 - t, 3) }}
                    >
                        <OverlayCard label={card.label} unit={card.unit} value={card.value} />
                    </div>
                {/each}
            </div>
        </div>

        <div class="controls {overlay_settings.position === 'top' ? 'bottom' : 'top'}">
            <div class="icon-container">
                <BatteryStatusIcon value={battery_percentage} />
                <SateliteIcon value={satellite_count} />
                {#if activeSession.isActive}
                    <div class="status-icon" in:fly={{ x: 24, duration: 500 }} out:fly={{ x: -24, duration: 500 }}>
                        <PauseIcon paused={activeSession.pausiert} />
                    </div>
                {/if}
            </div>

            <ToggleSessionButton {fahrtenbuch} bind:activeSession variant="overlay" />
        </div>
    </div>
</div>
{/if}

<script lang="ts">
    import { fly } from "svelte/transition";
    import { flip } from "svelte/animate";
    import BatteryStatusIcon from "./batteryStatusIcon.svelte";
    import OverlayCard from "./overlay_card.svelte";
    import PauseIcon from "./pauseIcon.svelte";
    import SateliteIcon from "./sateliteIcon.svelte";
    import ToggleSessionButton from "./toggleSessionButton.svelte";

    let {deviceStatus, activeSession = $bindable(), fahrtenbuch, overlay_settings} = $props();

    let fahrtzeit = $derived(activeSession.isActive ? Math.floor(activeSession.duration_secs / 60) + ":" + String(Math.floor(activeSession.duration_secs % 60)).padStart(2,'0') : '--:--');
    let distanz = $derived(activeSession.isActive ? activeSession.distance_traveled_km.toFixed(2) : '--.--');
    let battery_percentage = $derived(deviceStatus.battery_percentage);
    let satellite_count = $derived(deviceStatus.satellite_count);
    let speed_kmh = $derived(deviceStatus.speed_kmh.toFixed(1));
    let schlagzahl = $derived(deviceStatus.schlagzahl.toFixed(1));
    let distance_per_stroke_m = $derived((schlagzahl > 0 ? (deviceStatus.speed_kmh / 3600) / (schlagzahl / 60) * 1000 : 0).toFixed(2));

    let time_per_500m_s = $derived(deviceStatus.speed_kmh > 0 ? (500 / 1000) / (deviceStatus.speed_kmh / 3600) : 0);
    let time_per_500m_formatted = $derived(time_per_500m_s > 0 ? Math.floor(time_per_500m_s / 60) + ":" + String(Math.floor(time_per_500m_s % 60)).padStart(2,'0') : '0:00');

    let visibleCards = $derived([
        ...(overlay_settings.show_speed ? [{ id: 'speed', label: 'Geschwindigkeit', unit: 'km/h', value: speed_kmh }] : []),
        ...(overlay_settings.show_split_time ? [{ id: 'split', label: 'Split', unit: '500m', value: time_per_500m_formatted }] : []),
        ...(overlay_settings.show_schlagzahl ? [{ id: 'schlagzahl', label: 'Schlagzahl', unit: 'bpm', value: schlagzahl }] : []),
        ...(overlay_settings.show_distanc_per_stroke ? [{ id: 'distanz_per_schlag', label: 'Distanz/Schlag', unit: 'm', value: distance_per_stroke_m }] : []),
        ...(activeSession.isActive && overlay_settings.show_distanz ? [{ id: 'distanz', label: 'Distanz', unit: 'km', value: distanz }] : []),
        ...(activeSession.isActive && overlay_settings.show_fahrtzeit ? [{ id: 'fahrtzeit', label: 'Fahrtzeit', unit: '', value: fahrtzeit }] : [])
    ]);

</script>

<style>
    .overlay {
        position: absolute;
        inset: 0;
        display: flex;
        flex-direction: column;
        width: 100%;
        height: 100%;
        background: transparent;
        padding: 0 0.75rem;
        box-sizing: border-box;
        pointer-events: none;
    }

    .overlay.top {
        justify-content: space-between;
    }

    .overlay.bottom {
        justify-content: space-between;
    }

    .overlay-content {
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        width: 100%;
        height: 100%;
        min-height: 0;
    }

    .overlay.bottom .overlay-content {
        flex-direction: column-reverse;
    }

    .stats {
        display: grid;
        grid-template-columns: minmax(0, 1fr);
        align-items: center;
        width: 100%;
        background-color: rgba(0, 0, 0, 0.5);
        min-width: 0;
    }

    .controls {
        display: grid;
        grid-template-columns: auto minmax(0, 1fr);
        align-items: center;
        gap: 0.75rem;
        width: 100%;
        min-width: 0;
        min-height: 5.4rem;
    }

    .icon-container {
        display: grid;
        grid-template-rows: repeat(3, 1.7rem);
        grid-template-columns: 3.6rem;
        justify-items: center;
        align-content: center;
        row-gap: 0.35rem;
        width: 3.6rem;
        min-width: 3.6rem;
        height: calc(3 * 1.7rem + 2 * 0.35rem);
    }

    .icon-container,
    .controls,
    .card-group {
        pointer-events: auto;
    }

    .card-group {
        display: flex;
        flex-wrap: nowrap;
        align-items: stretch;
        gap: 0.4rem;
        width: 100%;
        min-width: 0;
        overflow: visible;
    }

    .overlay-stat {
        flex: 1 1 0;
        min-width: 0;
        width: 100%;
        contain: layout style;
        will-change: transform;
        overflow: visible;
    }

    .card-group :global(.stat) {
        flex: 1 1 0;
        min-width: 0;
        width: 100%;
        transform: translateZ(0);
        overflow: visible;
    }
</style>