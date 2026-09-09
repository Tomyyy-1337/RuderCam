<section>
    <div class="video-shell" bind:this={videoShell}>
        <video id="myvideo" controls muted autoplay playsinline></video>
        {#if fullscreen}
           <Overlay {deviceStatus} bind:activeSession {fahrtenbuch} {overlay_settings} />
        {/if}
    </div>
    <div class="spacer"></div>
    <button onclick={activateFullscreen} class="fullscreen-btn">
        Vollbild
    </button>
</section>

<script>
    import { onMount } from "svelte";
    import "./reader.js";
    import Overlay from "./overlay.svelte";
    
    let videoShell;
    let reader = null;

    let fullscreen = $state(false);
    let {deviceStatus, activeSession = $bindable(), fahrtenbuch, overlay_settings} = $props();

    onMount(() => {
        connectReader();

        const handleFullscreenChange = () => {
            fullscreen = document.fullscreenElement === videoShell;

            if (!fullscreen) {
                screen.orientation?.unlock?.();
            }
        };

        document.addEventListener("fullscreenchange", handleFullscreenChange);

        const video = document.getElementById("myvideo");
        video.removeAttribute("controls");

        return () => {
            document.removeEventListener("fullscreenchange", handleFullscreenChange);
        };
    })

    const activateFullscreen = async () => {
        if (videoShell?.requestFullscreen) {
            await videoShell.requestFullscreen();

            try {
                await screen.orientation?.lock?.("landscape");
            } catch (err) {
                console.warn("Could not lock orientation to landscape", err);
            }
        }
    };
    
    let retryTimer = null;

    async function connectReader() {
        if (retryTimer) {
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
                    try { reader?.close(); } catch {}
                    reader = null;
                    retryTimer = setTimeout(connectReader, 2000);
                },
                onTrack: (evt) => {
                    clearTimeout(retryTimer);
                    retryTimer = null;
                    document.getElementById("myvideo").srcObject = evt.streams[0];
                },
                onDataChannel: (evt) => {
                    evt.channel.binaryType = "arraybuffer";
                    evt.channel.onmessage = (evt) => {
                        console.log("data channel message", evt.data);
                    };
                },
            });
        } catch (err) {
            console.error("connectReader exception:", err);
            retryTimer = setTimeout(connectReader, 1000);
        }
    }
    
    window.addEventListener("beforeunload", () => {
        if (retryTimer) clearTimeout(retryTimer);
        if (reader !== null) {
            try { reader.close(); } catch {}
            reader = null;
        }
    });
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
        position: relative;
        background: black;
        border-radius: 0.75rem;
        overflow: hidden;
    }
    
    video {
        transform: scaleX(-1);
    }

    .video-shell video {
        max-width: 100%;
        max-height: 100%;
        display: block;
        background: black;
    }

    .video-shell::backdrop {
        background: black;
    }

    .video-shell:fullscreen {
        max-width: 100%;
        max-height: 100%;
        place-items: center;
        background: black;
        border-radius: 0;
        overflow: visible;
    }

    .video-shell:fullscreen video {
        max-height: 100%;
        max-width: 100%;
        object-fit: contain;
    }

    .fullscreen-btn {
        padding: 0.75rem 2rem;
        font-size: 1.25rem;
        font-weight: bold;
        background-color: #007bff;
        color: white;
        border: none;
        border-radius: 0.5rem;
        cursor: pointer;
        width: 80%;
        transition: background-color 0.3s;
    }

    .fullscreen-btn:hover {
        background-color: #0056b3;
    }
</style>