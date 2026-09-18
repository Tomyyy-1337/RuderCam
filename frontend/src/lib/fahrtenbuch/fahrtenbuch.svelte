<div class="entries">
    {#each $fahrtenbuch.getSessions().slice().reverse() as fahrt, index (fahrt)}
        <FahrtenbuchEintrag {fahrt} {index}/>
    {:else}
        <div class="no-entries-container">
            <p>Es wurden noch keine Fahrten aufgezeichnet.</p>
        </div>
    {/each}
</div>

<script lang="ts">
    import FahrtenbuchEintrag from "./fahrtenbuch_eintrag.svelte";
    import { fahrtenbuch } from "./fahrtenbuchStore";
</script>

<style>
    .entries {
        display: grid;
        gap: 0.8em;
    }

    .no-entries-container {
        display: flex;
        justify-content: center;
        align-items: center;
        height: 12em;
        color: var(--text);
        background-color: var(--section-background);
        border-radius: 1em;
        border: 1px solid color-mix(in srgb, var(--text) 18%, transparent);
    }

    p {
        font-size: 1.2em;
        text-align: center;
    }
</style>
