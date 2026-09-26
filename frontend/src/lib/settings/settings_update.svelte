<SettingsCard title="Update Firmware" description="Aktuelle Version: {currentVersion}">
    <UpdateHelper />
</SettingsCard>

<script>
    import { onMount } from 'svelte';
    import SettingsCard from './settings_card.svelte';
    import UpdateHelper from '../update_helper.rs/update_helper.svelte';

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