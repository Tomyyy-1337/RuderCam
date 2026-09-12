<article>
    <h4>Herunterfahren und Neustarten</h4>
    <button onclick={handleShutdown} style="width:100%;">
        Gerät herunterfahren
    </button>
    <div class="spacer"></div>
    <button onclick={handleReboot} style="width:100%;">
        Gerät neustarten
    </button>
    <div class="spacer"></div>
</article>

<script lang="ts">
    async function handleShutdown(): Promise<void> {
        if (confirm('Möchten Sie das Gerät wirklich herunterfahren?')) {
            await fetch('/api/shutdown', { method: 'GET' });
        }
    }

    async function handleReboot(): Promise<void> {
        if (confirm('Möchten Sie das Gerät wirklich neustarten?')) {
            await fetch('/api/reboot', { method: 'GET' });
        }
    }
</script>