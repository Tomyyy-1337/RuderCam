<main class="container">
    <PageHeadder {deviceStatus} />
    <Navbar bind:activeTab />

    {#if activeTab === "camera"}
        {#if deviceStatus.isConnected}
            <Livestream {deviceStatus} bind:activeSession {overlay_settings} />
            <Fahrt bind:activeSession />
        {:else}
            <NotConnected />
        {/if}
    {:else if activeTab === "sessions"}
        <Fahrtenbuch />
        <div style="height: 10rem;"></div>
    {:else if activeTab === "settings"}
        <Settings {deviceStatus} bind:overlay_settings />
        <div style="height: 10rem;"></div>
    {/if}
</main>

<script lang="ts">
    import { onMount } from "svelte";
    import Fahrt from "./lib/components/aktuelle_fahrt.svelte";
    import Fahrtenbuch from "./lib/fahrtenbuch/fahrtenbuch.svelte";
    import Livestream from "./lib/stream/livestream.svelte";
    import Navbar from "./lib/components/navbar.svelte";
    import NotConnected from "./lib/components/not_connected.svelte";
    import PageHeadder from "./lib/components/pageHeadder.svelte";
    import Settings from "./lib/settings/settings.svelte";
    import { setTheme } from "./lib/settings/setTheme";
    import {
        isDeviceStateMessage,
        isOverlaySettings,
        isRunningSessionMessage,
    } from "./lib/types";
    import type {
        ActiveSession,
        AppTab,
        DeviceStatus,
        OverlaySettings,
        Theme,
    } from "./lib/types";

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

    let activeTab = $state<AppTab>("camera");

    let socket: WebSocket | null = null;
    let sessionActivityTimeout: number | null = null;

    onMount(() => {
        const theme = (localStorage.getItem("theme") as Theme | null) ?? "dark";
        setTheme(theme);

        const storedActiveTab = localStorage.getItem("active_tab");
        if (storedActiveTab) {
            activeTab = storedActiveTab as AppTab;
        }

        const previousHistoryState = (window.history.state as { activeTab?: AppTab } | null) ?? {};
        if (previousHistoryState.activeTab !== activeTab) {
            window.history.replaceState({ ...previousHistoryState, activeTab }, "", window.location.href);
        }

        const handlePopState = (): void => {
            const nextTab = (window.history.state as { activeTab?: AppTab } | null)?.activeTab;
            if (nextTab === "camera" || nextTab === "sessions" || nextTab === "settings") {
                activeTab = nextTab;
            }
        };

        window.addEventListener("popstate", handlePopState);

        const storedOverlaySettings = localStorage.getItem("overlay_settings");
        if (storedOverlaySettings) {
            try {
                const parsedSettings: unknown = JSON.parse(storedOverlaySettings);
                if (isOverlaySettings(parsedSettings)) {
                    overlay_settings = parsedSettings;
                }
            } catch (error) {
                console.error("Failed to parse overlay settings from localStorage:", error);
            }
        }

        connectWebSocket();

        return () => {
            clearSessionActivityTimeout();
            socket?.close();
            window.removeEventListener("popstate", handlePopState);
        };
    });

    $effect(() => {
        localStorage.setItem("active_tab", activeTab);
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
        const payload: unknown = JSON.parse(event.data);
        deviceStatus.isConnected = true;
        if (isDeviceStateMessage(payload)) {
            deviceStatus.battery_percentage = payload.battery_percentage;
            deviceStatus.speed_kmh = payload.velocity;
            deviceStatus.schlagzahl = payload.schlagzahl;
            deviceStatus.satellite_count = payload.satellite_count;
        } else if (isRunningSessionMessage(payload)) {
            if (!activeSession.isActive || Math.abs(Date.parse(payload.client_time) - Date.now()) > 2000) {
                activeSession.isActive = true;
            }

            scheduleSessionActivityTimeout();
            activeSession.client_time = Date.parse(payload.client_time);
            activeSession.distance_traveled_km = payload.distance_traveled_km;
            activeSession.average_speed_kmh = payload.average_speed_kmh;
            activeSession.max_speed_kmh = payload.max_speed_kmh;
            activeSession.average_bpm = payload.average_bpm;
            activeSession.duration_secs = payload.duration_secs;
            activeSession.pausiert = payload.pausiert;
        }
    }

    function socketCloseListener(): void {
        deviceStatus.isConnected = false;
        deactivateSession();
        scheduleReconnect();
    }

    function scheduleSessionActivityTimeout(): void {
        clearSessionActivityTimeout();
        sessionActivityTimeout = window.setTimeout(() => {
            deactivateSession();
        }, 10000);
    }

    function clearSessionActivityTimeout(): void {
        if (sessionActivityTimeout !== null) {
            clearTimeout(sessionActivityTimeout);
            sessionActivityTimeout = null;
        }
    }

    function deactivateSession(): void {
        clearSessionActivityTimeout();
        activeSession.isActive = false;
    }

    function scheduleReconnect(): void {
        window.setTimeout(() => {
            console.log("Attempting to reconnect WebSocket...");
            connectWebSocket();
        }, 5000);
    }
</script>