<nav
    aria-label="Navigation"
    class="tabbar"
    class:camera-active={persistant_state.active_tab === 'camera'}
    class:sessions-active={persistant_state.active_tab === 'sessions'}
    class:settings-active={persistant_state.active_tab === 'settings'}
>
    <div class="tabbar_indicator" aria-hidden="true"></div>
    <button
        type="button"
        onclick={(event) => selectTab('camera', event)}
        class="tab"
        class:active={persistant_state.active_tab === 'camera'}
        aria-pressed={persistant_state.active_tab === 'camera'}
    >
        <span class="tab_label">Kamera</span>
    </button>
    <button
        type="button"
        onclick={(event) => selectTab('sessions', event)}
        class="tab"
        class:active={persistant_state.active_tab === 'sessions'}
        aria-pressed={persistant_state.active_tab === 'sessions'}
    >
        <span class="tab_label">Fahrtenbuch</span>
    </button>
    <button
        type="button"
        onclick={(event) => selectTab('settings', event)}
        class="tab"
        class:active={persistant_state.active_tab === 'settings'}
        aria-pressed={persistant_state.active_tab === 'settings'}
    >
        <span class="tab_label">Settings</span>
    </button>
</nav>

<script lang="ts">
    import { persistant_state, type AppTab } from "../classes/persistant_state_store.svelte";

    function selectTab(nextTab: AppTab, event?: MouseEvent): void {
        event?.preventDefault();
        event?.stopPropagation();

        if (document.activeElement instanceof HTMLElement) {
            document.activeElement.blur();
        }

        if (nextTab === persistant_state.active_tab) {
            return;
        }

        persistant_state.active_tab = nextTab;
        window.history.pushState({ activeTab: nextTab }, "", window.location.href);
    }
</script>

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
        align-items: center;
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
        -webkit-appearance: none;
        appearance: none;
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

    .tab:active,
    .tab:hover,
    .tab:focus-visible {
        background: transparent;
        border-color: transparent;
        box-shadow: none;
        outline: none;
        transform: none;
    }

    .tab:focus {
        outline: none;
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