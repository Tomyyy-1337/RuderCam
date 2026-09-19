<main class="container">
    <PageHeadder />
    <Navbar />

    {#if frontend_state.active_tab === "camera"}
        {#if frontend_state.is_connected}
            <Livestream />
            <Fahrt />
        {:else}
            <NotConnected />
        {/if}
    {:else if frontend_state.active_tab === "sessions"}
        <Fahrtenbuch />
        <div style="height: 10rem;"></div>
    {:else if frontend_state.active_tab === "settings"}
        <Settings />
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

    import { overlay_settings } from "./lib/classes/overlay_settings_store.svelte";   
    import { highFrequencyUpdate } from "./lib/classes/high_frequency_update_store.svelte";
    import { deviceStatus } from "./lib/classes/device_status_store.svelte";
    import { activeSession } from "./lib/classes/active_session_store.svelte";
    import { setTheme, type Theme } from "./lib/classes/themeStore.svelte";
    import { frontend_state, type AppTab } from "./lib/classes/frontend_state_store.svelte";

    let socket: WebSocket | null = null;
    let sessionActivityTimeout: number | null = null;

    onMount(() => {
        const theme = (localStorage.getItem("theme") as Theme | null) ?? "dark";
        setTheme(theme);

        const storedActiveTab = localStorage.getItem("active_tab");
        if (storedActiveTab) {
            frontend_state.active_tab = storedActiveTab as AppTab;
        }

        const previousHistoryState = (window.history.state as { activeTab?: AppTab } | null) ?? {};
        if (previousHistoryState.activeTab !== frontend_state.active_tab) {
            window.history.replaceState({ ...previousHistoryState, activeTab: frontend_state.active_tab }, "", window.location.href);
        }

        const handlePopState = (): void => {
            const nextTab = (window.history.state as { activeTab?: AppTab } | null)?.activeTab;
            if (nextTab === "camera" || nextTab === "sessions" || nextTab === "settings") {
                frontend_state.active_tab = nextTab;
            }
        };

        window.addEventListener("popstate", handlePopState);

        const storedOverlaySettings = localStorage.getItem("overlay_settings");
        if (storedOverlaySettings) {
            const parsedSettings = JSON.parse(storedOverlaySettings);
            overlay_settings.updateFromJson(parsedSettings);
        }

        connectWebSocket();

        return () => {
            clearSessionActivityTimeout();
            socket?.close();
            window.removeEventListener("popstate", handlePopState);
        };
    });

    $effect(() => {
        localStorage.setItem("active_tab", frontend_state.active_tab);
    });

    $effect(() => {
        localStorage.setItem("overlay_settings", overlay_settings.toJSONstring());
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
        const payload = JSON.parse(event.data);
        frontend_state.is_connected = true;
        deviceStatus.updateFromJson(payload);
        highFrequencyUpdate.updateFromJson(payload);
        if (activeSession.updateFromJson(payload)) {
            if (!frontend_state.session_is_active || Math.abs(Date.parse(payload.client_time) - Date.now()) > 2000) {
                frontend_state.session_is_active = true;
            }
            scheduleSessionActivityTimeout();
        }
    }

    function socketCloseListener(): void {
        frontend_state.is_connected = false;
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
        frontend_state.session_is_active = false;
    }

    function scheduleReconnect(): void {
        window.setTimeout(() => {
            console.log("Attempting to reconnect WebSocket...");
            connectWebSocket();
        }, 5000);
    }
</script>