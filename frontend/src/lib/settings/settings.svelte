<div class="settings-page">
    <AccordionSection
        title="Anzeige"
        description="Einstellungen für Theme und das Livestream Overlay"
    >
        <SettingsToggleTheme />
        <SettingsOverlay />
    </AccordionSection>

    {#if temporary_state.is_connected}
        <AccordionSection
            title="Kamera"
            description="Einstellungen für Fokus und Belichtungsmessung"
        >
            <SettingsCamera />
        </AccordionSection>

        <AccordionSection
            title="Geräteverwaltung"
            description="Einstellungen für Automatisches Herunterfahren und Wlan"
        >
            <SettingsShutdownTimer />
            <SettingsChangeSsid />
            <SettingsChangePassword />
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
    import { app_config } from "../classes/app_config.svelte";
    import { temporary_state } from "../classes/temporary_state_store.svelte";

    onMount(async () => {
        await app_config.fetch();
    });
</script>

<style>
    .settings-page {
        display: grid;
        gap: 1rem;
        width: 100%;
    }
</style>