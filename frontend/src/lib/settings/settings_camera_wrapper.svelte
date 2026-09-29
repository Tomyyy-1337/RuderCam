<div class="camera-settings">
    <button
        type="button"
        class="summary"
        aria-expanded={open}
        aria-controls="camera-settings-content"
        onclick={() => open = !open}
    >
        <span class="summary-copy">
            <span class="title">Kamera-Einstellungen</span>
            <span class="description">Fokus, Stream und Belichtung</span>
        </span>
        <span class="chevron" aria-hidden="true"></span>
    </button>

    <div id="camera-settings-content" class:expanded={open} class="settings-content" aria-hidden={!open}>
        <div class="settings-content-inner">
            <SettingsCamera />
        </div>
    </div>
</div>

<script lang="ts">
    import SettingsCamera from "./settings_camera.svelte";

    let open = $state(false);
</script>

<style>
    .camera-settings {
        border: 1px solid color-mix(in srgb, var(--text) 14%, transparent);
        border-radius: 1rem;
        background: color-mix(in srgb, var(--section-background) 70%, var(--background) 30%);
        overflow: hidden;
    }

    .summary {
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

    .summary:focus-visible {
        outline: 2px solid var(--button-color);
        outline-offset: -2px;
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
        transform: rotate(45deg) translateY(-0.15rem);
        transition: transform 700ms ease;
    }

    .camera-settings:has(.settings-content.expanded) .chevron {
        transform: rotate(225deg) translate(-0.1rem, -0.1rem);
    }

    .settings-content {
        display: grid;
        grid-template-rows: 0fr;
        padding: 0 0.65rem;
        transition:
            grid-template-rows 700ms ease,
            padding-bottom 700ms ease;
    }

    .settings-content.expanded {
        grid-template-rows: 1fr;
        padding-bottom: 0.65rem;
    }

    .settings-content-inner {
        min-height: 0;
        overflow: hidden;
        opacity: 0;
        transform: translateY(-0.5rem);
        transition:
            opacity 500ms ease,
            transform 700ms ease;
    }

    .settings-content.expanded .settings-content-inner {
        opacity: 1;
        transform: translateY(0);
    }
</style>