<div class={`battery ${tone}`}>
    <svg viewBox="0 0 24 24" aria-hidden="true">
        <rect x="3" y="7" width="16" height="10" rx="2" />
        <rect x="19" y="10" width="2" height="4" rx="1" />
        <rect x="5" y="9" width={Math.max(0, Math.min(12, value / 8))} height="6" rx="1" fill="currentColor" />
    </svg>
    <span>{value}%</span>
</div>

<script lang="ts">
    let { value = 0 }: { value: number } = $props();

    let tone = $derived(
        value <= 15 ? "red" :
        value <= 30 ? "orange" :
        value <= 50 ? "yellow" :
        "green"
    );
</script>

<style>
    .battery {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 3.6rem;
        min-width: 3.6rem;
        height: 1.45rem;
        gap: 0.2rem;
        padding: 0.2rem 0.35rem;
        border-radius: 999px;
        color: white;
        font-weight: 700;
        line-height: 1;
    }

    .battery.red {
        color: #e53935;
    }

    .battery.orange {
        color: #fb8c00;
    }

    .battery.yellow {
        color: #fdd835;
    }

    .battery.green {
        color: #43a047;
    }

    svg {
        width: 1.1rem;
        height: 1.1rem;
        display: block;
        flex-shrink: 0;
        fill: none;
        stroke: currentColor;
        stroke-width: 1.8;
    }

    span {
        line-height: 1;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        min-width: 2ch;
        text-align: center;
        font-size: 0.85rem;
    }
</style>