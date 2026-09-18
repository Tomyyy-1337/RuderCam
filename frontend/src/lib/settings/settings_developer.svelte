<SettingsCard title="Fahrtenbuch Testeintrag" description="Fügt einen gefälschten Eintrag in das Fahrtenbuch ein.">
    <button style="width:100%;" onclick={addFakeEntry}>
        Gefälschten Eintrag hinzufügen  
    </button>
</SettingsCard>

<script lang="ts">
    import SettingsCard from "./settings_card.svelte";
    import { fahrtenbuch, Session } from "../classes/fahrtenbuchStore";

    function addFakeEntry(): void {
        fahrtenbuch.update((store) => {
            store.addSession(new Session().fromJSON({
                distance_traveled_km: 10,
                average_speed_kmh: 10,
                duration_secs: 20,
            }));
            return store;
        });
    }
</script>
