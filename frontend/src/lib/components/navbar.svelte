<script lang="ts">
    import type { AppTab } from "../types";

    let { activeTab = $bindable() }: { activeTab: AppTab } = $props();
</script>

<nav
    aria-label="Navigation"
    class="tabbar"
    class:camera-active={activeTab === 'camera'}
    class:sessions-active={activeTab === 'sessions'}
    class:settings-active={activeTab === 'settings'}
>
    <div class="tabbar_indicator" aria-hidden="true"></div>
    <button
        type="button"
        onclick={() => activeTab = 'camera'}
        class="tab"
        class:active={activeTab === 'camera'}
        aria-pressed={activeTab === 'camera'}
    >
        <span class="tab_label">Kamera</span>
    </button>
    <button
        type="button"
        onclick={() => activeTab = 'sessions'}
        class="tab"
        class:active={activeTab === 'sessions'}
        aria-pressed={activeTab === 'sessions'}
    >
        <span class="tab_label sessions_label">Fahrtenbuch</span>
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
    nav {
        display: flex;
        background-color: var(--section-background);
        padding: 1em;
        border-radius: 1em;
        gap: var(--space-md);
    }

    .tabbar {
        --tabbar-padding: 0.5rem;
        --tabbar-gap: 0.6rem;
        --tab-width: calc((100% - (var(--tabbar-padding) * 2) - (var(--tabbar-gap) * 2)) / 3);
        --indicator-shift: 0px;
        box-sizing: border-box;
        display: grid;
        grid-template-columns: repeat(3, minmax(0, 1fr));
        gap: var(--tabbar-gap);
        padding: var(--tabbar-padding);
        margin: 1rem 0;
        background: var(--section-background);
        border: 1px solid color-mix(in srgb, var(--text) 14%, transparent);
        border-radius: 1.25rem;
        position: relative;
    }

    .tabbar.sessions-active {
        --indicator-shift: calc(var(--tab-width) + var(--tabbar-gap));
    }

    .tabbar.settings-active {
        --indicator-shift: calc(var(--tab-width) + var(--tabbar-gap) + var(--tab-width) + var(--tabbar-gap));
    }

    .tabbar.camera-active {
        --indicator-shift: 0px;
    }

    .tabbar_indicator {
        position: absolute;
        top: var(--tabbar-padding);
        left: calc(var(--tabbar-padding) + var(--indicator-shift));
        width: var(--tab-width);
        height: calc(100% - (var(--tabbar-padding) * 2));
        border-radius: 0.9rem;
        background: color-mix(in srgb, var(--button-color) 22%, var(--section-background));
        border: 1px solid color-mix(in srgb, var(--button-color) 52%, var(--text) 12%);
        box-sizing: border-box;
        transition: left 260ms ease;
        pointer-events: none;
    }

    .tab {
        -webkit-tap-highlight-color: transparent;
        display: flex;
        position: relative;
        z-index: 1;
        align-items: center;
        justify-content: center;
        min-width: 0;
        width: 100%;
        box-sizing: border-box;
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

    .sessions_label {
        transform: translateX(-0.2rem);
    }

    @media (prefers-reduced-motion: reduce) {
        .tabbar_indicator {
            transition: none;
        }
    }
</style>