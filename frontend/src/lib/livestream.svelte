<section>
    <div class="video-shell" bind:this={videoShell}>
        <video id="myvideo" bind:this={videoElement} controls muted autoplay playsinline></video>
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
    let videoElement: HTMLVideoElement | null = null;
    let reader: MediaMTXWebRTCReader | null = null;
    let retryTimer: number | null = null;
    let playRetryTimer: number | null = null;
    let reconnectGeneration = 0;

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
            reconnectGeneration++;

            if (retryTimer !== null) {
                clearTimeout(retryTimer);
                retryTimer = null;
            }

            if (playRetryTimer !== null) {
                clearTimeout(playRetryTimer);
                playRetryTimer = null;
            }

            destroyReader();
            resetVideo();
        };

        const handleVisibilityChange = () => {
            if (document.visibilityState !== "visible") {
                return;
            }

            // A hidden tab can leave WebRTC/video in a stale state.
            hardReconnect(0);
        };

        document.addEventListener("fullscreenchange", handleFullscreenChange);
        document.addEventListener("visibilitychange", handleVisibilityChange);
        window.addEventListener("beforeunload", handleBeforeUnload);

        const video = document.getElementById("myvideo");

        if (video instanceof HTMLVideoElement) {
            video.removeAttribute("controls");
        }

        return () => {
            document.removeEventListener("fullscreenchange", handleFullscreenChange);
            document.removeEventListener("visibilitychange", handleVisibilityChange);
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

    function resetVideo() {
        if (videoElement === null) {
            return;
        }

        if (playRetryTimer !== null) {
            clearTimeout(playRetryTimer);
            playRetryTimer = null;
        }

        try {
            videoElement.pause();
        } catch {
        }

        // This is important. Do not just assign another MediaStream.
        // Completely detach the old media pipeline.
        videoElement.srcObject = null;
        videoElement.removeAttribute("src");

        try {
            videoElement.load();
        } catch {
        }

        videoElement.currentTime = 0;
    }

    function destroyReader() {
        if (reader !== null) {
            try {
                reader.close();
            } catch {
            }

            reader = null;
        }
    }

    function hardReconnect(delay = 500) {
        reconnectGeneration++;

        const generation = reconnectGeneration;

        if (retryTimer !== null) {
            clearTimeout(retryTimer);
            retryTimer = null;
        }

        if (playRetryTimer !== null) {
            clearTimeout(playRetryTimer);
            playRetryTimer = null;
        }

        destroyReader();
        resetVideo();

        window.setTimeout(() => {
            if (generation !== reconnectGeneration) {
                return;
            }

            if (document.visibilityState !== "visible") {
                return;
            }

            connectReader(generation);
        }, delay);
    }

    async function connectReader(expectedGeneration = reconnectGeneration): Promise<void> {
        if (expectedGeneration !== reconnectGeneration) {
            return;
        }

        if (retryTimer !== null) {
            clearTimeout(retryTimer);
            retryTimer = null;
        }

        destroyReader();

        const generation = expectedGeneration;

        try {
            const newReader = new MediaMTXWebRTCReader({
                url: "http://192.168.50.1:8889/stream/whep",
                user: "",
                pass: "",
                token: "",

                onError: (err) => {
                    console.error("MediaMTX error:", err);

                    if (generation !== reconnectGeneration) {
                        return;
                    }

                    resetVideo();

                    // The reader itself normally retries runtime failures.
                    // For a high-latency reconnect we deliberately perform a
                    // complete browser-side reset instead.
                    if (err.includes("video latency too high")) {
                        hardReconnect(500);
                        return;
                    }

                    if (err.includes("retrying in some seconds")) {
                        return;
                    }

                    hardReconnect(3000);
                },

                onTrack: (evt) => {
                    if (generation !== reconnectGeneration) {
                        return;
                    }

                    const track = evt.track;

                    console.log("[WebRTC] track:", track.kind, track.id);

                    if (videoElement === null) {
                        return;
                    }

                    // Build our own MediaStream instead of blindly using
                    // evt.streams[0]. This guarantees that the new connection
                    // gets its own fresh stream object.
                    let stream = videoElement.srcObject instanceof MediaStream
                        ? videoElement.srcObject
                        : null;

                    if (stream === null || stream.getTracks().some((t) => t.id === track.id) === false) {
                        stream = new MediaStream();

                        for (const existingTrack of evt.streams[0]?.getTracks() ?? []) {
                            stream.addTrack(existingTrack);
                        }

                        if (stream.getTracks().some((t) => t.id === track.id) === false) {
                            stream.addTrack(track);
                        }

                        videoElement.srcObject = stream;
                    }

                    const attemptPlay = () => {
                        if (generation !== reconnectGeneration) {
                            return;
                        }

                        if (videoElement === null) {
                            return;
                        }

                        if (playRetryTimer !== null) {
                            clearTimeout(playRetryTimer);
                            playRetryTimer = null;
                        }

                        videoElement.play().catch(() => {
                            playRetryTimer = window.setTimeout(attemptPlay, 500);
                        });
                    };

                    attemptPlay();

                    track.onended = () => {
                        if (generation !== reconnectGeneration) {
                            return;
                        }

                        console.warn("[WebRTC] track ended");

                        hardReconnect(250);
                    };
                },

                onDataChannel: (evt) => {
                    if (generation !== reconnectGeneration) {
                        return;
                    }

                    evt.channel.binaryType = "arraybuffer";

                    evt.channel.onmessage = (messageEvent) => {
                        console.log("data channel message", messageEvent.data);
                    };
                },
            });

            if (generation !== reconnectGeneration) {
                newReader.close();
                return;
            }

            reader = newReader;
        } catch (err) {
            console.error("connectReader exception:", err);

            if (generation !== reconnectGeneration) {
                return;
            }

            retryTimer = window.setTimeout(() => {
                retryTimer = null;
                connectReader(generation);
            }, 3000);
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