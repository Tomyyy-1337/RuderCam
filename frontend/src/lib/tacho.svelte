<section>    
    <Table tableColumns={[
        { header: 'Geschwindigkeit (km/h)', value: deviceStatus.speed_kmh },
        { header: 'Schlagfrequenz (spm)', value: deviceStatus.schlagzahl },
    ]} />
</section>

<script>
    import Table from "./table.svelte";
    
    let { deviceStatus } = $props();
</script>