<div class="entries">
    {#each $fahrtenbuch.getSessions().slice().reverse() as fahrt, index (fahrt)}
        <FahrtenbuchEintrag {fahrt} {index} {fahrtenbuch}/>
    {:else}
        <p>Es wurden noch keine Fahrten aufgezeichnet.</p>
    {/each}
</div>

<script lang="ts">
    import FahrtenbuchEintrag from "./fahrtenbuch_eintrag.svelte";
    import type { Writable } from "svelte/store";
    import type { FahrtenbuchStore } from "./fahrtenbuchStore";

    let { fahrtenbuch }: { fahrtenbuch: Writable<FahrtenbuchStore> } = $props();
</script>

<style>
    .entries {
        display: grid;
        gap: 0.8em;
    }
</style>
