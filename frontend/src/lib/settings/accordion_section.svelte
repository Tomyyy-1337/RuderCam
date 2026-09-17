<svelte:window onkeydown={handleWindowKeydown} onpopstate={handleWindowPopstate} />

<div class="accordion-section">
    <button
        type="button"
        class="accordion-summary"
        aria-expanded={open}
        onclick={openOverlay}
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
        <div
            class="overlay-backdrop"
            role="button"
            tabindex="0"
            aria-label="Einstellungen schließen"
            onclick={handleBackdropClick}
            onkeydown={handleBackdropKeydown}
        >
            <div class="overlay-panel" role="dialog" aria-modal="true" tabindex="-1">
                <div class="overlay-header">
                    <div class="summary-copy">
                        <span class="title">{title}</span>
                        {#if description}
                            <span class="description">{description}</span>
                        {/if}
                    </div>

                    <button type="button" class="close-button" aria-label="Einstellungen schließen" onclick={() => close()}>
                        X
                    </button>
                </div>

                <div class="overlay-content">
                    {@render children?.()}
                </div>
            </div>
        </div>
    {/if}
</div>

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

    let scrollLocked = false;
    let historyEntryActive = false;

    $effect(() => {
        if (open && !scrollLocked) {
            document.documentElement.style.overflow = 'hidden';
            document.body.style.overflow = 'hidden';
            scrollLocked = true;
        } else if (!open && scrollLocked) {
            document.documentElement.style.overflow = '';
            document.body.style.overflow = '';
            scrollLocked = false;
        }

        return () => {
            if (scrollLocked) {
                document.documentElement.style.overflow = '';
                document.body.style.overflow = '';
                scrollLocked = false;
            }
        };
    });

    function openOverlay(): void {
        if (open) {
            return;
        }

        open = true;
        history.pushState({ ...(history.state ?? {}), accordionOverlay: true }, '');
        historyEntryActive = true;
    }

    function close(fromHistory = false): void {
        open = false;

        if (historyEntryActive && !fromHistory) {
            historyEntryActive = false;
            history.back();
        } else {
            historyEntryActive = false;
        }
    }

    function handleBackdropClick(event: MouseEvent): void {
        if (event.target === event.currentTarget) {
            close();
        }
    }

    function handleBackdropKeydown(event: KeyboardEvent): void {
        if ((event.key === 'Enter' || event.key === ' ') && event.target === event.currentTarget) {
            event.preventDefault();
            close();
        }
    }

    function handleWindowKeydown(event: KeyboardEvent): void {
        if (event.key === 'Escape' && open) {
            close();
        }
    }

    function handleWindowPopstate(): void {
        if (open && historyEntryActive) {
            close(true);
        }
    }
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

    .accordion-summary:hover {
        transform: none;
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
        transform: rotate(-45deg);
    }

    .overlay-backdrop {
        position: fixed;
        inset: 0;
        z-index: 50;
        display: grid;
        place-items: stretch;
        background: rgb(4 10 20 / 72%);
        backdrop-filter: blur(10px);
    }

    .overlay-panel {
        display: grid;
        grid-template-rows: auto 1fr;
        width: min(100%, 72rem);
        height: 100%;
        box-sizing: border-box;
        margin: 0 auto;
        padding: calc(1rem + env(safe-area-inset-top)) clamp(0.75rem, 1.5vw, 1.25rem) 1rem;
        overflow: auto;
        background: var(--section-background);
        box-shadow: 0 24px 80px rgb(0 0 0 / 45%);
    }

    .overlay-header {
        display: flex;
        align-items: flex-start;
        justify-content: space-between;
        gap: 1rem;
        padding-bottom: 1.25rem;
        border-bottom: 1px solid color-mix(in srgb, var(--text) 14%, transparent);
    }

    .close-button {
        flex: 0 0 auto;
        min-height: 2.5rem;
        padding: 0.55rem 0.85rem;
        border: 1px solid color-mix(in srgb, var(--text) 20%, transparent);
        border-radius: 0.7rem;
        background: color-mix(in srgb, var(--text) 8%, var(--section-background));
        color: var(--text);
        font: inherit;
        font-size: 1.4rem;
        cursor: pointer;
    }

    .close-button:focus-visible {
        outline: 2px solid var(--button-color);
        outline-offset: 2px;
    }

    .overlay-content {
        display: grid;
        align-content: start;
        gap: 0.65rem;
        padding-top: 1.25rem;
    }

    .overlay-content :global(button) {
        max-width: 100%;
    }

    .overlay-content :global(> *) {
        display: grid;
    }
</style>