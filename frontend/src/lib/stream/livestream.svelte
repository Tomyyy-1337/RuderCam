<section>
    <div class="video-shell" class:ios-virtual-fullscreen={iosVirtualFullscreen} bind:this={videoShell}>
        <video id="myvideo" bind:this={videoElement} controls muted autoplay playsinline></video>
        {#if fullscreen}
            <Overlay {deviceStatus} bind:activeSession {fahrtenbuch} {overlay_settings} />
            {#if iosVirtualFullscreen}
                <button
                    onclick={exitIOSVirtualFullscreen}
                    class="exit-fullscreen-btn"
                    type="button"
                    title="Vollbild beenden"
                    aria-label="Vollbild beenden"
                >&times;</button>
            {/if}
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
            {#if showIOSInstallHint}
                <div class="ios-install-hint">
                    <button
                        onclick={() => (showIOSInstallHint = false)}
                        class="ios-install-hint-close"
                        type="button"
                        title="Schließen"
                        aria-label="Schließen"
                    >&times;</button>
                    <p>
                        Zum installieren auf "Teilen" tippen und "Zum Home-Bildschirm" auswählen.
                        Danach kann die App im Vollbildmodus genutzt werden.
                    </p>
                </div>
            {/if}
        {/if}
    </div>
</section>

<script lang="ts">
    import { onMount } from "svelte";
    import Overlay from "./overlay.svelte";
    import { MediaMTXWebRTCReader } from "./reader";
    import type { ActiveSession, DeviceStatus, OverlaySettings } from "../types";
    import type { Writable } from "svelte/store";
    import type { FahrtenbuchStore } from "../fahrtenbuch/fahrtenbuchStore";

    let videoShell: HTMLDivElement | null = null;
    let videoElement: HTMLVideoElement | null = null;
    let reader: MediaMTXWebRTCReader | null = null;
    let retryTimer: number | null = null;
    let playRetryTimer: number | null = null;
    let healthTimer: number | null = null;
    let reconnectGeneration = 0;

    let reconnectAttempts = 0;
    let lastProgressAt = 0;
    let lastObservedVideoTime = 0;
    let waitingSince: number | null = null;
    let stablePlaybackSince: number | null = null;
    let streamAttachedAt = 0;

    const MAX_RECONNECT_DELAY_MS = 4000;
    const PLAY_RETRY_DELAY_MS = 250;
    const HEALTH_CHECK_INTERVAL_MS = 400;
    const STARTUP_GRACE_MS = 1200;
    const FREEZE_NO_PROGRESS_MS = 1400;
    const WAITING_TIMEOUT_MS = 1000;
    const STABLE_RECOVERY_MS = 2500;

    let fullscreen = $state(false);
    let showIOSInstallHint = $state(false);
    let iosVirtualFullscreen = $state(false);

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
        setupVideoWatchdog();
        connectReader();
        showIOSInstallHint = isIOSSafariBrowser();

        const handleKeyDown = (event: KeyboardEvent) => {
            if (event.key === "Escape" && iosVirtualFullscreen) {
                exitIOSVirtualFullscreen();
            }
        };

        const handlePopState = () => {
            if (iosVirtualFullscreen) {
                exitIOSVirtualFullscreen();
            }
        };

        const handleFullscreenChange = () => {
            fullscreen = document.fullscreenElement === videoShell;

            if (!fullscreen) {
                screen.orientation?.unlock?.();
            }
        };

        const handleBeforeUnload = () => {
            reconnectGeneration++;
            clearAllTimers();
            destroyReader();
            resetVideo();
        };

        const handleVisibilityChange = () => {
            if (document.visibilityState !== "visible") {
                return;
            }

            hardReconnect(0, "tab-visible");
        };

        const handleOnline = () => {
            hardReconnect(0, "network-online");
        };

        const handleOffline = () => {
            hardReconnect(100, "network-offline");
        };

        let cleanupVideoListeners: (() => void) | null = null;

        if (videoElement !== null) {
            cleanupVideoListeners = attachVideoEventListeners(videoElement);
        }

        document.addEventListener("fullscreenchange", handleFullscreenChange);
        document.addEventListener("keydown", handleKeyDown);
        document.addEventListener("visibilitychange", handleVisibilityChange);
        window.addEventListener("online", handleOnline);
        window.addEventListener("offline", handleOffline);
        window.addEventListener("popstate", handlePopState);
        window.addEventListener("beforeunload", handleBeforeUnload);

        const video = document.getElementById("myvideo");

        if (video instanceof HTMLVideoElement) {
            video.removeAttribute("controls");
        }

        return () => {
            document.removeEventListener("fullscreenchange", handleFullscreenChange);
            document.removeEventListener("keydown", handleKeyDown);
            document.removeEventListener("visibilitychange", handleVisibilityChange);
            window.removeEventListener("online", handleOnline);
            window.removeEventListener("offline", handleOffline);
            window.removeEventListener("popstate", handlePopState);
            window.removeEventListener("beforeunload", handleBeforeUnload);
            cleanupVideoListeners?.();
            handleBeforeUnload();
        };
    });

    const activateFullscreen = async () => {
        if (videoShell === null) {
            return;
        }

        const orientation = screen.orientation as ScreenOrientation & {
            lock?: (value: string) => Promise<void>;
        };

        // iOS never implements Element.requestFullscreen(), not even for standalone
        // home-screen apps, so fall back to a manually toggled fullscreen layout there.
        if (isIOSDevice()) {
            iosVirtualFullscreen = true;
            fullscreen = true;
            window.history.pushState({ ...window.history.state, videoFullscreen: true }, "", window.location.href);
            await orientation.lock?.("landscape").catch(() => {});
            return;
        }

        try {
            await videoShell.requestFullscreen();
            await orientation.lock?.("landscape");
        } catch (err) {
            console.warn("Could not open fullscreen video", err);
        }
    };

    function exitIOSVirtualFullscreen() {
        iosVirtualFullscreen = false;
        fullscreen = false;
        screen.orientation?.unlock?.();
    }

    function isIOSDevice(): boolean {
        return /iP(hone|od|ad)/.test(navigator.userAgent)
            || (navigator.platform === "MacIntel" && navigator.maxTouchPoints > 1);
    }

    // Neither Safari nor standalone home-screen apps support requestFullscreen() on iOS;
    // the only way to get a chromeless view there is adding the page to the Home Screen.
    function isIOSSafariBrowser(): boolean {
        const runningStandalone = (navigator as Navigator & { standalone?: boolean }).standalone === true
            || window.matchMedia("(display-mode: standalone)").matches;

        return isIOSDevice() && !runningStandalone;
    }

    function resetVideo() {
        if (videoElement === null) {
            return;
        }

        if (playRetryTimer !== null) {
            clearTimeout(playRetryTimer);
            playRetryTimer = null;
        }

        waitingSince = null;
        streamAttachedAt = 0;
        stablePlaybackSince = null;
        lastObservedVideoTime = 0;
        lastProgressAt = 0;

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

    function clearAllTimers() {
        if (retryTimer !== null) {
            clearTimeout(retryTimer);
            retryTimer = null;
        }

        if (playRetryTimer !== null) {
            clearTimeout(playRetryTimer);
            playRetryTimer = null;
        }

        if (healthTimer !== null) {
            clearTimeout(healthTimer);
            healthTimer = null;
        }
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

    function nextReconnectDelay(baseDelayMs: number): number {
        const exponent = Math.min(reconnectAttempts, 4);
        const delay = Math.min(baseDelayMs * 2 ** exponent, MAX_RECONNECT_DELAY_MS);
        reconnectAttempts += 1;
        return delay;
    }

    function markPlaybackProgress() {
        const now = performance.now();

        lastProgressAt = now;
        waitingSince = null;

        if (stablePlaybackSince === null) {
            stablePlaybackSince = now;
            return;
        }

        if (now - stablePlaybackSince >= STABLE_RECOVERY_MS) {
            reconnectAttempts = 0;
        }
    }

    function scheduleHealthCheck() {
        if (healthTimer !== null) {
            clearTimeout(healthTimer);
            healthTimer = null;
        }

        healthTimer = window.setTimeout(() => {
            healthTimer = null;

            const video = videoElement;

            if (video === null || document.visibilityState !== "visible") {
                scheduleHealthCheck();
                return;
            }

            if (video.srcObject instanceof MediaStream && !video.paused && !video.ended) {
                const now = performance.now();
                const mediaAge = streamAttachedAt === 0 ? 0 : now - streamAttachedAt;

                if (video.currentTime > lastObservedVideoTime + 0.02) {
                    lastObservedVideoTime = video.currentTime;
                    markPlaybackProgress();
                } else if (
                    mediaAge > STARTUP_GRACE_MS &&
                    lastProgressAt !== 0 &&
                    now - lastProgressAt > FREEZE_NO_PROGRESS_MS
                ) {
                    hardReconnect(nextReconnectDelay(150), "video-no-progress");
                    return;
                }

                if (
                    waitingSince !== null &&
                    mediaAge > STARTUP_GRACE_MS &&
                    now - waitingSince > WAITING_TIMEOUT_MS
                ) {
                    hardReconnect(nextReconnectDelay(150), "video-waiting");
                    return;
                }
            }

            scheduleHealthCheck();
        }, HEALTH_CHECK_INTERVAL_MS);
    }

    function setupVideoWatchdog() {
        scheduleHealthCheck();
    }

    function attachVideoEventListeners(video: HTMLVideoElement) {
        const onTimeUpdate = () => {
            if (video.currentTime > lastObservedVideoTime + 0.01) {
                lastObservedVideoTime = video.currentTime;
                markPlaybackProgress();
            }
        };

        const onPlaying = () => {
            markPlaybackProgress();
        };

        const onWaiting = () => {
            if (waitingSince === null) {
                waitingSince = performance.now();
            }
        };

        const onStalled = () => {
            if (waitingSince === null) {
                waitingSince = performance.now();
            }
        };

        const onEnded = () => {
            hardReconnect(nextReconnectDelay(100), "video-ended");
        };

        video.addEventListener("timeupdate", onTimeUpdate);
        video.addEventListener("playing", onPlaying);
        video.addEventListener("waiting", onWaiting);
        video.addEventListener("stalled", onStalled);
        video.addEventListener("ended", onEnded);

        return () => {
            video.removeEventListener("timeupdate", onTimeUpdate);
            video.removeEventListener("playing", onPlaying);
            video.removeEventListener("waiting", onWaiting);
            video.removeEventListener("stalled", onStalled);
            video.removeEventListener("ended", onEnded);
        };
    }

    function hardReconnect(delay = 500, reason = "unspecified") {
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

        waitingSince = null;

        destroyReader();
        resetVideo();

        console.warn("[WebRTC] reconnect", { reason, delay, reconnectAttempts });

        retryTimer = window.setTimeout(() => {
            retryTimer = null;

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

        if (!navigator.onLine) {
            retryTimer = window.setTimeout(() => {
                retryTimer = null;
                connectReader(expectedGeneration);
            }, nextReconnectDelay(400));
            return;
        }

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

                    const lowered = err.toLowerCase();

                    if (lowered.includes("stream not found")) {
                        hardReconnect(nextReconnectDelay(1200), "stream-not-found");
                        return;
                    }

                    if (lowered.includes("latency too high") || lowered.includes("video stalled")) {
                        hardReconnect(nextReconnectDelay(120), "reader-health");
                        return;
                    }

                    hardReconnect(nextReconnectDelay(500), "reader-error");
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

                    streamAttachedAt = performance.now();
                    stablePlaybackSince = null;
                    lastObservedVideoTime = 0;
                    lastProgressAt = streamAttachedAt;
                    waitingSince = null;

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

                        videoElement.play().then(() => {
                            markPlaybackProgress();
                        }).catch(() => {
                            playRetryTimer = window.setTimeout(attemptPlay, PLAY_RETRY_DELAY_MS);
                        });
                    };

                    attemptPlay();

                    track.onended = () => {
                        if (generation !== reconnectGeneration) {
                            return;
                        }

                        console.warn("[WebRTC] track ended");

                        hardReconnect(nextReconnectDelay(100), "track-ended");
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
            }, nextReconnectDelay(500));
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

    .video-shell:fullscreen,
    .video-shell.ios-virtual-fullscreen {
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

    .video-shell.ios-virtual-fullscreen {
        position: fixed;
        inset: 0;
        z-index: 1000;
        height: 100dvh;
    }

    .video-shell:fullscreen video,
    .video-shell.ios-virtual-fullscreen video {
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

    .exit-fullscreen-btn {
        position: fixed;
        top: calc(0.75rem + env(safe-area-inset-top));
        right: calc(0.75rem + env(safe-area-inset-right));
        z-index: 2147483647;
        width: 3rem;
        height: 3rem;
        padding: 0;
        color: #f2f5f8;
        background: rgba(18, 20, 22, 0.84);
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 50%;
        font-size: 1.8rem;
        line-height: 1;
        cursor: pointer;
        touch-action: manipulation;
    }

    .ios-install-hint {
        position: absolute;
        top: calc(0.75rem + env(safe-area-inset-top));
        left: 0.75rem;
        right: 0.75rem;
        z-index: 6;
        display: flex;
        align-items: flex-start;
        gap: 0.5rem;
        padding: 0.6rem 0.75rem;
        color: #f2f5f8;
        background: rgba(18, 20, 22, 0.82);
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 0.6rem;
        backdrop-filter: blur(6px);
        -webkit-backdrop-filter: blur(6px);
        font-size: 0.85rem;
        line-height: 1.35;
    }

    .ios-install-hint p {
        margin: 0;
    }

    .ios-install-hint-close {
        flex: 0 0 auto;
        order: 2;
        width: 1.5rem;
        height: 1.5rem;
        padding: 0;
        color: #f2f5f8;
        background: transparent;
        border: none;
        font-size: 1.3rem;
        line-height: 1;
        cursor: pointer;
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