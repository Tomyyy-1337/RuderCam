<div class="heading">
    <h2>Verfügbare Versionen</h2>
</div>

{#if is_loading}
    <p class="status">Versionen werden geladen...</p>
{:else if !internet_connected}
    <p>Um eine neue Version herunter zu laden verbinden sie ihr Smartphone mit dem Internet. Um ein Update zu installieren, verbinden sie ihr Smartphone mit der Kamera.</p>
{:else if releases.length === 0}
    <p class="status">Keine Versionen verfügbar.</p>
{:else}
    <p>Laden sie die gewünschte Firmware Version herunter und stellen sie anschließend die Verbindung zur Kamera wieder her um das Update durchzuführen.</p>

    <ReleaseCard release={latest_release} latest />

    <details>
        <summary>Alle Versionen anzeigen</summary>
        <ul>
            {#each releases as release}
                <ReleaseCard {release} />
            {/each}
        </ul>
    </details>
{/if}

<div class="spacer"></div>

<div class="heading">
    <h2>Upload Firmware-Update</h2>
</div>
{#if temporary_state.is_connected}
    <div class="upload-panel">
        <label for="update-file">Firmware-Update auswählen</label>
        <input id="update-file" type="file" accept=".tar,.tar.gz,.tgz" onchange={select_update_file} />
        <button type="button" onclick={upload_update} disabled={!update_file || is_uploading}>
            {is_uploading ? "Wird hochgeladen..." : "Update durchführen"}
        </button>
        {#if upload_status}<p class="upload-status" role="status">{upload_status}</p>{/if}
    </div>
{:else}
    <p>Verbinden sie ihr Smartphone mit der Kamera, um ein Firmware-Update hochzuladen.</p>
{/if}

<Dialog
    bind:open={dialog_open}
    title={dialog_title}
    message={dialog_message}
    confirmLabel="Schließen"
    showCancel={false}
    showConfirm={!is_uploading}
    dismissible={!is_uploading}
    tone={dialog_tone}
    onConfirm={close_dialog}
/>

<script lang="ts">
    import { onMount } from "svelte";
    import Dialog from "../components/dialog.svelte";
    import ReleaseCard, { type Release } from "./release_card.svelte";
    import { temporary_state } from "../classes/temporary_state_store.svelte";

    const RELEASES_URL = "https://rudercam.passberg.net/updates/";

    let releases = $state<Release[]>([]);
    let is_loading = $state(true);
    let internet_connected = $state(false);
    let latest_release = $derived(releases[0]);
    let update_file = $state<File | null>(null);
    let upload_status = $state("");
    let is_uploading = $state(false);
    let dialog_open = $state(false);
    let dialog_title = $state("");
    let dialog_message = $state("");
    let dialog_tone = $state<"info" | "warning" | "danger">("info");
    let update_succeeded = $state(false);

    const select_update_file = (event: Event) => {
        const input = event.currentTarget as HTMLInputElement;
        update_file = input.files?.[0] ?? null;
        upload_status = "";
    };

    const upload_update = async () => {
        if (!update_file) return;

        is_uploading = true;
        upload_status = "Update wird hochgeladen...";
        open_dialog("Update wird durchgeführt", "Datei wird vorbereitet...", "info");

        try {
            let arrayBuffer = await update_file.arrayBuffer();
            await upload_archive(arrayBuffer);

            update_file = null;
            upload_status = "Update erfolgreich hochgeladen.";
            dialog_title = "Update erfolgreich";
            dialog_message = "Das Firmware-Update wurde erfolgreich installiert.";
            dialog_tone = "info";
            update_succeeded = true;
        } catch (error) {
            const message = error instanceof Error ? error.message : "Update fehlgeschlagen.";
            upload_status = message;
            dialog_title = "Update fehlgeschlagen";
            dialog_message = message;
            dialog_tone = "danger";
            update_succeeded = false;
        } finally {
            is_uploading = false;
        }
    };

    const upload_archive = (arrayBuffer: ArrayBuffer): Promise<void> =>
        new Promise((resolve, reject) => {
            const request = new XMLHttpRequest();
            request.open("POST", `http://${window.location.hostname}:4000/api/update`);
            request.setRequestHeader("Content-Type", "application/octet-stream");

            request.upload.onprogress = (event) => {
                if (!event.lengthComputable) {
                    dialog_message = "Update wird hochgeladen...";
                    return;
                }

                const progress = Math.round((event.loaded / event.total) * 100);
                if (progress >= 100) {
                    dialog_title = "Update wird installiert";
                    dialog_message = "Update zu 100% hochgeladen. Update wird installiert...";
                } else {
                    dialog_message = `Update wird hochgeladen... ${progress}%`;
                }
            };

            request.onload = () => {
                if (request.status >= 200 && request.status < 300) {
                    resolve();
                } else {
                    reject(new Error(`Update fehlgeschlagen (${request.status})`));
                }
            };
            request.onerror = () => reject(new Error("Update konnte nicht übertragen werden."));
            request.send(arrayBuffer);
        });

    const open_dialog = (title: string, message: string, tone: "info" | "warning" | "danger") => {
        dialog_title = title;
        dialog_message = message;
        dialog_tone = tone;
        dialog_open = true;
    };

    const close_dialog = () => {
        if (!is_uploading) {
            dialog_open = false;

            if (update_succeeded) {
                window.setTimeout(() => window.location.reload(), 2000);
            }
        }
    };

    const fetch_releases = async () => {
        is_loading = true;
        internet_connected = true;

        try {
            const response = await fetch(RELEASES_URL, { cache: "no-store" });

            if (!response.ok) {
                throw new Error(`Firmware server returned ${response.status}`);
            }

            const document = new DOMParser().parseFromString(await response.text(), "text/html");
            releases = [...document.querySelectorAll<HTMLAnchorElement>("a[href]")]
                .map((link): Release | null => {
                    const url = new URL(link.getAttribute("href")!, RELEASES_URL);
                    const name = decodeURIComponent(url.pathname.split("/").pop() || "");

                    if (url.pathname.startsWith("/updates/") && /\.(tar|tar\.gz|tgz)$/i.test(name)) {
                        return {
                            name,
                            url: url.href,
                        };
                    }

                    return null;
                })
                .filter((release): release is Release => release !== null)
                .sort((a, b) => b.name.localeCompare(a.name, undefined, { numeric: true }));
        } catch (error) {
            console.error("Error fetching available firmware versions:", error);
            internet_connected = false;
        } finally {
            is_loading = false;
        }
    };

    onMount(() => {
        const handle_online = () => {
            void fetch_releases();
        };

        window.addEventListener("online", handle_online);

        if (navigator.onLine) {
            void fetch_releases();
        } else {
            internet_connected = false;
            is_loading = false;
        }

        return () => {
            window.removeEventListener("online", handle_online);
        };
    });
</script>

<style>
    .heading {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
    }

    h2 {
        margin: 0;
        font-size: 1.1rem;
    }

    .upload-panel {
        display: grid;
        gap: 0.6rem;
        margin: 1rem 0;
        padding: 0.9rem;
        border: 1px solid color-mix(in srgb, var(--text) 18%, transparent);
        border-radius: 0.4rem;
    }

    .upload-panel label {
        font-weight: 600;
    }

    .upload-panel input {
        min-width: 0;
        height: 3rem;
        min-height: 0;
        margin-bottom: 0;
        padding: 0.25rem;
        border: 1px solid color-mix(in srgb, var(--text) 18%, transparent);
        border-radius: 0.5rem;
        background: var(--section-background);
        color: var(--text);
        font: inherit;
    }

    .upload-panel input::file-selector-button {
        margin-right: 0.65rem;
        height: 100%;
        box-sizing: border-box;
        padding: 0.3rem 0.8rem;
        border: 1px solid color-mix(in srgb, var(--button-color) 48%, var(--text) 10%);
        border-radius: 0.35rem;
        background: var(--section-background);
        color: var(--text);
        cursor: pointer;
        font: inherit;
        font-size: 0.75rem;
    }

    .upload-panel input:focus-visible {
        outline: 2px solid color-mix(in srgb, var(--button-color) 88%, white 12%);
        outline-offset: 2px;
    }

    .upload-panel button:disabled {
        cursor: not-allowed;
        opacity: 0.55;
    }

    .upload-status {
        margin: 0;
        color: color-mix(in srgb, var(--text) 72%, transparent);
        font-size: 0.9rem;
    }

    ul {
        display: grid;
        gap: 0.65rem;
        margin: 0.8rem 0 0;
        padding: 0;
        list-style: none;
    }

    details {
        margin-top: 0.8rem;
    }

    summary {
        cursor: pointer;
        color: var(--button-color);
        font-weight: 600;
    }

    .status {
        color: color-mix(in srgb, var(--text) 68%, transparent);
        font-size: 0.9rem;
    }

    @media (max-width: 520px) {
    }
</style>