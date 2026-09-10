<section>
    <div class="video-shell" bind:this={videoShell}>
        <video id="myvideo" controls muted autoplay playsinline></video>
        {#if fullscreen}
           <Overlay {deviceStatus} bind:activeSession {fahrtenbuch} {overlay_settings} />
        {:else}
            <button
                onclick={activateFullscreen}
                class="fullscreen-btn"
                type="button"
                title="Vollbild"
                aria-label="Vollbild"
            >
                <span class="fullscreen-icon" aria-hidden="true"></span>
                <span>Vollbild</span>
            </button>
        {/if}
    </div>
</section>

<script lang="ts">
    import { onMount } from "svelte";
    import Overlay from "./overlay.svelte";
    import { MediaMTXWebRTCReader } from "./reader";
    import type { ActiveSession, DeviceStatus, OverlaySettings } from "./types";
    import type { Writable } from "svelte/store";
    import type { FahrtenbuchStore } from "./fahrtenbuchStore";

    let videoShell: HTMLDivElement | null = null;
    let reader: MediaMTXWebRTCReader | null = null;
    let retryTimer: number | null = null;

    let fullscreen = $state(false);
    let {
        deviceStatus,
        activeSession = $bindable(),
        fahrtenbuch,
        overlay_settings,
    }: {
        deviceStatus: DeviceStatus;
        activeSession: ActiveSession;
        fahrtenbuch: Writable<FahrtenbuchStore>;
        overlay_settings: OverlaySettings;
    } = $props();

    onMount(() => {
        connectReader();

        const handleFullscreenChange = () => {
            fullscreen = document.fullscreenElement === videoShell;

            if (!fullscreen) {
                screen.orientation?.unlock?.();
            }
        };

        const handleBeforeUnload = () => {
            if (retryTimer !== null) {
                clearTimeout(retryTimer);
            }

            if (reader !== null) {
                try {
                    reader.close();
                } catch {
                }
                reader = null;
            }
        };

        document.addEventListener("fullscreenchange", handleFullscreenChange);
        window.addEventListener("beforeunload", handleBeforeUnload);

        const video = document.getElementById("myvideo");
        if (video instanceof HTMLVideoElement) {
            video.removeAttribute("controls");
        }

        return () => {
            document.removeEventListener("fullscreenchange", handleFullscreenChange);
            window.removeEventListener("beforeunload", handleBeforeUnload);
            handleBeforeUnload();
        };
    });

    const activateFullscreen = async () => {
        if (videoShell?.requestFullscreen) {
            await videoShell.requestFullscreen();

            try {
                const orientation = screen.orientation as ScreenOrientation & {
                    lock?: (value: string) => Promise<void>;
                };
                await orientation.lock?.("landscape");
            } catch (err) {
                console.warn("Could not lock orientation to landscape", err);
            }
        }
    };

    async function connectReader(): Promise<void> {
        if (retryTimer !== null) {
            clearTimeout(retryTimer);
            retryTimer = null;
        }

        try {
            reader = new MediaMTXWebRTCReader({
                url: "http://192.168.50.1:8889/stream/whep",
                user: "",
                pass: "",
                token: "",
                onError: (err) => {
                    console.error("MediaMTX error:", err);
                    try {
                        reader?.close();
                    } catch {
                    }
                    reader = null;
                    retryTimer = window.setTimeout(connectReader, 3000);
                },
                onTrack: (evt) => {
                    if (retryTimer !== null) {
                        clearTimeout(retryTimer);
                        retryTimer = null;
                    }

                    const video = document.getElementById("myvideo");
                    if (video instanceof HTMLVideoElement) {
                        video.srcObject = evt.streams[0] ?? null;
                    }
                },
                onDataChannel: (evt) => {
                    evt.channel.binaryType = "arraybuffer";
                    evt.channel.onmessage = (messageEvent) => {
                        console.log("data channel message", messageEvent.data);
                    };
                },
            });
        } catch (err) {
            console.error("connectReader exception:", err);
            retryTimer = window.setTimeout(connectReader, 3000);
        }
    }
</script>

<style>
    section {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.2rem;
        padding: 1rem;
    }

    .video-shell {
        width: 80%;
        aspect-ratio: 16 / 9;
        position: relative;
        display: grid;
        place-items: center;
        background: black;
        border-radius: 0.75rem;
        overflow: hidden;
    }

    .video-shell video {
        width: 100%;
        height: 100%;
        display: block;
        object-fit: cover;
        object-position: center;
        background: black;
    }

    .video-shell::backdrop {
        background: black;
    }

    .video-shell:fullscreen {
        width: 100vw;
        height: 100vh;
        max-width: none;
        max-height: none;
        aspect-ratio: auto;
        place-items: center;
        background: black;
        border-radius: 0;
        overflow: visible;
    }

    .video-shell:fullscreen video {
        width: 100%;
        height: 100%;
        max-height: 100vh;
        max-width: 100vw;
        object-fit: contain;
    }

    .fullscreen-btn {
        position: absolute;
        right: 0.75rem;
        bottom: 0.75rem;
        z-index: 5;
        display: inline-flex;
        align-items: center;
        gap: 0.45rem;
        padding: 0.5rem 0.8rem;
        font-size: 0.92rem;
        font-weight: 600;
        letter-spacing: 0.01em;
        color: #f2f5f8;
        background: rgba(18, 20, 22, 0.62);
        border: 1px solid rgba(255, 255, 255, 0.2);
        backdrop-filter: blur(6px);
        -webkit-backdrop-filter: blur(6px);
        border-radius: 999px;
        cursor: pointer;
        transition:
            transform 120ms ease,
            background-color 180ms ease,
            border-color 180ms ease,
            box-shadow 180ms ease;
    }

    .fullscreen-btn:hover {
        background: rgba(26, 28, 31, 0.82);
        border-color: rgba(255, 255, 255, 0.34);
        box-shadow: 0 10px 24px rgba(0, 0, 0, 0.35);
    }

    .fullscreen-btn:active {
        transform: translateY(1px) scale(0.985);
    }

    .fullscreen-btn:focus-visible {
        outline: 2px solid #7bc0ff;
        outline-offset: 2px;
    }

    .fullscreen-icon {
        width: 0.95rem;
        height: 0.95rem;
        position: relative;
        display: inline-block;
    }

    .fullscreen-icon::before,
    .fullscreen-icon::after {
        content: "";
        position: absolute;
        inset: 0;
        border: 2px solid currentColor;
        border-radius: 2px;
    }

    .fullscreen-icon::before {
        clip-path: polygon(0 0, 38% 0, 38% 12%, 12% 12%, 12% 38%, 0 38%);
    }

    .fullscreen-icon::after {
        clip-path: polygon(100% 100%, 62% 100%, 62% 88%, 88% 88%, 88% 62%, 100% 62%);
    }

    .video-shell:fullscreen .fullscreen-btn {
        left: 1rem;
        bottom: 1rem;
    }

    @media (max-width: 768px) {
        section {
            padding: 0.75rem;
        }

        .video-shell {
            width: 100%;
        }

        .video-shell video {
            object-fit: contain;
        }
    }
</style>