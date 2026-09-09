<main class="container">
    {#if deviceStatus.isConnected}
        <PageHeadder {deviceStatus} />
        <Livestream {deviceStatus} bind:activeSession {fahrtenbuch} {overlay_settings} />
        <Tacho {deviceStatus} />
        <Fahrt {fahrtenbuch} bind:activeSession />
    {:else}
        <NotConnected />
    {/if}

    <Navbar bind:activeTab />

    {#if activeTab === 'settings'}
        <Settings {fahrtenbuch} bind:overlay_settings />
    {:else if activeTab === 'sessions'}
        <Fahrtenbuch {fahrtenbuch} />
    {/if}

    <div style="height: 400px;"></div>
</main>

<script>
// @ts-nocheck

    import { writable } from "svelte/store";
    import Fahrt from "./lib/fahrt.svelte";
    import Fahrtenbuch from "./lib/fahrtenbuch.svelte";
    import { FahrtenbuchStore } from "./lib/fahrtenbuchStore";
    import Livestream from "./lib/livestream.svelte";
    import Settings from "./lib/settings.svelte";
    import Navbar from "./lib/navbar.svelte";
    import NotConnected from "./lib/not_connected.svelte";
    import { onMount } from "svelte";
    import Tacho from "./lib/tacho.svelte";
    import { setTheme } from "./lib/setTheme";
    import PageHeadder from "./lib/pageHeadder.svelte";

    let fahrtenbuch = writable(new FahrtenbuchStore());

    let deviceStatus = $state({
        isConnected: true,
        battery_percentage: 0,
        speed_kmh: 0,
        schlagzahl: 0,
        satellite_count: 0
    });

    let activeSession = $state({
        isActive: false,
        end_time: new Date(),
        client_time: null,
        distance_traveled_km: 0,
        average_speed_kmh: 0,
        max_speed_kmh: 0,
        average_bpm: 0,
        duration_secs: 0,
        pausiert: true
    });

    let overlay_settings = $state({
        show_overlay: true,
        position: "top",
        show_speed: true,
        show_split_time: true,
        show_schlagzahl: true,
        show_fahrtzeit: true,
        show_distanz: true,
        show_distanc_per_stroke: true
    });

    let activeTab = $state('sessions');

    let socket;
    
    onMount(() => { 
        let theme = localStorage.getItem('theme') || 'dark';
        if (theme) {
            setTheme(theme);
        }

        const storedOverlaySettings = localStorage.getItem('overlay_settings');
        if (storedOverlaySettings) {
            try {
                overlay_settings = JSON.parse(storedOverlaySettings) || overlay_settings;
            } catch (error) {
                console.error('Failed to parse overlay settings from localStorage:', error);
            }
        }

        connectWebSocket();
    });

    $effect(() => {
        if (typeof localStorage !== 'undefined') {
            localStorage.setItem('overlay_settings', JSON.stringify(overlay_settings));
        }
    });

    function connectWebSocket() {
        try {
            socket = new WebSocket('ws://' + window.location.host + '/ws');
            
            socket.addEventListener('message', socketEventListener);
            socket.addEventListener('close', socketCloseListener);
        } catch (error) {
            console.error('Failed to create WebSocket:', error);
            scheduleReconnect();
        }
    }

    function socketEventListener(event) {
        let json = JSON.parse(event.data);
        deviceStatus.isConnected = true;

        if (isDeviceState(json)) {
            deviceStatus.battery_percentage = json.battery_percentage;
            deviceStatus.speed_kmh = json.velocity;
            deviceStatus.schlagzahl = json.schlagzahl;
            deviceStatus.satellite_count = json.satellite_count;
        } else if (isRunningSession(json)) {
            if (!activeSession.isActive || Math.abs(Date.parse(json.client_time) - new Date()) > 2000) {
                activeSession.isActive = true;
            }
            activeSession.client_time = Date.parse(json.client_time);
            activeSession.distance_traveled_km = json.distance_traveled_km;
            activeSession.average_speed_kmh = json.average_speed_kmh;
            activeSession.max_speed_kmh = json.max_speed;
            activeSession.average_bpm = json.average_bpm;
            activeSession.duration_secs = json.duration_secs;
            activeSession.pausiert = json.pausiert;
        }
    }

    function socketCloseListener() {
        deviceStatus.isConnected = false;
        activeSession.isActive = false;
        scheduleReconnect();
    }

    function isDeviceState(message) {
        return message
            && typeof message === 'object'
            && 'velocity' in message
            && 'satellite_count' in message
            && 'schlagzahl' in message
            && 'battery_percentage' in message;
    }

    function isRunningSession(message) {
        return message
            && typeof message === 'object'
            && 'client_time' in message
            && 'distance_traveled_km' in message
            && 'average_speed_kmh' in message
            && 'max_speed_kmh' in message
            && 'average_bpm' in message
            && 'duration_secs' in message
            && 'pausiert' in message;
    }

    function scheduleReconnect() {
        setTimeout(() => {
            console.log('Attempting to reconnect WebSocket...');
            connectWebSocket();
        }, 5000);
    }

</script>