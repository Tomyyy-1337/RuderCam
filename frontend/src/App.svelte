<main class="container">
    <PageHeadder />
    <Navbar />

    {#if persistant_state.active_tab === "camera"}
        {#if temporary_state.is_connected}
            <Livestream />
            <Fahrt />
        {:else}
            <NotConnected />
        {/if}
    {:else if persistant_state.active_tab === "sessions"}
        <Fahrtenbuch />
        <div style="height: 10rem;"></div>
    {:else if persistant_state.active_tab === "settings"}
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

    import { persistant_state } from "./lib/classes/persistant_state_store.svelte";   
    import { highFrequencyUpdate } from "./lib/classes/high_frequency_update_store.svelte";
    import { deviceStatus } from "./lib/classes/device_status_store.svelte";
    import { activeSession } from "./lib/classes/active_session_store.svelte";
    import { temporary_state } from "./lib/classes/temporary_state_store.svelte";
    import type { AppTab } from "./lib/classes/persistant_state_store.svelte";
    import { setTheme, type Theme } from "./lib/classes/themeStore.svelte";
    import { decode } from "@msgpack/msgpack";

    let socket: WebSocket | null = null;
    let sessionActivityTimeout: number | null = null;

    onMount(() => {
        const theme = (localStorage.getItem("theme") as Theme | null) ?? "dark";
        setTheme(theme);

        const storedPersitsantState = localStorage.getItem("persistant_state");
        if (storedPersitsantState) {
            const parsedSettings = JSON.parse(storedPersitsantState);
            persistant_state.updateFromJson(parsedSettings);
        }

        const previousHistoryState = (window.history.state as { activeTab?: AppTab } | null) ?? {};
        if (previousHistoryState.activeTab !== persistant_state.active_tab) {
            window.history.replaceState({ ...previousHistoryState, activeTab: persistant_state.active_tab }, "", window.location.href);
        }

        const handlePopState = (): void => {
            const nextTab = (window.history.state as { activeTab?: AppTab } | null)?.activeTab;
            if (nextTab === "camera" || nextTab === "sessions" || nextTab === "settings") {
                persistant_state.active_tab = nextTab;
            }
        };

        window.addEventListener("popstate", handlePopState);

        connectWebSocket();

        return () => {
            clearSessionActivityTimeout();
            socket?.close();
            window.removeEventListener("popstate", handlePopState);
        };
    });

    $effect(() => {
        localStorage.setItem("persistant_state", persistant_state.toJSONstring());
    });

    function connectWebSocket(): void {
        try {
            socket = new WebSocket(`ws://${window.location.host}/ws`);
            socket.binaryType = "arraybuffer";
            socket.addEventListener("message", socketEventListener);
            socket.addEventListener("close", socketCloseListener);
        } catch (error) {
            console.error("Failed to create WebSocket:", error);
            scheduleReconnect();
        }
    }

    function socketEventListener(event: MessageEvent<ArrayBuffer>): void {
        let record = decode(event.data) as Record<string, unknown>
        temporary_state.is_connected = true;
        deviceStatus.updateFromMsgpack(record);
        highFrequencyUpdate.updateFromMsgpack(record);
        if (activeSession.updateFromMsgpack(record)) {
            temporary_state.session_is_active = true;
            scheduleSessionActivityTimeout();
        }
    }

    function socketCloseListener(): void {
        temporary_state.is_connected = false;
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
        temporary_state.session_is_active = false;
    }

    function scheduleReconnect(): void {
        window.setTimeout(() => {
            console.log("Attempting to reconnect WebSocket...");
            connectWebSocket();
        }, 5000);
    }
</script>