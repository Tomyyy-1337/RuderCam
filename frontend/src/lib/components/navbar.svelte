<script lang="ts">
    import type { AppTab } from "../types";

    let { activeTab = $bindable() }: { activeTab: AppTab } = $props();
</script>

<nav
    aria-label="Navigation"
    class="tabbar"
    class:sessions-active={activeTab === 'sessions'}
    class:settings-active={activeTab === 'settings'}
>
    <div class="tabbar_indicator" aria-hidden="true"></div>
    <button
        type="button"
        onclick={() => activeTab = 'sessions'}
        class="tab"
        class:active={activeTab === 'sessions'}
        aria-pressed={activeTab === 'sessions'}
    >
        <span class="tab_label">Fahrtenbuch</span>
    </button>
    <button
        type="button"
        onclick={() => activeTab = 'settings'}
        class="tab"
        class:active={activeTab === 'settings'}
        aria-pressed={activeTab === 'settings'}
    >
        <span class="tab_label">Einstellungen</span>
    </button>
</nav>

<style>
    .tabbar {
        --tabbar-padding: 0.5rem;
        --tabbar-gap: 0.6rem;
        --active-index: 0;
        display: flex;
        gap: var(--tabbar-gap);
        padding: var(--tabbar-padding);
        margin: 1rem 0 1.25rem;
        background: var(--section-background);
        border: 1px solid color-mix(in srgb, var(--text) 14%, transparent);
        border-radius: 1.25rem;
        position: relative;
    }

    .tabbar.sessions-active {
        --active-index: 0;
    }

    .tabbar.settings-active {
        --active-index: 1;
    }

    .tabbar_indicator {
        position: absolute;
        top: var(--tabbar-padding);
        left: calc(
            var(--tabbar-padding)
            + var(--active-index) * (
                (100% - (var(--tabbar-padding) * 2) - var(--tabbar-gap)) / 2
                + var(--tabbar-gap)
            )
        );
        width: calc((100% - (var(--tabbar-padding) * 2) - var(--tabbar-gap)) / 2);
        height: calc(100% - (var(--tabbar-padding) * 2));
        border-radius: 0.9rem;
        background: color-mix(in srgb, var(--button-color) 22%, var(--section-background));
        border: 1px solid color-mix(in srgb, var(--button-color) 52%, var(--text) 12%);
        transition: left 260ms ease;
        pointer-events: none;
    }

    .tab {
        -webkit-tap-highlight-color: transparent;
        display: flex;
        flex: 1 1 0;
        position: relative;
        z-index: 1;
        align-items: center;
        justify-content: center;
        min-width: 0;
        padding: 0.9rem 1rem;
        color: color-mix(in srgb, var(--text) 72%, transparent);
        background: transparent;
        border: 1px solid transparent;
        border-radius: 0.9rem;
        transition:
            transform 160ms ease,
            background-color 160ms ease,
            color 160ms ease,
            border-color 160ms ease;
    }

    .tab:focus {
        outline: none;
    }

    .tab:hover {
        transform: translateY(-1px);
        color: var(--text);
        background: color-mix(in srgb, var(--text) 8%, transparent);
        border-color: color-mix(in srgb, var(--text) 18%, transparent);
    }

    .tab:focus-visible {
        outline: 2px solid color-mix(in srgb, var(--button-color) 88%, white 12%);
        outline-offset: 2px;
    }

    .tab.active {
        color: var(--text);
        background: transparent;
        border-color: transparent;
    }

    .tab_label {
        font-size: 0.95rem;
        font-weight: 700;
        letter-spacing: 0.01em;
        white-space: nowrap;
    }

    @media (prefers-reduced-motion: reduce) {
        .tabbar_indicator {
            transition: none;
        }
    }
</style>