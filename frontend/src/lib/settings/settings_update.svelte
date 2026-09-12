<article>
    <h4>Update Firmware</h4>

    <p>Aktuelle Version: {currentVersion}</p>

    <button style="width:100%;" onclick={() => window.open(`http://${window.location.hostname}:4000`, '_blank')}>Update Service öffnen</button>

    <div class="spacer"></div>
</article>

<script>
    import { onMount } from 'svelte';

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