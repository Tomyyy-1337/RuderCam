<div class="settings-page">
    <AccordionSection
        title="Anzeige"
        description="Einstellungen für Theme und das Livestream Overlay"
    >
        <SettingsToggleTheme />
        <SettingsOverlay bind:overlay_settings />
    </AccordionSection>

    {#if deviceStatus.isConnected}
        <AccordionSection
            title="Kamera"
            description="Einstellungen für Fokus und Belichtungsmessung"
        >
            <SettingsCamera {config} bind:overlay_settings />
        </AccordionSection>

        <AccordionSection
            title="Geräteverwaltung"
            description="Einstellungen für Automatisches Herunterfahren und Wlan"
        >
            <SettingsShutdownTimer {config} />
            <SettingsChangeSsid {config} />
            <SettingsChangePassword {config} />
        </AccordionSection>
    {/if}

    <AccordionSection
        title="Fahrtenbuch"
        description="Gespeicherte Fahrten verwalten"
    >
        <SettingsFahrtenbuch />
    </AccordionSection>

    <AccordionSection
        title="Update"
        description="Neue Versionen auf das Gerät übertragen"
    >
        <SettingsUpdate />
    </AccordionSection>

    <AccordionSection
        title="Entwickleroptionen"
        description="Optionen für Entwickler"
    >
        <SettingsDeveloper />
    </AccordionSection>
</div>

<script lang="ts">
    import { onMount } from "svelte";
    import { isAppConfig } from "../types";
    import type { AppConfig, DeviceStatus, OverlaySettings } from "../types";
    import AccordionSection from "./accordion_section.svelte";
    import SettingsToggleTheme from "./settings_toggle_theme.svelte";
    import SettingsOverlay from "./settings_overlay.svelte";
    import SettingsCamera from "./settings_camera.svelte";
    import SettingsShutdownTimer from "./settings_shutdown_timer.svelte";
    import SettingsChangeSsid from "./settings_change_ssid.svelte";
    import SettingsChangePassword from "./settings_change_password.svelte";
    import SettingsFahrtenbuch from "./settings_fahrtenbuch.svelte";
    import SettingsDeveloper from "./settings_developer.svelte";
    import SettingsUpdate from "./settings_update.svelte";

    const defaultConfig: AppConfig = {
        ssid: "TestSSID",
        password: "TestPassword",
        auto_shutdown_time: 5,
        focus_mode: "Fixed",
        metering_mode: "Average",
        bitrate: 1600000,
        exposure_compenstion: 0,
    };

    function toConfig(value: unknown): AppConfig {
        if (!isAppConfig(value)) {
            return defaultConfig;
        }

        return {
            ssid: value.ssid || defaultConfig.ssid,
            password: value.password || defaultConfig.password,
            auto_shutdown_time: Number(value.auto_shutdown_time || defaultConfig.auto_shutdown_time),
            focus_mode: value.focus_mode || defaultConfig.focus_mode,
            metering_mode: value.metering_mode || defaultConfig.metering_mode,
            bitrate: Number(value.bitrate || defaultConfig.bitrate),
            exposure_compenstion: Number(value.exposure_compenstion || defaultConfig.exposure_compenstion),
        };
    }

    let { deviceStatus, overlay_settings = $bindable() }: {
        deviceStatus: DeviceStatus;
        overlay_settings: OverlaySettings;
    } = $props();

    let config = $state<AppConfig>(defaultConfig);

    onMount(async () => {
        const storedConfig = localStorage.getItem("config");
        if (storedConfig) {
            try {
                const parsedConfig: unknown = JSON.parse(storedConfig);
                config = toConfig(parsedConfig);
            } catch {
                config = defaultConfig;
            }
        }

        await fetchConfig();
    });

    $effect(() => {
        localStorage.setItem("config", JSON.stringify(config));
    });

    async function fetchConfig(): Promise<void> {
        const response = await fetch("/api/get_config", {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
            },
        });

        if (response.ok) {
            const parsedConfig: unknown = await response.json();
            config = toConfig(parsedConfig);
        }
    }
</script>

<style>
    .settings-page {
        display: grid;
        gap: 1rem;
        width: 100%;
    }
</style>