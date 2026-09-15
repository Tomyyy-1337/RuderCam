<div class="card-group">
    {#each visibleCards as card (card.id)}
        <div
            class="overlay-stat"
            in:fly={{ x: 36, duration: 800 }}
            out:fly={{ x: -36, duration: 600 }}
            animate:flip={{ duration: 600, easing: (t) => 1 - Math.pow(1 - t, 3) }}
        >
            <div class="stat">
                <p class="stat_label">{card.label}</p>
                <p class="stat_value">{card.value}</p>
                {#if card.unit}
                    <span class="stat_unit">{card.unit}</span>
                {/if}
            </div>
        </div>
    {/each}
</div>

<script lang="ts">
    import { fly } from "svelte/transition";
    import { flip } from "svelte/animate";

    let { visibleCards }: {
        visibleCards: Array<{ id: string; label: string; value: string | number; unit?: string }>;
    } = $props();
</script>

<style>
    .card-group {
        display: flex;
        flex-wrap: nowrap;
        align-items: stretch;
        gap: 0.4rem;
        width: 100%;
        min-width: 0;
        background-color: rgba(0, 0, 0, 0.5);
        padding: 0 0.75rem;
        box-sizing: border-box;
        overflow: visible;
        pointer-events: auto;
    }

    .overlay-stat {
        flex: 1 1 0;
        min-width: 0;
        width: 100%;
        contain: layout style;
        will-change: transform;
        overflow: visible;
    }

    .stat {
        margin-top: 0;
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        gap: 0.02rem;
        padding: 0 0.15rem;
        width: 100%;
        min-width: 7.5rem;
        box-sizing: border-box;
        overflow: visible;
        position: relative;
        z-index: 1;
        flex: 1 1 0;
        transform: translateZ(0);
    }

    .stat_label {
        font-weight: bold;
        font-size: clamp(0.7rem, 1.5vw + 0.2rem, 0.85rem);
        line-height: 1;
        min-height: 1.25em;
        margin: 0;
        color: white;
        text-align: center;
        width: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .stat_value {
        font-weight: normal;
        margin: 0;
        font-size: clamp(1.6rem, 4.3vw + 0.5rem, 4rem);
        line-height: 1;
        text-align: center;
        color: white;
        white-space: nowrap;
        display: block;
    }

    .stat_unit {
        display: block;
        font-size: clamp(0.65rem, 1.2vw + 0.45rem, 0.95rem);
        line-height: 1;
        margin-top: -0.08rem;
        color: lightgray;
        white-space: nowrap;
        text-align: center;
    }
</style>