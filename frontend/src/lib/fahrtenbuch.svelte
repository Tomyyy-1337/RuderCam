<div class="entries">
    {#each $fahrtenbuch.getSessions().slice().reverse() as fahrt, index (fahrt)}
        <FahrtenbuchEintrag {fahrt} {index} {fahrtenbuch}/>
    {:else}
        <p>Es wurden noch keine Fahrten aufgezeichnet.</p>
    {/each}
</div>

<script>
    import FahrtenbuchEintrag from "./fahrtenbuch_eintrag.svelte";

    let { fahrtenbuch } = $props();
</script>

<style>
    .entries {
        display: grid;
        gap: 0.8em;
    }
</style>
