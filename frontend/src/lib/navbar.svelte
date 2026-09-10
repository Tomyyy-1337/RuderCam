<script lang="ts">
    let { activeTab = $bindable() } = $props();
</script>

<nav aria-label="Navigation" class="tabbar">
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
        display: flex;
        gap: 0.6rem;
        padding: 0.5rem;
        margin: 1rem 0 1.25rem;
        background:
            linear-gradient(180deg, rgba(255, 255, 255, 0.08), rgba(255, 255, 255, 0.03)),
            var(--section-background);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 1.25rem;
        box-shadow: 0 18px 40px rgba(0, 0, 0, 0.22);
        backdrop-filter: blur(10px);
        -webkit-backdrop-filter: blur(10px);
    }

    .tab {
        display: flex;
        flex: 1 1 0;
        align-items: center;
        justify-content: center;
        min-width: 0;
        padding: 0.9rem 1rem;
        color: rgba(255, 255, 255, 0.72);
        background: linear-gradient(180deg, rgba(255, 255, 255, 0.04), rgba(255, 255, 255, 0.01));
        border: 1px solid rgba(255, 255, 255, 0.06);
        border-radius: 0.9rem;
        transition:
            transform 160ms ease,
            box-shadow 160ms ease,
            background-color 160ms ease,
            color 160ms ease,
            border-color 160ms ease;
    }

    .tab:hover {
        transform: translateY(-1px);
        color: var(--text);
        border-color: rgba(255, 255, 255, 0.12);
        box-shadow: 0 10px 24px rgba(0, 0, 0, 0.18);
    }

    .tab:focus-visible {
        outline: 2px solid rgba(66, 153, 225, 0.9);
        outline-offset: 2px;
    }

    .tab.active {
        color: var(--text);
        background:
            radial-gradient(circle at top left, rgba(74, 163, 255, 0.28), transparent 55%),
            linear-gradient(135deg, rgba(0, 97, 200, 0.95), rgba(0, 132, 255, 0.7));
        border-color: rgba(108, 184, 255, 0.45);
        box-shadow:
            inset 0 1px 0 rgba(255, 255, 255, 0.18),
            0 14px 30px rgba(0, 97, 200, 0.28);
    }

    .tab_label {
        font-size: 0.95rem;
        font-weight: 700;
        letter-spacing: 0.01em;
        white-space: nowrap;
    }
</style>