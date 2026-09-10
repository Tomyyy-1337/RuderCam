<section>    
    <Table tableColumns={[
        { header: 'Geschwindigkeit (km/h)', value: deviceStatus.speed_kmh },
        { header: 'Schlagfrequenz (spm)', value: deviceStatus.schlagzahl },
    ]} />
</section>

<script lang="ts">
    import Table from "./table.svelte";
    import type { DeviceStatus } from "./types";
    
    let { deviceStatus }: { deviceStatus: DeviceStatus } = $props();
</script>