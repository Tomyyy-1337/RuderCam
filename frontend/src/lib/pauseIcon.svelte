<div class="pause-icon {paused ? 'paused' : 'running'}" role="status" aria-live="polite" aria-label={paused ? "Session paused" : "Session running"}>
    <svg viewBox="0 0 24 24" aria-hidden="true">
        {#if paused}
            <rect x="7" y="5" width="3" height="14" rx="1.2" />
            <rect x="14" y="5" width="3" height="14" rx="1.2" />
        {:else}
            <polygon points="7,5 19,12 7,19" />
        {/if}
    </svg>
</div>

<script lang="ts">
    let { paused = true } = $props();
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
    color: #fdd835;
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

.pause-icon svg {
    width: 1.2rem;
    height: 1.2rem;
    display: block;
    fill: currentColor;
    flex-shrink: 0;
}
</style>