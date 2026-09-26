<script lang="ts">
    export type Release = {
        name: string;
        url: string;
    };

    let { release, latest = false }: { release: Release; latest?: boolean } = $props();
</script>

<li class:latest-release={latest}>
    <div class="release-content">
        <div class="release-info">
            {#if latest}<span class="latest-label">Neuste Firmware</span>{/if}
            <span class="release-name">
                {release.name.replace(/\.tar(?:\.gz|\.tgz)?$/i, "")}
            </span>
        </div>
        <a
            class="download-button"
            href={release.url}
            download={release.name}
            aria-label={`Download ${release.name}`}
        >
            Download
        </a>
    </div>
</li>

<style>
    li {
        display: grid;
        gap: 1rem;
        padding: 0.8rem;
        border: 1px solid color-mix(in srgb, var(--text) 18%, transparent);
        border-radius: 0.4rem;
    }

    .latest-release {
        gap: 0.45rem;
        margin-top: 0.8rem;
        padding: 0.9rem;
        border: 2px solid var(--button-color);
    }

    .latest-label {
        font-size: 0.8rem;
        font-weight: 700;
        text-transform: uppercase;
    }

    .release-content {
        display: grid;
        grid-template-columns: minmax(0, 1fr) auto;
        width: 100%;
        box-sizing: border-box;
        align-items: center;
        gap: 1rem;
    }

    .release-info {
        display: grid;
        gap: 0.2rem;
    }

    .release-name {
        font-weight: 600;
    }

    .download-button {
        display: inline-flex;
        align-self: center;
        align-items: center;
        justify-content: center;
        min-height: 2.5rem;
        box-sizing: border-box;
        padding: 0.55rem 0.8rem;
        border: 1px solid color-mix(in srgb, var(--button-color) 48%, var(--text) 10%);
        border-radius: 0.5rem;
        background: color-mix(in srgb, var(--button-color) 18%, var(--section-background));
        color: var(--button-color);
        cursor: pointer;
        font: inherit;
        text-decoration: none;
    }

    .download-button:hover {
        color: var(--text);
        background: color-mix(in srgb, var(--button-color) 28%, var(--section-background));
    }

    .download-button:focus-visible {
        outline: 2px solid color-mix(in srgb, var(--button-color) 88%, white 12%);
        outline-offset: 2px;
    }

    @media (max-width: 520px) {
        .release-content {
            min-width: 0;
        }

        .release-info {
            min-width: 0;
        }
    }
</style>
