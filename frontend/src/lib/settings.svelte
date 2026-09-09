<section>
    <h3>Anzeige Einstellungen</h3>
    <SettingsToggleTheme />
    <SettingsOverlay bind:overlay_settings />

    <h3>Geräteverwaltung</h3>   
    
    <SettingsShutdownTimer {config} />
    <SettingsPowerButton />
    
    <h3>WLAN Einstellungen</h3>
    
    <SettingsChangeSsid {config} />
    <SettingsChangePassword {config} />

    <h3>Fahrtenbuch</h3>
    
    <SettingsFahrtenbuch {fahrtenbuch} />

    <h3>Update</h3>

    <Update />
</section>

<script>
    import { onMount } from "svelte";
    import SettingsChangePassword from "./settings_change_password.svelte";
    import SettingsChangeSsid from "./settings_change_ssid.svelte";
    import SettingsPowerButton from "./settings_power_button.svelte";
    import SettingsShutdownTimer from "./settings_shutdown_timer.svelte";
    import SettingsFahrtenbuch from "./settings_fahrtenbuch.svelte";
    import SettingsToggleTheme from "./settings_toggle_theme.svelte";
    import Update from "./update.svelte";
    import SettingsOverlay from "./settings_overlay.svelte";

    let { fahrtenbuch, overlay_settings = $bindable() } = $props();

    let config = $state({
        ssid: 'TestSSID',
        password: 'TestPassword',
        auto_shutdown_time: 5,
    });

    onMount(async () => {
        // Load config from localStorage if available for offline access
        let storedConfig = localStorage.getItem('config');
        if (storedConfig) {
            config = JSON.parse(storedConfig);
        }
        // Fetch latest config from backend 
        await fetchConfig();
    });

    $effect(() => {
        // Save config to localStorage whenever it changes 
        localStorage.setItem('config', JSON.stringify(config));
    });

    async function fetchConfig() {
        const response = await fetch('/api/get_config', {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
            }
        });
        if (response.ok) {
            const config_json = await response.json();
            config = {
                ssid: config_json.ssid || '',
                password: config_json.password || '',
                auto_shutdown_time: config_json.auto_shutdown_time || 5,
            };
        }
    }
</script>