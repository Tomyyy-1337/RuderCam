<section>  
    <ToggleSessionButton {fahrtenbuch} bind:activeSession variant="primary" />

    <div class="spacer"></div>

    <Table tableColumns={[
        { header: 'Dauer', value: `${duration_mins}:${duration_secs}` },
        { header: 'Distanz (km)', value: distance_traveled_km },
        { header: 'Avg. Geschwindigkeit (km/h)', value: average_speed_kmh },
    ]} />

</section>
    
<script lang="ts">
    import Table from "./table.svelte";
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
    let average_speed_kmh = $derived(activeSession.isActive ? activeSession.average_speed_kmh.toFixed(2) : '--.--');
</script>