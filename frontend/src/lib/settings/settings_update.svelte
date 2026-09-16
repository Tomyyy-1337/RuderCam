<SettingsCard title="Update Firmware" description="Aktuelle Version: {currentVersion}">

    <p>Um die aktuelle version der Firmware herunterzuladen trennen sie die WLAN Verbindung zur Ruder Kamera und verbinden sie sich mit Mobilen Daten oder einem anderen WLAN.</p>

    <p>Nach dem Herunterladen der Firmware stellen Sie die Verbindung zum WLAN der Ruder Kamera wieder her und öffnen Sie den Update service um das Update durchzuführen.</p>

    <p>Die neuste Version der Firmware finden Sie <a class="release-link" href="https://github.com/Tomyyy-1337/RuderCam/releases/latest" target="_blank" rel="noreferrer">hier</a>.</p>

    <button style="width:100%;" onclick={() => window.open(`http://${window.location.hostname}:4000`, '_blank')}>Update Service öffnen</button>

</SettingsCard>

<script>
    import { onMount } from 'svelte';
    import SettingsCard from './settings_card.svelte';

    let currentVersion = $state('Loading...');

    onMount(async () => {
        try {
            const response = await fetch('/api/get_firmware_version');
            if (response.ok) {
                const version = await response.text();
                currentVersion = version.trim();
            } else {
                currentVersion = 'Error fetching version';
            }
        } catch (error) {
            console.error('Error fetching firmware version:', error);
            currentVersion = 'Error fetching version';
        }
    });
    
</script>

<style>
    .release-link {
        color: color-mix(in srgb, var(--button-color) 82%, white 18%);
        text-underline-offset: 0.14em;
        text-decoration-thickness: 0.08em;
        transition: color 160ms ease;
    }

    .release-link:hover {
        color: color-mix(in srgb, var(--button-color) 92%, white 8%);
    }

    .release-link:focus-visible {
        outline: 2px solid color-mix(in srgb, var(--button-color) 88%, white 12%);
        outline-offset: 2px;
        border-radius: 0.2em;
    }
</style>