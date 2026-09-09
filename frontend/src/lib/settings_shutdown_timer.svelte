<article>
    <h4>Automatisches Herunterfahren</h4>
    <p>Das Gerät fährt nach {config.auto_shutdown_time} Minuten automatisch herunter wenn keine Verbindung zum Smartphone besteht.</p>
    <input id="shutdown-timer" type="number" placeholder="Zeit in Minuten" bind:value={shutdownTimerInput}>
    <button style="width:100%;" onclick={handleSaveShutdownTimer}>
        Speichern
    </button>
    <div class="spacer"></div>
</article>

<script>
    let { config } = $props();
    let shutdownTimerInput = $state('');

    async function handleSaveShutdownTimer() {
        const minutes = parseInt(shutdownTimerInput);
        shutdownTimerInput = '';
        if (isNaN(minutes) || minutes <= 0) {
            alert('Bitte geben Sie eine gültige positive Zahl ein.');
            return;
        }
        config.auto_shutdown_time = minutes;
        await fetch('/api/set_shutdown_timer', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json', },
            body: JSON.stringify({ auto_shutdown_time: minutes })
        });
    }
</script>