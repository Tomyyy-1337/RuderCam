<article class="settings-card">
    <div class="header">
        <h4>{title}</h4>

        {#if description}
            <p class="description">{description}</p>
        {/if}
    </div>

    <div class="content">
        {@render children?.()}
    </div>
</article>

<script lang="ts">
    import type { Snippet } from "svelte";

    let {
        title,
        description = "",
        children,
    }: {
        title: string;
        description?: string;
        children?: Snippet;
    } = $props();
</script>

<style>
    .settings-card {
        margin: 0;
        padding: 1rem;
        background-color: var(--background);
        border: 1px solid color-mix(in srgb, var(--text) 18%, transparent);
        border-radius: 1rem;
    }

    .header {
        padding-bottom: 0.85rem;
        border-bottom: 1px solid color-mix(in srgb, var(--text) 18%, transparent);
    }

    h4 {
        margin: 0;
    }

    .description {
        margin: 0.45rem 0 0;
        color: color-mix(in srgb, var(--text) 72%, transparent);
        line-height: 1.45;
    }

    .content {
        display: grid;
        gap: 0.2rem;
        padding-top: 0.55rem;
    }

    .content :global(p) {
        margin: 0.2rem;
    }
</style>