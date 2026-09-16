<SettingsCard title="Fahrtenbuch Testeintrag" description="Fügt einen gefälschten Eintrag in das Fahrtenbuch ein.">
    <button style="width:100%;" onclick={addFakeEntry}>
        Gefälschten Eintrag hinzufügen  
    </button>
</SettingsCard>

<script lang="ts">
    import SettingsCard from "./settings_card.svelte";
    import type { Writable } from "svelte/store";
    import { FahrtenbuchStore, Session } from "../fahrtenbuch/fahrtenbuchStore";
    import type { GpsPosition, SessionJson } from "../types";

    let { fahrtenbuch = $bindable() }: { fahrtenbuch: Writable<FahrtenbuchStore> } = $props();

    const rheinRoute: GpsPosition[] = [
        { lat: 50.364000, lon: 7.607292, speed_kmh: 5.8 },
        { lat: 50.363648, lon: 7.607115, speed_kmh: 6.1 },
        { lat: 50.363294, lon: 7.606924, speed_kmh: 6.3 },
        { lat: 50.362938, lon: 7.606717, speed_kmh: 6.0 },
        { lat: 50.362577, lon: 7.606491, speed_kmh: 6.4 },
        { lat: 50.362211, lon: 7.606243, speed_kmh: 6.7 },
        { lat: 50.361838, lon: 7.605971, speed_kmh: 6.5 },
        { lat: 50.361458, lon: 7.605676, speed_kmh: 6.2 },
        { lat: 50.361069, lon: 7.605356, speed_kmh: 5.9 },
        { lat: 50.360671, lon: 7.605014, speed_kmh: 6.1 },
        { lat: 50.360264, lon: 7.604649, speed_kmh: 6.4 },
        { lat: 50.359847, lon: 7.604265, speed_kmh: 6.8 },
        { lat: 50.359421, lon: 7.603865, speed_kmh: 6.6 },
        { lat: 50.358985, lon: 7.603451, speed_kmh: 6.2 },
        { lat: 50.358539, lon: 7.603027, speed_kmh: 5.7 },
        { lat: 50.358085, lon: 7.602598, speed_kmh: 5.9 },
        { lat: 50.357623, lon: 7.602168, speed_kmh: 6.3 },
        { lat: 50.357155, lon: 7.601740, speed_kmh: 6.5 },
        { lat: 50.356680, lon: 7.601321, speed_kmh: 6.2 },
        { lat: 50.356201, lon: 7.600912, speed_kmh: 6.0 },
        { lat: 50.355719, lon: 7.600519, speed_kmh: 5.8 },
        { lat: 50.355234, lon: 7.600143, speed_kmh: 6.1 },
        { lat: 50.354750, lon: 7.599789, speed_kmh: 6.4 },
        { lat: 50.354266, lon: 7.599456, speed_kmh: 6.7 },
        { lat: 50.353785, lon: 7.599148, speed_kmh: 6.5 },
        { lat: 50.353308, lon: 7.598864, speed_kmh: 6.2 },
        { lat: 50.352835, lon: 7.598603, speed_kmh: 5.9 },
        { lat: 50.352369, lon: 7.598365, speed_kmh: 5.6 },
        { lat: 50.351910, lon: 7.598148, speed_kmh: 5.8 },
        { lat: 50.351459, lon: 7.597948, speed_kmh: 6.1 },
        { lat: 50.351018, lon: 7.597764, speed_kmh: 6.3 },
        { lat: 50.350585, lon: 7.597592, speed_kmh: 6.6 },
        { lat: 50.350162, lon: 7.597427, speed_kmh: 6.8 },
        { lat: 50.349749, lon: 7.597265, speed_kmh: 6.5 },
        { lat: 50.349346, lon: 7.597103, speed_kmh: 6.2 },
        { lat: 50.348952, lon: 7.596935, speed_kmh: 5.9 },
        { lat: 50.348566, lon: 7.596758, speed_kmh: 5.7 },
        { lat: 50.348189, lon: 7.596567, speed_kmh: 6.0 },
        { lat: 50.347819, lon: 7.596360, speed_kmh: 6.3 },
        { lat: 50.347455, lon: 7.596134, speed_kmh: 6.5 },
        { lat: 50.347096, lon: 7.595885, speed_kmh: 6.2 },
        { lat: 50.346741, lon: 7.595614, speed_kmh: 5.9 },
        { lat: 50.346388, lon: 7.595319, speed_kmh: 5.8 },
        { lat: 50.346036, lon: 7.594999, speed_kmh: 6.1 },
        { lat: 50.345684, lon: 7.594657, speed_kmh: 6.4 },
        { lat: 50.345329, lon: 7.594292, speed_kmh: 6.6 },
        { lat: 50.344971, lon: 7.593908, speed_kmh: 6.3 },
        { lat: 50.344608, lon: 7.593507, speed_kmh: 6.0 },
        { lat: 50.344240, lon: 7.593093, speed_kmh: 5.7 },
        { lat: 50.343864, lon: 7.592670, speed_kmh: 5.9 },
    ];

    function addFakeEntry(): void {
        const fakeSession: SessionJson = {
            client_time: new Date().toISOString(),
            duration_secs: 3120,
            distance_traveled_km: 10.6,
            max_speed_kmh: 14.3,
            average_speed_kmh: 12.0,
            average_bpm: 122,
            gps_positions: rheinRoute,
        };

        fahrtenbuch.update((store) => {
            store.addSession(new Session(fakeSession));
            return store;
        });
    }
</script>
