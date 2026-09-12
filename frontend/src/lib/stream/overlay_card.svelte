<div class="stat">
    <p class="stat_label">{label}</p>
    <p class="stat_value">{value}</p>
    {#if unit}
        <span class="stat_unit">{unit}</span>
    {/if}
</div>

<script lang="ts">
    let { label, value, unit = "" }: { label: string; value: string | number; unit?: string } = $props();
</script>

<style>
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