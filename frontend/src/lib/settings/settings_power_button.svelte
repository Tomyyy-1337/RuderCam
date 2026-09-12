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

    <Dialog
        bind:open={confirmOpen}
        title={confirmTitle}
        message={confirmMessage}
        confirmLabel={confirmLabel}
        cancelLabel="Abbrechen"
        tone="danger"
        onConfirm={handleConfirmedAction}
    />
</article>

<script lang="ts">
    import Dialog from "../components/dialog.svelte";

    let confirmOpen = $state(false);
    let confirmTitle = $state('');
    let confirmMessage = $state('');
    let confirmLabel = $state('OK');
    let pendingAction = $state<'shutdown' | 'reboot' | null>(null);

    async function handleShutdown(): Promise<void> {
        confirmTitle = 'Gerät herunterfahren?';
        confirmMessage = 'Möchten Sie das Gerät wirklich herunterfahren?';
        confirmLabel = 'Herunterfahren';
        pendingAction = 'shutdown';
        confirmOpen = true;
    }

    async function handleReboot(): Promise<void> {
        confirmTitle = 'Gerät neustarten?';
        confirmMessage = 'Möchten Sie das Gerät wirklich neustarten?';
        confirmLabel = 'Neustarten';
        pendingAction = 'reboot';
        confirmOpen = true;
    }

    async function handleConfirmedAction(): Promise<void> {
        if (pendingAction === 'shutdown') {
            await fetch('/api/shutdown', { method: 'GET' });
        }
        if (pendingAction === 'reboot') {
            await fetch('/api/reboot', { method: 'GET' });
        }
        pendingAction = null;
    }
</script>