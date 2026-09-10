<details class="accordion-section" bind:open={open}>
    <summary class="accordion-summary">
        <div class="summary-copy">
            <span class="title">{title}</span>
            {#if description}
                <span class="description">{description}</span>
            {/if}
        </div>

        <span class="chevron" aria-hidden="true"></span>
    </summary>

    <div class="accordion-panel">
        <div class="accordion-panel-inner">
            {@render children?.()}
        </div>
    </div>
</details>

<script lang="ts">
    import type { Snippet } from "svelte";

    let {
        title,
        description = '',
        defaultOpen = false,
        children,
    }: {
        title: string;
        description?: string;
        defaultOpen?: boolean;
        children?: Snippet;
    } = $props();

    function getInitialOpen(): boolean {
        return defaultOpen;
    }

    let open = $state(getInitialOpen());
</script>

<style>
    .accordion-section {
        position: relative;
        overflow: clip;
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 1.35rem;
        background:
            linear-gradient(180deg, rgba(255, 255, 255, 0.08), rgba(255, 255, 255, 0.03)),
            rgba(20, 20, 20, 0.92);
        box-shadow:
            0 18px 48px rgba(0, 0, 0, 0.28),
            inset 0 1px 0 rgba(255, 255, 255, 0.04);
        backdrop-filter: blur(14px);
    }

    .accordion-section::before {
        content: '';
        position: absolute;
        inset: 0;
        border-radius: inherit;
        padding: 1px;
        background: linear-gradient(135deg, rgba(0, 97, 200, 0.5), rgba(255, 255, 255, 0.06));
        mask:
            linear-gradient(#000 0 0) content-box,
            linear-gradient(#000 0 0);
        mask-composite: exclude;
        pointer-events: none;
        opacity: 0.75;
    }

    .accordion-summary {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 0.75rem;
        padding: 0.85rem 1rem;
        cursor: pointer;
        list-style: none;
        user-select: none;
    }

    .accordion-summary::-webkit-details-marker {
        display: none;
    }

    .summary-copy {
        display: grid;
        gap: 0.1rem;
        min-width: 0;
    }

    .title {
        color: var(--text);
        font-size: clamp(1.1rem, 1rem + 0.55vw, 1.5rem);
        font-weight: 700;
        line-height: 1.15;
    }

    .description {
        color: rgba(255, 255, 255, 0.72);
        font-size: 0.9rem;
        line-height: 1.3;
    }

    .chevron {
        flex: 0 0 auto;
        width: 0.9rem;
        height: 0.9rem;
        border-right: 2px solid rgba(255, 255, 255, 0.72);
        border-bottom: 2px solid rgba(255, 255, 255, 0.72);
        transform: rotate(45deg);
        transition: transform 180ms ease, border-color 180ms ease;
    }

    .accordion-section[open] .chevron {
        transform: rotate(225deg);
        border-color: var(--button-color);
    }

    .accordion-section[open] .accordion-summary {
        padding-bottom: 0.6rem;
    }

    .accordion-panel {
        display: grid;
        grid-template-rows: 0fr;
        transition: grid-template-rows 220ms ease;
        padding: 0 1rem 0.75rem;
    }

    .accordion-section[open] .accordion-panel {
        grid-template-rows: 1fr;
    }

    .accordion-panel-inner {
        overflow: hidden;
        display: grid;
    }
</style>