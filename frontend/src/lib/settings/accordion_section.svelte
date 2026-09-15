<div class="accordion-section" class:expanded={open}>
    <button
        type="button"
        class="accordion-summary"
        aria-expanded={open}
        onclick={() => (open = !open)}
    >
        <div class="summary-copy">
            <span class="title">{title}</span>
            {#if description}
                <span class="description">{description}</span>
            {/if}
        </div>

        <span class="chevron" aria-hidden="true"></span>
    </button>

    {#if open}
        <div class="accordion-panel" transition:slide={{ duration: 320 }}>
        <div class="accordion-panel-inner">
            {@render children?.()}
        </div>
        </div>
    {/if}
</div>

<script lang="ts">
    import { slide } from "svelte/transition";
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
        border: 1px solid color-mix(in srgb, var(--text) 14%, transparent);
        border-radius: 1rem;
        background: color-mix(in srgb, var(--section-background) 70%, var(--background) 30%);
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
        -webkit-tap-highlight-color: transparent;
        width: 100%;
        border: 0;
        border-radius: 1rem;
        background: transparent;
        color: inherit;
        font: inherit;
        text-align: left;
    }

    .accordion-summary:focus-visible {
        outline: 2px solid var(--button-color);
        outline-offset: 2px;
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
        color: color-mix(in srgb, var(--text) 72%, transparent);
        font-size: 0.9rem;
        line-height: 1.3;
    }

    .chevron {
        flex: 0 0 auto;
        width: 0.9rem;
        height: 0.9rem;
        border-right: 2px solid color-mix(in srgb, var(--text) 72%, transparent);
        border-bottom: 2px solid color-mix(in srgb, var(--text) 72%, transparent);
        transform: rotate(45deg);
        transition: transform 180ms ease, border-color 180ms ease;
    }

    .accordion-section.expanded .chevron {
        transform: rotate(225deg);
        border-color: var(--text);
    }

    .accordion-section.expanded .accordion-summary {
        padding-bottom: 0.6rem;
    }

    .accordion-panel {
        display: grid;
        padding: 0 0.6rem 0.75rem;
    }

    .accordion-panel-inner {
        display: grid;
    }

    .accordion-panel-inner :global(> article) {
        margin-top: 0.25em;
        margin-bottom: 0.25em;
    }
</style>