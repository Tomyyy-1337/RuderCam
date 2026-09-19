<div class="pause-icon {style}" role="status" aria-live="polite" aria-label={label}>
    <svg viewBox="0 0 24 24" aria-hidden="true">
        {#if temporary_state.session_is_active}
            {#if paused}
                <rect x="7" y="5" width="3" height="14" rx="1.2" />
                <rect x="14" y="5" width="3" height="14" rx="1.2" />
            {:else}
                <polygon points="7,5 19,12 7,19" />
            {/if}
        {:else}
            <rect x="6" y="6" width="12" height="12" rx="2" />
        {/if}
        
    </svg>
</div>

<script lang="ts">
    import { temporary_state } from "../classes/temporary_state_store.svelte";

    let { paused }: { paused: boolean } = $props();

    let style = $derived(temporary_state.session_is_active ? (paused ? "paused" : "running") : "inactive");
    let label = $derived(temporary_state.session_is_active ? (paused ? "Session paused" : "Session running") : "Session inactive");
</script>

<style>
.pause-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 3.6rem;
    min-width: 3.6rem;
    height: 1.7rem;
    padding: 0.22rem 0.4rem;
    border-radius: 999px;
    background-color: rgba(0,0,0,0);
    flex: 0 0 auto;
    position: relative;
    left: -0.5rem;
    box-sizing: border-box;
}

.pause-icon.paused {
    color: #fdd835;
}

.pause-icon.running {
    color: #4CAF50;
}

.pause-icon.inactive {
    color: #AE1E1E;
}

.pause-icon svg {
    width: 1.2rem;
    height: 1.2rem;
    display: block;
    fill: currentColor;
    flex-shrink: 0;
}
</style>