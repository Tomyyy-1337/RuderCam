<article>
    <h4>SSID (Wlan Name) ändern</h4>
    <p>Aktuelle SSID: {config.ssid}</p>
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
    <div class="spacer"></div>
</article>

<script lang="ts">
    import Dialog from "../components/dialog.svelte";
    import type { AppConfig } from "../types";

    let { config }: { config: AppConfig } = $props();

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
        config.ssid = SSIDInput;
        await fetch('/api/set_wifi_ssid', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json', },
            body: JSON.stringify({ ssid: SSIDInput })
        });
        SSIDInput = '';
    }
</script>