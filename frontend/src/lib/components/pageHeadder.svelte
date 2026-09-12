<section>
    <div id="Akkustand"><BatteryStatusIcon value={battery_percentage} /> </div>
    <h2>Ruder Cam Beta</h2>
    <div id="Satelien"><SateliteIcon value={satellite_count} /> </div>
</section>

<script lang="ts">
    import BatteryStatusIcon from "../icons/batteryStatusIcon.svelte";
    import SateliteIcon from "../icons/sateliteIcon.svelte";
    import type { DeviceStatus } from "../types";

    let { deviceStatus }: { deviceStatus: DeviceStatus } = $props();

    let battery_percentage = $derived(deviceStatus.battery_percentage);
    let satellite_count = $derived(deviceStatus.satellite_count);
</script>

<style>
    section {
        display: flex;
        flex-direction: row;
        justify-content: space-between;
        padding: 0.5rem;
    }

    #Akkustand, #Satelien {
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 0.2rem;
    }

    h2 {
        margin: 0;
        font-size: 1.5rem;
        font-weight: bold;
        text-align: center;
    }
</style>