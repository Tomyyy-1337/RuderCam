{#if overlay_settings.show_overlay}
<div class="overlay {overlay_settings.position}">
    <OverlayCard {visibleCards} />

    <div class="controls">
        <div class="icon-container">
            <BatteryStatusIcon value={battery_percentage} />
            <SateliteIcon value={satellite_count} />
            <PauseIcon paused={activeSession.pausiert} session={activeSession} />
        </div>

        <ToggleSessionButton {fahrtenbuch} bind:activeSession variant="overlay" />
    </div>
</div>
{/if}

<script lang="ts">
    import BatteryStatusIcon from "../icons/batteryStatusIcon.svelte";
    import OverlayCard from "./overlay_card.svelte";
    import PauseIcon from "../icons/pauseIcon.svelte";
    import SateliteIcon from "../icons/sateliteIcon.svelte";
    import ToggleSessionButton from "../components/toggleSessionButton.svelte";
    import type { Writable } from "svelte/store";
    import type { FahrtenbuchStore } from "../fahrtenbuch/fahrtenbuchStore";
    import type { ActiveSession, DeviceStatus, OverlaySettings } from "../types";

    let {
        deviceStatus,
        activeSession = $bindable(),
        fahrtenbuch,
        overlay_settings,
    }: {
        deviceStatus: DeviceStatus;
        activeSession: ActiveSession;
        fahrtenbuch: Writable<FahrtenbuchStore>;
        overlay_settings: OverlaySettings;
    } = $props();

    let fahrtzeit = $derived(activeSession.isActive ? Math.floor(activeSession.duration_secs / 60) + ":" + String(Math.floor(activeSession.duration_secs % 60)).padStart(2,'0') : '--:--');
    let distanz = $derived(activeSession.isActive ? activeSession.distance_traveled_km.toFixed(2) : '--.--');
    let battery_percentage = $derived(deviceStatus.battery_percentage);
    let satellite_count = $derived(deviceStatus.satellite_count);
    let speed_kmh = $derived(deviceStatus.speed_kmh.toFixed(1));
    let schlagzahlValue = $derived(deviceStatus.schlagzahl);
    let schlagzahl = $derived(schlagzahlValue.toFixed(1));
    let distance_per_stroke_m = $derived((schlagzahlValue > 0 ? (deviceStatus.speed_kmh / 3600) / (schlagzahlValue / 60) * 1000 : 0).toFixed(1));

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
        justify-content: space-between;
        width: 100%;
        height: 100%;
        min-height: 0;
        background: transparent;
        box-sizing: border-box;
        pointer-events: none;
        padding-top: env(safe-area-inset-top);
    }

    .overlay.bottom {
        flex-direction: column-reverse;
    }

    .controls {
        display: grid;
        grid-template-columns: auto minmax(0, 1fr);
        align-items: center;
        gap: 0.75rem;
        width: 100%;
        padding: 0 0.75rem;
        box-sizing: border-box;
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
    .controls {
        pointer-events: auto;
    }

</style>