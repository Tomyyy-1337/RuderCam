<script lang="ts">
    let { label, value, unit }: {
        label: string;
        value: string | number;
        unit: string;
    } = $props();
</script>

<div class="stat-card">
    <span class="stat-tag">{label}</span>
    <span class="stat-value">{value}</span>
    <span class="stat-unit">{unit}</span>
</div>

<style>
    .stat-card {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 0.2rem;
        min-height: 6rem;
        padding: 0.9rem 0.7rem;
        border-radius: 0.75em;
        background-color: var(--background);
        border: 1px solid var(--subtle-border);
        text-align: center;
    }

    .stat-tag,
    .stat-unit {
        color: color-mix(in srgb, var(--text) 72%, transparent);
        text-transform: uppercase;
        letter-spacing: 0.12em;
    }

    .stat-tag {
        font-size: 0.68rem;
    }

    .stat-value {
        font-size: clamp(1.7rem, 3vw, 2.45rem);
        line-height: 1.05;
        color: var(--text);
        font-weight: 800;
    }

    .stat-unit {
        font-size: 0.8rem;
        letter-spacing: 0.08em;
    }
</style>
