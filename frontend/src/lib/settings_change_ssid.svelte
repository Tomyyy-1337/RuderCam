<article>
    <h4>SSID ändern</h4>
    <p>Aktuelle SSID: {config.ssid}</p>
    <input id="ssid" name="ssid" placeholder="Netzwerkname" bind:value={SSIDInput}>
    <button 
        style="width:100%;" 
        onclick={handleSSIdChange}
    > Speichern </button>
    <div class="spacer"></div>
</article>

<script lang="ts">
    let {config} = $props();

    let SSIDInput = $state('');

    async function handleSSIdChange() {
        if (SSIDInput.trim() === '') {
            alert('Bitte geben Sie eine gültige SSID ein.');
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