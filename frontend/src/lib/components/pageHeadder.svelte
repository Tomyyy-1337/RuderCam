<section>
    <div id="Akkustand">
        {#if deviceStatus.isConnected}
            <BatteryStatusIcon value={battery_percentage} />
        {/if}
    </div>
    <h2>Ruder Cam Beta</h2>
    <div id="Satelien">
        {#if deviceStatus.isConnected}
            <SateliteIcon value={satellite_count} />
        {/if}
    </div>
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
        display: grid;
        grid-template-columns: 1fr auto 1fr;
        align-items: center;
        padding: 0.625rem;
        margin-top: 0.2rem;
    }

    #Akkustand, #Satelien {
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 0.2rem;
    }

    #Akkustand {
        justify-content: flex-start;
    }

    #Satelien {
        justify-content: flex-end;
    }

    h2 {
        margin: 0;
        height: 2.5rem;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 1.5rem;
        font-weight: bold;
        text-align: center;
        justify-self: center;
    }
</style>