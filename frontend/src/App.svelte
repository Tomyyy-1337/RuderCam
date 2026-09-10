<main class="container">
    {#if deviceStatus.isConnected}
        <PageHeadder {deviceStatus} />
        <Livestream {deviceStatus} bind:activeSession {fahrtenbuch} {overlay_settings} />
        <Tacho {deviceStatus} />
        <Fahrt {fahrtenbuch} bind:activeSession />
    {:else}
        <NotConnected />
    {/if}

    <section>
        <Navbar bind:activeTab />
        
        {#if activeTab === 'settings'}
            <Settings {fahrtenbuch} bind:overlay_settings />
        {:else if activeTab === 'sessions'}
            <Fahrtenbuch {fahrtenbuch} />
        {/if}
    </section>

    <div style="height: 400px;"></div>
</main>

<script lang="ts">
    import { onMount } from "svelte";
    import { writable, type Writable } from "svelte/store";
    import Fahrt from "./lib/fahrt.svelte";
    import Fahrtenbuch from "./lib/fahrtenbuch.svelte";
    import { FahrtenbuchStore } from "./lib/fahrtenbuchStore";
    import Livestream from "./lib/livestream.svelte";
    import Navbar from "./lib/navbar.svelte";
    import NotConnected from "./lib/not_connected.svelte";
    import PageHeadder from "./lib/pageHeadder.svelte";
    import Settings from "./lib/settings.svelte";
    import { setTheme } from "./lib/setTheme";
    import Tacho from "./lib/tacho.svelte";
    import type {
        ActiveSession,
        DeviceStateMessage,
        DeviceStatus,
        OverlaySettings,
        RunningSessionMessage,
        Theme,
    } from "./lib/types";

    function isOverlaySettings(value: unknown): value is OverlaySettings {
        return !!value && typeof value === "object" && "show_overlay" in value && "position" in value;
    }

    function isDeviceState(message: unknown): message is DeviceStateMessage {
        return !!message
            && typeof message === "object"
            && "velocity" in message
            && "satellite_count" in message
            && "schlagzahl" in message
            && "battery_percentage" in message;
    }

    function isRunningSession(message: unknown): message is RunningSessionMessage {
        return !!message
            && typeof message === "object"
            && "client_time" in message
            && "distance_traveled_km" in message
            && "average_speed_kmh" in message
            && "max_speed" in message
            && "average_bpm" in message
            && "duration_secs" in message
            && "pausiert" in message;
    }

    let fahrtenbuch: Writable<FahrtenbuchStore> = writable(new FahrtenbuchStore());

    let deviceStatus = $state<DeviceStatus>({
        isConnected: true,
        battery_percentage: 0,
        speed_kmh: 0,
        schlagzahl: 0,
        satellite_count: 0,
    });

    let activeSession = $state<ActiveSession>({
        isActive: false,
        end_time: new Date(),
        client_time: null,
        distance_traveled_km: 0,
        average_speed_kmh: 0,
        max_speed_kmh: 0,
        average_bpm: 0,
        duration_secs: 0,
        pausiert: true,
    });

    let overlay_settings = $state<OverlaySettings>({
        show_overlay: true,
        position: "top",
        show_speed: true,
        show_split_time: true,
        show_schlagzahl: true,
        show_fahrtzeit: true,
        show_distanz: true,
        show_distanc_per_stroke: true,
    });

    let activeTab = $state("sessions");

    let socket: WebSocket | null = null;

    onMount(() => {
        const theme = (localStorage.getItem("theme") as Theme | null) ?? "dark";
        setTheme(theme);

        const storedOverlaySettings = localStorage.getItem("overlay_settings");
        if (storedOverlaySettings) {
            try {
                const parsedSettings = JSON.parse(storedOverlaySettings) as unknown;
                if (isOverlaySettings(parsedSettings)) {
                    overlay_settings = parsedSettings;
                }
            } catch (error) {
                console.error("Failed to parse overlay settings from localStorage:", error);
            }
        }

        connectWebSocket();

        return () => {
            socket?.close();
        };
    });

    $effect(() => {
        localStorage.setItem("overlay_settings", JSON.stringify(overlay_settings));
    });

    function connectWebSocket(): void {
        try {
            socket = new WebSocket(`ws://${window.location.host}/ws`);
            socket.addEventListener("message", socketEventListener);
            socket.addEventListener("close", socketCloseListener);
        } catch (error) {
            console.error("Failed to create WebSocket:", error);
            scheduleReconnect();
        }
    }

    function socketEventListener(event: MessageEvent<string>): void {
        const payload = JSON.parse(event.data) as unknown;
        deviceStatus.isConnected = true;

        if (isDeviceState(payload)) {
            deviceStatus.battery_percentage = payload.battery_percentage;
            deviceStatus.speed_kmh = payload.velocity;
            deviceStatus.schlagzahl = payload.schlagzahl;
            deviceStatus.satellite_count = payload.satellite_count;
        } else if (isRunningSession(payload)) {
            if (!activeSession.isActive || Math.abs(Date.parse(payload.client_time) - Date.now()) > 2000) {
                activeSession.isActive = true;
            }
            activeSession.client_time = Date.parse(payload.client_time);
            activeSession.distance_traveled_km = payload.distance_traveled_km;
            activeSession.average_speed_kmh = payload.average_speed_kmh;
            activeSession.max_speed_kmh = payload.max_speed;
            activeSession.average_bpm = payload.average_bpm;
            activeSession.duration_secs = payload.duration_secs;
            activeSession.pausiert = payload.pausiert;
        }
    }

    function socketCloseListener(): void {
        deviceStatus.isConnected = false;
        activeSession.isActive = false;
        scheduleReconnect();
    }

    function scheduleReconnect(): void {
        window.setTimeout(() => {
            console.log("Attempting to reconnect WebSocket...");
            connectWebSocket();
        }, 5000);
    }
</script>