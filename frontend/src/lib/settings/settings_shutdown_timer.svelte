<SettingsCard title="Automatisches Herunterfahren" description="Das Gerät fährt nach {app_config.auto_shutdown_time} Minuten automatisch herunter wenn keine Verbindung zum Smartphone besteht.">
    <input id="shutdown-timer" name="shutdown-timer" type="number" placeholder="Zeit in Minuten" bind:value={shutdownTimerInput}>
    <button style="width:100%;" onclick={handleSaveShutdownTimer}>
        Speichern
    </button>
    <Dialog
        bind:open={alertOpen}
        title={alertTitle}
        message={alertMessage}
        confirmLabel="OK"
        showCancel={false}
        tone="warning"
    />
</SettingsCard>

<script lang="ts">
    import { app_config } from "../classes/app_config.svelte";
    import Dialog from "../components/dialog.svelte";
    import SettingsCard from "./settings_card.svelte"

    let shutdownTimerInput = $state('');
    let alertOpen = $state(false);
    let alertTitle = $state('Ungültige Eingabe');
    let alertMessage = $state('');

    async function handleSaveShutdownTimer(): Promise<void> {
        const minutes = parseInt(shutdownTimerInput);
        shutdownTimerInput = '';
        if (isNaN(minutes) || minutes <= 0) {
            alertMessage = 'Bitte geben Sie eine gültige positive Zahl ein.';
            alertOpen = true;
            return;
        }
        app_config.auto_shutdown_time = minutes;
        await fetch('/api/set_shutdown_timer', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json', },
            body: JSON.stringify({ auto_shutdown_time: minutes })
        });
    }
</script>