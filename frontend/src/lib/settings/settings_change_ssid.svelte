<SettingsCard title="SSID (Wlan Name) ändern" description="Aktuelle SSID: {app_config.ssid}">
    <p>Das Ändern der SSID wird nach dem nächsten Neustart des Geräts wirksam.</p>
    <p>Die Verbindung zur Kamera ist nach dem Ändern der SSID nur noch über den neuen Netzwerknamen möglich. Um sich wieder zu verbinden, müssen Sie in den Netzwerkeinstellungen Ihres Geräts das neue WLAN auswählen und das Passwort eingeben.</p>

    <input id="ssid" name="ssid" placeholder="Netzwerkname" bind:value={SSIDInput}>
    <button 
        style="width:100%;" 
        onclick={handleSSIdChange}
    > Speichern </button>
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
    import SettingsCard from "./settings_card.svelte";

    let SSIDInput = $state('');
    let alertOpen = $state(false);
    let alertTitle = $state('Ungültige SSID');
    let alertMessage = $state('');

    async function handleSSIdChange(): Promise<void> {
        if (SSIDInput.trim() === '') {
            alertMessage = 'Bitte geben Sie eine gültige SSID ein.';
            alertOpen = true;
            return;
        }
        app_config.ssid = SSIDInput;
        await fetch('/api/set_wifi_ssid', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json', },
            body: JSON.stringify({ ssid: SSIDInput })
        });
        SSIDInput = '';
    }
</script>