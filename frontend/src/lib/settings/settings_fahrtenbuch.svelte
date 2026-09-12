<article>
    <h4>Fahrtenbuch löschen</h4>
    <p>Lösche alle Fahrten aus dem Fahrtenbuch.</p>
    <button style="width:100%;" onclick={deleteFahrtenbuch}>
        Alle Fahrten löschen
    </button>
    <Dialog
        bind:open={confirmOpen}
        title="Fahrtenbuch löschen?"
        message="Möchten Sie wirklich alle Fahrten löschen? Diese Aktion kann nicht rückgängig gemacht werden."
        confirmLabel="Löschen"
        cancelLabel="Abbrechen"
        tone="danger"
        onConfirm={deleteConfirmed}
    />
    <div class="spacer"></div>
</article>

<script lang="ts">
    import Dialog from "../components/dialog.svelte";
    import type { Writable } from "svelte/store";
    import type { FahrtenbuchStore } from "../fahrtenbuch/fahrtenbuchStore";

    let { fahrtenbuch = $bindable() }: { fahrtenbuch: Writable<FahrtenbuchStore> } = $props();
    let confirmOpen = $state(false);

    function deleteFahrtenbuch(): void {
        confirmOpen = true;
    }

    function deleteConfirmed(): void {
        fahrtenbuch.update((store) => {
            store.clearHistory();
            return store;
        });
    }
</script>